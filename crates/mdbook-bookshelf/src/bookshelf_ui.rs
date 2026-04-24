use crate::root_bookshelf_preprocessor::ROOT_BOOKSHELF_HTML_PATH;
use anyhow::{bail, Context, Result};
use mdbook_driver::config::Config;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

const BOOKSHELF_UI_ASSET_DIR: &str = ".mdbook-bookshelf";
const BOOKSHELF_UI_SHARED_DIR: &str = "shared";
const BOOKSHELF_UI_BOOKS_DIR: &str = "books";
const BOOKSHELF_STAGED_CONFIG_ASSET_DIR: &str = "bookshelf-config-assets";
const BOOKSHELF_BREADCRUMB_CSS_NAME: &str = "bookshelf-breadcrumb.css";
const BOOKSHELF_BREADCRUMB_JS_NAME: &str = "bookshelf-breadcrumb.js";
const BOOKSHELF_BREADCRUMB_CLASS: &str = "bookshelf-breadcrumb";
const BOOKSHELF_BREADCRUMB_ID: &str = "bookshelf-breadcrumb";
const BOOKSHELF_BREADCRUMB_ATTRIBUTE: &str = "data-bookshelf-breadcrumb";
const BOOKSHELF_RETURN_CSS_NAME: &str = "bookshelf-return.css";
const BOOKSHELF_RETURN_JS_NAME: &str = "bookshelf-return.js";
const BOOKSHELF_RETURN_LINK_CLASS: &str = "bookshelf-return-link";
const BOOKSHELF_RETURN_LINK_ID: &str = "bookshelf-return-link";
const BOOKSHELF_SEARCH_JS_NAME: &str = "bookshelf-search.js";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct BookshelfBreadcrumbPage {
    pub html_path: String,
    pub breadcrumb: String,
}

pub struct TransientBookshelfUiAssets {
    config_root: PathBuf,
    build_rel_root: PathBuf,
    build_abs_root: PathBuf,
    cleaned: bool,
}

impl TransientBookshelfUiAssets {
    pub fn new(config_root: &Path) -> Result<Self> {
        for attempt in 0..16 {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .context("system clock is before the unix epoch")?
                .as_nanos();
            let suffix = if attempt == 0 {
                format!("build-{}-{nanos}", process::id())
            } else {
                format!("build-{}-{nanos}-{attempt}", process::id())
            };
            let build_rel_root = PathBuf::from(BOOKSHELF_UI_ASSET_DIR).join(suffix);
            let build_abs_root = config_root.join(&build_rel_root);

            if build_abs_root.exists() {
                continue;
            }

            fs::create_dir_all(&build_abs_root).with_context(|| {
                format!(
                    "failed to create transient bookshelf UI asset root {}",
                    build_abs_root.display()
                )
            })?;

            return Ok(Self {
                config_root: config_root.to_path_buf(),
                build_rel_root,
                build_abs_root,
                cleaned: false,
            });
        }

        bail!(
            "failed to allocate a unique transient bookshelf UI asset root under {}",
            config_root.display()
        );
    }

    pub fn inject_bookshelf_ui_assets(
        &self,
        config: &mut Config,
        asset_key: &str,
        return_target: &str,
        searchindex_target: &str,
        breadcrumb_pages: &[BookshelfBreadcrumbPage],
    ) -> Result<()> {
        let return_css_rel_path = self.bookshelf_return_css_path();
        let return_js_rel_path = self.bookshelf_return_js_path(asset_key);
        let breadcrumb_css_rel_path = self.bookshelf_breadcrumb_css_path();
        let breadcrumb_js_rel_path = self.bookshelf_breadcrumb_js_path(asset_key);
        let search_js_rel_path = self.bookshelf_search_js_path();

        write_asset_file(
            &self.config_root.join(&return_css_rel_path),
            &render_bookshelf_return_css(),
        )?;
        write_asset_file(
            &self.config_root.join(&return_js_rel_path),
            &render_bookshelf_return_js(return_target),
        )?;
        write_asset_file(
            &self.config_root.join(&breadcrumb_css_rel_path),
            &render_bookshelf_breadcrumb_css(),
        )?;
        write_asset_file(
            &self.config_root.join(&breadcrumb_js_rel_path),
            &render_bookshelf_breadcrumb_js(breadcrumb_pages),
        )?;
        write_asset_file(
            &self.config_root.join(&search_js_rel_path),
            &render_bookshelf_search_js(searchindex_target),
        )?;

        append_output_asset(config, "output.html.additional-css", return_css_rel_path).context(
            "failed to append bookshelf return stylesheet to output.html.additional-css",
        )?;
        append_output_asset(
            config,
            "output.html.additional-css",
            breadcrumb_css_rel_path,
        )
        .context(
            "failed to append bookshelf breadcrumb stylesheet to output.html.additional-css",
        )?;
        append_output_asset(config, "output.html.additional-js", return_js_rel_path)
            .context("failed to append bookshelf return script to output.html.additional-js")?;
        append_output_asset(config, "output.html.additional-js", breadcrumb_js_rel_path)
            .context("failed to append bookshelf breadcrumb script to output.html.additional-js")?;
        append_output_asset(config, "output.html.additional-js", search_js_rel_path)
            .context("failed to append bookshelf search override to output.html.additional-js")?;

        Ok(())
    }

    pub fn stage_config_root_output_assets(
        &self,
        config: &mut Config,
        config_root: &Path,
    ) -> Result<()> {
        if let Some(theme) = config.get::<PathBuf>("output.html.theme")? {
            if !theme.is_absolute() {
                config.set("output.html.theme", config_root.join(theme))?;
            }
        }

        if self.config_root == config_root {
            return Ok(());
        }

        if let Some(input_404) = config.get::<String>("output.html.input-404")? {
            if !input_404.is_empty() && !Path::new(&input_404).is_absolute() {
                bail!(
                    "relative output.html.input-404 cannot be shared across books with different roots: {}",
                    input_404
                );
            }
        }

        stage_output_asset_list(config, "output.html.additional-css", config_root, self)
            .context("failed to stage config-root output.html.additional-css assets")?;
        stage_output_asset_list(config, "output.html.additional-js", config_root, self)
            .context("failed to stage config-root output.html.additional-js assets")?;

        Ok(())
    }

    pub fn cleanup(&mut self) -> Result<()> {
        if self.cleaned {
            return Ok(());
        }

        if let Err(err) = fs::remove_dir_all(&self.build_abs_root) {
            if err.kind() != std::io::ErrorKind::NotFound {
                return Err(err).with_context(|| {
                    format!(
                        "failed to remove transient bookshelf UI asset root {}",
                        self.build_abs_root.display()
                    )
                });
            }
        }

        let staged_config_assets_root = self.config_root.join(self.staged_config_rel_root());
        if let Err(err) = fs::remove_dir_all(&staged_config_assets_root) {
            if err.kind() != std::io::ErrorKind::NotFound {
                return Err(err).with_context(|| {
                    format!(
                        "failed to remove staged config-root asset directory {}",
                        staged_config_assets_root.display()
                    )
                });
            }
        }

        prune_empty_bookshelf_ui_dirs(&self.config_root.join(BOOKSHELF_UI_ASSET_DIR))
            .with_context(|| {
                format!(
                    "failed to prune empty bookshelf UI asset directories under {}",
                    self.config_root.display()
                )
            })?;

        self.cleaned = true;
        Ok(())
    }

    fn bookshelf_return_css_path(&self) -> PathBuf {
        self.build_rel_root
            .join(BOOKSHELF_UI_SHARED_DIR)
            .join(BOOKSHELF_RETURN_CSS_NAME)
    }

    fn bookshelf_breadcrumb_css_path(&self) -> PathBuf {
        self.build_rel_root
            .join(BOOKSHELF_UI_SHARED_DIR)
            .join(BOOKSHELF_BREADCRUMB_CSS_NAME)
    }

    fn bookshelf_return_js_path(&self, book_id: &str) -> PathBuf {
        self.build_rel_root
            .join(BOOKSHELF_UI_BOOKS_DIR)
            .join(book_id)
            .join(BOOKSHELF_RETURN_JS_NAME)
    }

    fn bookshelf_breadcrumb_js_path(&self, book_id: &str) -> PathBuf {
        self.build_rel_root
            .join(BOOKSHELF_UI_BOOKS_DIR)
            .join(book_id)
            .join(BOOKSHELF_BREADCRUMB_JS_NAME)
    }

    fn bookshelf_search_js_path(&self) -> PathBuf {
        self.build_rel_root
            .join(BOOKSHELF_UI_SHARED_DIR)
            .join(BOOKSHELF_SEARCH_JS_NAME)
    }

    fn staged_config_rel_root(&self) -> PathBuf {
        let suffix = self
            .build_rel_root
            .file_name()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("build"));
        PathBuf::from(BOOKSHELF_STAGED_CONFIG_ASSET_DIR).join(suffix)
    }
}

impl Drop for TransientBookshelfUiAssets {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

fn append_output_asset(config: &mut Config, key: &str, asset_path: PathBuf) -> Result<()> {
    let mut assets = config.get::<Vec<PathBuf>>(key)?.unwrap_or_default();
    if !assets.iter().any(|existing| existing == &asset_path) {
        assets.push(asset_path);
    }
    config.set(key, assets)?;
    Ok(())
}

fn stage_output_asset_list(
    config: &mut Config,
    key: &str,
    config_root: &Path,
    ui_assets: &TransientBookshelfUiAssets,
) -> Result<()> {
    let Some(assets) = config.get::<Vec<PathBuf>>(key)? else {
        return Ok(());
    };

    let staged_assets = assets
        .into_iter()
        .map(|asset_path| stage_output_asset(config_root, ui_assets, &asset_path))
        .collect::<Result<Vec<_>>>()?;
    config.set(key, staged_assets)?;

    Ok(())
}

fn stage_output_asset(
    config_root: &Path,
    ui_assets: &TransientBookshelfUiAssets,
    asset_path: &Path,
) -> Result<PathBuf> {
    let input_path = resolve_config_root_asset_path(config_root, asset_path)?;
    let stage_rel_path = ui_assets
        .staged_config_rel_root()
        .join(safe_stage_asset_path(config_root, asset_path)?);
    let output_path = ui_assets.config_root.join(&stage_rel_path);
    let parent = output_path.parent().with_context(|| {
        format!(
            "staged output asset path {} is missing a parent directory",
            output_path.display()
        )
    })?;

    fs::create_dir_all(parent).with_context(|| {
        format!(
            "failed to create staged asset directory {}",
            parent.display()
        )
    })?;
    fs::copy(&input_path, &output_path).with_context(|| {
        format!(
            "failed to stage output asset {} at {}",
            input_path.display(),
            output_path.display()
        )
    })?;

    Ok(stage_rel_path)
}

fn resolve_config_root_asset_path(config_root: &Path, asset_path: &Path) -> Result<PathBuf> {
    if asset_path.is_absolute() {
        asset_path.strip_prefix(config_root).with_context(|| {
            format!(
                "output asset path must stay under the shared config root {}: {}",
                config_root.display(),
                asset_path.display()
            )
        })?;
        Ok(asset_path.to_path_buf())
    } else {
        Ok(config_root.join(asset_path))
    }
}

fn safe_stage_asset_path(config_root: &Path, asset_path: &Path) -> Result<PathBuf> {
    let logical_path = if asset_path.is_absolute() {
        asset_path.strip_prefix(config_root).with_context(|| {
            format!(
                "output asset path must stay under the shared config root {}: {}",
                config_root.display(),
                asset_path.display()
            )
        })?
    } else {
        asset_path
    };
    let mut safe_path = PathBuf::new();

    for component in logical_path.components() {
        match component {
            std::path::Component::Normal(part) => safe_path.push(part),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir
            | std::path::Component::RootDir
            | std::path::Component::Prefix(_) => {
                bail!(
                    "output asset path must stay under the shared config root: {}",
                    asset_path.display()
                );
            }
        }
    }

    if safe_path.as_os_str().is_empty() {
        bail!(
            "output asset path must name a file: {}",
            asset_path.display()
        );
    }

    Ok(safe_path)
}

fn write_asset_file(path: &Path, contents: &str) -> Result<()> {
    let parent = path.parent().with_context(|| {
        format!(
            "asset path {} is missing a parent directory",
            path.display()
        )
    })?;
    fs::create_dir_all(parent)
        .with_context(|| format!("failed to create asset directory {}", parent.display()))?;
    fs::write(path, contents).with_context(|| format!("failed to write {}", path.display()))
}

fn prune_empty_dir_tree(path: &Path) -> std::io::Result<bool> {
    if !path_is_dir(path)? {
        return Ok(false);
    }

    let entries = match fs::read_dir(path) {
        Ok(entries) => entries.collect::<Result<Vec<_>, _>>()?,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(err) => return Err(err),
    };
    for entry in entries {
        let child = entry.path();
        if child.is_dir() {
            let _ = prune_empty_dir_tree(&child)?;
        }
    }

    let is_empty = match fs::read_dir(path) {
        Ok(mut entries) => entries.next().is_none(),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(true),
        Err(err) => return Err(err),
    };
    if is_empty {
        match fs::remove_dir(path) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(err),
        }
        return Ok(true);
    }

    Ok(false)
}

fn prune_empty_bookshelf_ui_dirs(path: &Path) -> std::io::Result<bool> {
    if !path_is_dir(path)? {
        return Ok(false);
    }

    let entries = match fs::read_dir(path) {
        Ok(entries) => entries.collect::<Result<Vec<_>, _>>()?,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(err) => return Err(err),
    };
    for entry in entries {
        let child = entry.path();
        if !child.is_dir() {
            continue;
        }

        let child_name = entry.file_name();
        if child_name.to_string_lossy().starts_with("build-") {
            // Sibling build roots may belong to concurrent builds. Leave them untouched.
            continue;
        }

        let _ = prune_empty_dir_tree(&child)?;
    }

    let is_empty = match fs::read_dir(path) {
        Ok(mut entries) => entries.next().is_none(),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(true),
        Err(err) => return Err(err),
    };
    if is_empty {
        match fs::remove_dir(path) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(err),
        }
        return Ok(true);
    }

    Ok(false)
}

fn path_is_dir(path: &Path) -> std::io::Result<bool> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(metadata.is_dir()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err),
    }
}

fn render_bookshelf_return_js(target: &str) -> String {
    format!(
        "(() => {{\n\
const bookshelfTarget = \"{target}\";\n\
const currentPage = window.location.pathname.split(\"/\").pop();\n\
if (currentPage === \"{root_bookshelf_html_path}\") {{\n\
    return;\n\
}}\n\
const buttonContainer = document.querySelector(\"#mdbook-menu-bar .right-buttons\");\n\
if (!buttonContainer || document.getElementById(\"{link_id}\")) {{\n\
    return;\n\
}}\n\
const rootPath = typeof path_to_root === \"string\" ? path_to_root : \"\";\n\
const link = document.createElement(\"a\");\n\
link.id = \"{link_id}\";\n\
link.className = \"{link_class}\";\n\
link.href = `${{rootPath}}${{bookshelfTarget}}`;\n\
link.rel = \"up\";\n\
link.title = \"Return to Bookshelf\";\n\
link.setAttribute(\"aria-label\", \"Return to Bookshelf\");\n\
link.textContent = \"Bookshelf\";\n\
buttonContainer.prepend(link);\n\
}})();\n",
        link_class = BOOKSHELF_RETURN_LINK_CLASS,
        link_id = BOOKSHELF_RETURN_LINK_ID,
        root_bookshelf_html_path = ROOT_BOOKSHELF_HTML_PATH,
        target = escape_js_string(target),
    )
}

fn render_bookshelf_return_css() -> String {
    format!(
        "#mdbook-menu-bar .right-buttons .{class_name} {{\n\
    align-items: center;\n\
    border: 1px solid currentColor;\n\
    border-radius: 999px;\n\
    color: var(--fg);\n\
    display: inline-flex;\n\
    font-size: 0.85em;\n\
    font-weight: 600;\n\
    line-height: 1;\n\
    margin-right: 0.75rem;\n\
    padding: 0.3rem 0.75rem;\n\
    text-decoration: none;\n\
    white-space: nowrap;\n\
}}\n\
#mdbook-menu-bar .right-buttons .{class_name}:hover,\n\
#mdbook-menu-bar .right-buttons .{class_name}:focus-visible {{\n\
    border-color: var(--links);\n\
    color: var(--links);\n\
}}\n",
        class_name = BOOKSHELF_RETURN_LINK_CLASS,
    )
}

fn render_bookshelf_breadcrumb_css() -> String {
    format!(
        "#mdbook-content main .{class_name} {{\n\
    color: var(--fg);\n\
    display: block;\n\
    font-size: 0.9em;\n\
    font-weight: 600;\n\
    letter-spacing: 0.01em;\n\
    margin: 0 0 1rem;\n\
    opacity: 0.72;\n\
}}\n",
        class_name = BOOKSHELF_BREADCRUMB_CLASS,
    )
}

fn render_bookshelf_breadcrumb_js(breadcrumb_pages: &[BookshelfBreadcrumbPage]) -> String {
    let breadcrumb_entries = if breadcrumb_pages.is_empty() {
        String::new()
    } else {
        breadcrumb_pages
            .iter()
            .map(|page| {
                format!(
                    "    \"{}\": \"{}\"",
                    escape_js_string(&page.html_path),
                    escape_js_string(&page.breadcrumb),
                )
            })
            .collect::<Vec<_>>()
            .join(",\n")
    };

    format!(
        "(() => {{\n\
const breadcrumbByPage = {{\n\
{entries}\n\
}};\n\
const main = document.querySelector(\"#mdbook-content main\");\n\
if (!main || document.getElementById(\"{breadcrumb_id}\")) {{\n\
    return;\n\
}}\n\
const rootPath = typeof path_to_root === \"string\" ? path_to_root : \"\";\n\
let currentPath = \"\";\n\
try {{\n\
    const currentUrl = new URL(window.location.href);\n\
    const bookRootUrl = new URL(rootPath === \"\" ? \"./\" : rootPath, currentUrl);\n\
    if (currentUrl.pathname.startsWith(bookRootUrl.pathname)) {{\n\
        currentPath = currentUrl.pathname.slice(bookRootUrl.pathname.length);\n\
    }} else {{\n\
        currentPath = currentUrl.pathname.split(\"/\").pop() || \"\";\n\
    }}\n\
}} catch (_err) {{\n\
    currentPath = window.location.pathname.split(\"/\").pop() || \"\";\n\
}}\n\
currentPath = currentPath.replace(/^\\/+/u, \"\");\n\
if (currentPath === \"\") {{\n\
    currentPath = \"index.html\";\n\
}}\n\
const breadcrumbText = breadcrumbByPage[currentPath];\n\
if (!breadcrumbText) {{\n\
    return;\n\
}}\n\
const breadcrumb = document.createElement(\"nav\");\n\
breadcrumb.id = \"{breadcrumb_id}\";\n\
breadcrumb.className = \"{breadcrumb_class}\";\n\
breadcrumb.setAttribute(\"aria-label\", \"Breadcrumb\");\n\
breadcrumb.setAttribute(\"{breadcrumb_attr}\", \"true\");\n\
breadcrumb.textContent = breadcrumbText;\n\
main.prepend(breadcrumb);\n\
}})();\n",
        entries = breadcrumb_entries,
        breadcrumb_attr = BOOKSHELF_BREADCRUMB_ATTRIBUTE,
        breadcrumb_class = BOOKSHELF_BREADCRUMB_CLASS,
        breadcrumb_id = BOOKSHELF_BREADCRUMB_ID,
    )
}

fn render_bookshelf_search_js(searchindex_target: &str) -> String {
    format!(
        "(() => {{\n\
const rootPath = typeof path_to_root === \"string\" ? path_to_root : \"\";\n\
window.path_to_searchindex_js = `${{rootPath}}{searchindex_target}`;\n\
}})();\n",
        searchindex_target = escape_js_string(searchindex_target),
    )
}

fn escape_js_string(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{2028}' => escaped.push_str("\\u2028"),
            '\u{2029}' => escaped.push_str("\\u2029"),
            control if control.is_control() => {
                escaped.push_str(&format!("\\u{:04x}", control as u32));
            }
            other => escaped.push(other),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_book_return_target_points_to_local_bookshelf_page() {
        let script = render_bookshelf_return_js("bookshelf.html");
        assert!(script.contains("const bookshelfTarget = \"bookshelf.html\";"));
    }

    #[test]
    fn non_root_book_return_target_points_back_to_root_bookshelf_page() {
        let script = render_bookshelf_return_js("../../../docs/bookshelf.html");
        assert!(script.contains("const bookshelfTarget = \"../../../docs/bookshelf.html\";"));
    }

    #[test]
    fn inject_bookshelf_ui_assets_preserves_existing_output_assets() {
        let temp_root = make_temp_dir("mdbook-bookshelf-bookshelf-ui");
        let mut ui_assets =
            TransientBookshelfUiAssets::new(&temp_root).expect("asset root should be created");
        let mut config = Config::default();
        config
            .set(
                "output.html.additional-css",
                vec![PathBuf::from("shared/site.css")],
            )
            .expect("existing css should be configured");
        config
            .set(
                "output.html.additional-js",
                vec![PathBuf::from("shared/site.js")],
            )
            .expect("existing js should be configured");

        ui_assets
            .inject_bookshelf_ui_assets(
                &mut config,
                "modules/parser/docs",
                "../../../docs/bookshelf.html",
                "bookshelf-searchindex.js",
                &sample_breadcrumb_pages(),
            )
            .expect("asset injection should succeed");

        let css_assets = config
            .get::<Vec<PathBuf>>("output.html.additional-css")
            .expect("css assets should deserialize")
            .expect("css assets should exist");
        assert_eq!(css_assets[0], PathBuf::from("shared/site.css"));
        assert!(css_assets[1].starts_with(Path::new(BOOKSHELF_UI_ASSET_DIR)));
        assert!(css_assets[1].ends_with(Path::new(BOOKSHELF_RETURN_CSS_NAME)));
        assert!(css_assets[2].starts_with(Path::new(BOOKSHELF_UI_ASSET_DIR)));
        assert!(css_assets[2].ends_with(Path::new(BOOKSHELF_BREADCRUMB_CSS_NAME)));
        assert!(temp_root.join(&css_assets[1]).exists());
        assert!(temp_root.join(&css_assets[2]).exists());

        let js_assets = config
            .get::<Vec<PathBuf>>("output.html.additional-js")
            .expect("js assets should deserialize")
            .expect("js assets should exist");
        assert_eq!(js_assets[0], PathBuf::from("shared/site.js"));
        assert!(js_assets[1].starts_with(Path::new(BOOKSHELF_UI_ASSET_DIR)));
        assert!(
            js_assets[1].ends_with(Path::new("modules/parser/docs").join(BOOKSHELF_RETURN_JS_NAME))
        );
        assert!(js_assets[2].starts_with(Path::new(BOOKSHELF_UI_ASSET_DIR)));
        assert!(js_assets[2]
            .ends_with(Path::new("modules/parser/docs").join(BOOKSHELF_BREADCRUMB_JS_NAME)));
        assert!(js_assets[3].starts_with(Path::new(BOOKSHELF_UI_ASSET_DIR)));
        assert!(js_assets[3].ends_with(Path::new(BOOKSHELF_SEARCH_JS_NAME)));
        assert!(temp_root.join(&js_assets[1]).exists());
        assert!(temp_root.join(&js_assets[2]).exists());
        assert!(temp_root.join(&js_assets[3]).exists());
        let breadcrumb_js = fs::read_to_string(temp_root.join(&js_assets[2]))
            .expect("breadcrumb script should be readable");
        assert!(breadcrumb_js.contains("\"grammar.html\": \"Example Parser / Grammar\""));
        assert!(breadcrumb_js.contains("setAttribute(\"data-bookshelf-breadcrumb\", \"true\")"));
        let search_js = fs::read_to_string(temp_root.join(&js_assets[3]))
            .expect("search override should be readable");
        assert!(search_js.contains("window.path_to_searchindex_js"));
        assert!(search_js.contains("bookshelf-searchindex.js"));

        ui_assets.cleanup().expect("asset root should be removed");
        assert!(!temp_root.join(BOOKSHELF_UI_ASSET_DIR).exists());
        fs::remove_dir_all(temp_root).expect("temporary asset directory should be removed");
    }

    #[test]
    fn render_bookshelf_breadcrumb_js_uses_exact_page_map() {
        let script = render_bookshelf_breadcrumb_js(&sample_breadcrumb_pages());

        assert!(script.contains("\"grammar.html\": \"Example Parser / Grammar\""));
        assert!(script.contains("\"runtime.html\": \"Example Parser / Runtime\""));
        assert!(script.contains("breadcrumbByPage[currentPath]"));
        assert!(script.contains("main.prepend(breadcrumb)"));
        assert!(script.contains("currentPath = \"index.html\""));
    }

    #[test]
    fn render_bookshelf_search_js_points_to_site_root_shared_index() {
        let script = render_bookshelf_search_js("bookshelf-searchindex.js");

        assert!(script.contains(
            "const rootPath = typeof path_to_root === \"string\" ? path_to_root : \"\";"
        ));
        assert!(script
            .contains("window.path_to_searchindex_js = `${rootPath}bookshelf-searchindex.js`;"));
    }

    #[test]
    fn cleanup_removes_transient_asset_root_on_drop() {
        let temp_root = make_temp_dir("mdbook-bookshelf-bookshelf-ui-drop");

        {
            let ui_assets =
                TransientBookshelfUiAssets::new(&temp_root).expect("asset root should be created");
            let mut config = Config::default();
            ui_assets
                .inject_bookshelf_ui_assets(
                    &mut config,
                    "docs",
                    "bookshelf.html",
                    "bookshelf-searchindex.js",
                    &sample_breadcrumb_pages(),
                )
                .expect("asset injection should succeed");
            assert!(temp_root.join(BOOKSHELF_UI_ASSET_DIR).exists());
        }

        assert!(!temp_root.join(BOOKSHELF_UI_ASSET_DIR).exists());
        fs::remove_dir_all(temp_root).expect("temporary asset directory should be removed");
    }

    #[test]
    fn cleanup_prunes_preexisting_empty_bookshelf_ui_dirs() {
        let temp_root = make_temp_dir("mdbook-bookshelf-bookshelf-ui-prune");
        fs::create_dir_all(
            temp_root
                .join(BOOKSHELF_UI_ASSET_DIR)
                .join(BOOKSHELF_UI_SHARED_DIR),
        )
        .expect("legacy shared dir should be created");
        fs::create_dir_all(
            temp_root
                .join(BOOKSHELF_UI_ASSET_DIR)
                .join(BOOKSHELF_UI_BOOKS_DIR)
                .join("docs"),
        )
        .expect("legacy book dir should be created");

        let mut ui_assets =
            TransientBookshelfUiAssets::new(&temp_root).expect("asset root should be created");
        let mut config = Config::default();
        ui_assets
            .inject_bookshelf_ui_assets(
                &mut config,
                "docs",
                "bookshelf.html",
                "bookshelf-searchindex.js",
                &sample_breadcrumb_pages(),
            )
            .expect("asset injection should succeed");

        ui_assets.cleanup().expect("cleanup should succeed");

        assert!(!temp_root.join(BOOKSHELF_UI_ASSET_DIR).exists());
        fs::remove_dir_all(temp_root).expect("temporary asset directory should be removed");
    }

    #[test]
    fn cleanup_preserves_sibling_build_roots_while_pruning_legacy_empty_dirs() {
        let temp_root = make_temp_dir("mdbook-bookshelf-bookshelf-ui-sibling");
        let legacy_shared_dir = temp_root
            .join(BOOKSHELF_UI_ASSET_DIR)
            .join(BOOKSHELF_UI_SHARED_DIR);
        let legacy_book_dir = temp_root
            .join(BOOKSHELF_UI_ASSET_DIR)
            .join(BOOKSHELF_UI_BOOKS_DIR)
            .join("docs");
        let sibling_build_dir = temp_root
            .join(BOOKSHELF_UI_ASSET_DIR)
            .join("build-sibling")
            .join(BOOKSHELF_UI_SHARED_DIR);

        fs::create_dir_all(&legacy_shared_dir).expect("legacy shared dir should be created");
        fs::create_dir_all(&legacy_book_dir).expect("legacy book dir should be created");
        fs::create_dir_all(&sibling_build_dir).expect("sibling build dir should be created");
        fs::write(sibling_build_dir.join("keep.js"), "console.log('keep');")
            .expect("sibling build marker should be written");

        let mut ui_assets =
            TransientBookshelfUiAssets::new(&temp_root).expect("asset root should be created");
        let mut config = Config::default();
        ui_assets
            .inject_bookshelf_ui_assets(
                &mut config,
                "docs",
                "bookshelf.html",
                "bookshelf-searchindex.js",
                &sample_breadcrumb_pages(),
            )
            .expect("asset injection should succeed");

        ui_assets.cleanup().expect("cleanup should succeed");

        assert!(!legacy_shared_dir.exists());
        assert!(!temp_root
            .join(BOOKSHELF_UI_ASSET_DIR)
            .join(BOOKSHELF_UI_BOOKS_DIR)
            .exists());
        assert!(sibling_build_dir.exists());
        assert!(temp_root.join(BOOKSHELF_UI_ASSET_DIR).exists());

        fs::remove_dir_all(temp_root).expect("temporary asset directory should be removed");
    }

    #[test]
    fn cleanup_tolerates_racing_bookshelf_ui_root_removal() {
        let temp_root = make_temp_dir("mdbook-bookshelf-bookshelf-ui-racing-root");

        let mut ui_assets =
            TransientBookshelfUiAssets::new(&temp_root).expect("asset root should be created");
        let mut config = Config::default();
        ui_assets
            .inject_bookshelf_ui_assets(
                &mut config,
                "docs",
                "bookshelf.html",
                "bookshelf-searchindex.js",
                &sample_breadcrumb_pages(),
            )
            .expect("asset injection should succeed");

        fs::remove_dir_all(temp_root.join(BOOKSHELF_UI_ASSET_DIR))
            .expect("simulated sibling cleanup should remove the ui asset root");

        ui_assets
            .cleanup()
            .expect("cleanup should tolerate a missing ui asset root");

        assert!(!temp_root.join(BOOKSHELF_UI_ASSET_DIR).exists());
        fs::remove_dir_all(temp_root).expect("temporary asset directory should be removed");
    }

    fn sample_breadcrumb_pages() -> Vec<BookshelfBreadcrumbPage> {
        vec![
            BookshelfBreadcrumbPage {
                html_path: "grammar.html".to_string(),
                breadcrumb: "Example Parser / Grammar".to_string(),
            },
            BookshelfBreadcrumbPage {
                html_path: "runtime.html".to_string(),
                breadcrumb: "Example Parser / Runtime".to_string(),
            },
        ]
    }

    fn make_temp_dir(tag: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        let dir = std::env::temp_dir()
            .join("mdbook-bookshelf")
            .join(format!("{tag}-{nanos}"));
        fs::create_dir_all(&dir).expect("temp output directory should be created");
        dir
    }
}
