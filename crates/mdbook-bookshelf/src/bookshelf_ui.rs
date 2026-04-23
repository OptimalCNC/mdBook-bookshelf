use crate::root_bookshelf_preprocessor::ROOT_BOOKSHELF_HTML_PATH;
use crate::search::SHARED_SEARCH_INDEX_NAME;
use anyhow::{bail, Context, Result};
use mdbook_driver::config::Config;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

const BOOKSHELF_UI_ASSET_DIR: &str = ".mdbook-bookshelf";
const BOOKSHELF_UI_SHARED_DIR: &str = "shared";
const BOOKSHELF_UI_BOOKS_DIR: &str = "books";
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
        book_id: &str,
        root_book_id: &str,
        breadcrumb_pages: &[BookshelfBreadcrumbPage],
    ) -> Result<()> {
        let return_css_rel_path = self.bookshelf_return_css_path();
        let return_js_rel_path = self.bookshelf_return_js_path(book_id);
        let breadcrumb_css_rel_path = self.bookshelf_breadcrumb_css_path();
        let breadcrumb_js_rel_path = self.bookshelf_breadcrumb_js_path(book_id);
        let search_js_rel_path = self.bookshelf_search_js_path();

        write_asset_file(
            &self.config_root.join(&return_css_rel_path),
            &render_bookshelf_return_css(),
        )?;
        write_asset_file(
            &self.config_root.join(&return_js_rel_path),
            &render_bookshelf_return_js(book_id, root_book_id),
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
            &render_bookshelf_search_js(),
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

    pub fn cleanup(&mut self) -> Result<()> {
        if self.cleaned {
            return Ok(());
        }

        if self.build_abs_root.exists() {
            fs::remove_dir_all(&self.build_abs_root).with_context(|| {
                format!(
                    "failed to remove transient bookshelf UI asset root {}",
                    self.build_abs_root.display()
                )
            })?;
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
    if !path.is_dir() {
        return Ok(false);
    }

    let entries = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
    for entry in entries {
        let child = entry.path();
        if child.is_dir() {
            let _ = prune_empty_dir_tree(&child)?;
        }
    }

    if fs::read_dir(path)?.next().is_none() {
        fs::remove_dir(path)?;
        return Ok(true);
    }

    Ok(false)
}

fn prune_empty_bookshelf_ui_dirs(path: &Path) -> std::io::Result<bool> {
    if !path.is_dir() {
        return Ok(false);
    }

    let entries = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
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

    if fs::read_dir(path)?.next().is_none() {
        fs::remove_dir(path)?;
        return Ok(true);
    }

    Ok(false)
}

fn render_bookshelf_return_js(book_id: &str, root_book_id: &str) -> String {
    let target = bookshelf_return_target_from_book_root(book_id, root_book_id);

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
        target = target,
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

fn render_bookshelf_search_js() -> String {
    format!(
        "(() => {{\n\
const rootPath = typeof path_to_root === \"string\" ? path_to_root : \"\";\n\
window.path_to_searchindex_js = `${{rootPath}}../../{shared_search_index_name}`;\n\
}})();\n",
        shared_search_index_name = SHARED_SEARCH_INDEX_NAME,
    )
}

fn bookshelf_return_target_from_book_root(book_id: &str, root_book_id: &str) -> String {
    if book_id == root_book_id {
        ROOT_BOOKSHELF_HTML_PATH.to_string()
    } else {
        format!("../{root_book_id}/{ROOT_BOOKSHELF_HTML_PATH}")
    }
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
        assert_eq!(
            bookshelf_return_target_from_book_root("meta", "meta"),
            "bookshelf.html"
        );
    }

    #[test]
    fn non_root_book_return_target_points_back_to_root_bookshelf_page() {
        assert_eq!(
            bookshelf_return_target_from_book_root("parser", "meta"),
            "../meta/bookshelf.html"
        );
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
            .inject_bookshelf_ui_assets(&mut config, "parser", "meta", &sample_breadcrumb_pages())
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
        assert!(js_assets[1].ends_with(Path::new("parser").join(BOOKSHELF_RETURN_JS_NAME)));
        assert!(js_assets[2].starts_with(Path::new(BOOKSHELF_UI_ASSET_DIR)));
        assert!(js_assets[2].ends_with(Path::new("parser").join(BOOKSHELF_BREADCRUMB_JS_NAME)));
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
        assert!(search_js.contains("../../searchindex.js"));

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
        let script = render_bookshelf_search_js();

        assert!(script.contains(
            "const rootPath = typeof path_to_root === \"string\" ? path_to_root : \"\";"
        ));
        assert!(
            script.contains("window.path_to_searchindex_js = `${rootPath}../../searchindex.js`;")
        );
    }

    #[test]
    fn cleanup_removes_transient_asset_root_on_drop() {
        let temp_root = make_temp_dir("mdbook-bookshelf-bookshelf-ui-drop");

        {
            let ui_assets =
                TransientBookshelfUiAssets::new(&temp_root).expect("asset root should be created");
            let mut config = Config::default();
            ui_assets
                .inject_bookshelf_ui_assets(&mut config, "meta", "meta", &sample_breadcrumb_pages())
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
                .join("meta"),
        )
        .expect("legacy book dir should be created");

        let mut ui_assets =
            TransientBookshelfUiAssets::new(&temp_root).expect("asset root should be created");
        let mut config = Config::default();
        ui_assets
            .inject_bookshelf_ui_assets(&mut config, "meta", "meta", &sample_breadcrumb_pages())
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
            .join("meta");
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
            .inject_bookshelf_ui_assets(&mut config, "meta", "meta", &sample_breadcrumb_pages())
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
