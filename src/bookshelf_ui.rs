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
const BOOKSHELF_RETURN_CSS_NAME: &str = "bookshelf-return.css";
const BOOKSHELF_RETURN_JS_NAME: &str = "bookshelf-return.js";
const BOOKSHELF_RETURN_LINK_CLASS: &str = "bookshelf-return-link";
const BOOKSHELF_RETURN_LINK_ID: &str = "bookshelf-return-link";

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

    pub fn inject_bookshelf_return_assets(
        &self,
        config: &mut Config,
        book_id: &str,
        root_book_id: &str,
    ) -> Result<()> {
        let css_rel_path = self.bookshelf_return_css_path();
        let js_rel_path = self.bookshelf_return_js_path(book_id);
        let css_abs_path = self.config_root.join(&css_rel_path);
        let js_abs_path = self.config_root.join(&js_rel_path);

        write_asset_file(&css_abs_path, &render_bookshelf_return_css())?;
        write_asset_file(
            &js_abs_path,
            &render_bookshelf_return_js(book_id, root_book_id),
        )?;

        append_output_asset(config, "output.html.additional-css", css_rel_path).context(
            "failed to append bookshelf return stylesheet to output.html.additional-css",
        )?;
        append_output_asset(config, "output.html.additional-js", js_rel_path)
            .context("failed to append bookshelf return script to output.html.additional-js")?;

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

        prune_empty_dir_tree(&self.config_root.join(BOOKSHELF_UI_ASSET_DIR)).with_context(
            || {
                format!(
                    "failed to prune empty bookshelf UI asset directories under {}",
                    self.config_root.display()
                )
            },
        )?;

        self.cleaned = true;
        Ok(())
    }

    fn bookshelf_return_css_path(&self) -> PathBuf {
        self.build_rel_root
            .join(BOOKSHELF_UI_SHARED_DIR)
            .join(BOOKSHELF_RETURN_CSS_NAME)
    }

    fn bookshelf_return_js_path(&self, book_id: &str) -> PathBuf {
        self.build_rel_root
            .join(BOOKSHELF_UI_BOOKS_DIR)
            .join(book_id)
            .join(BOOKSHELF_RETURN_JS_NAME)
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

fn bookshelf_return_target_from_book_root(book_id: &str, root_book_id: &str) -> String {
    if book_id == root_book_id {
        ROOT_BOOKSHELF_HTML_PATH.to_string()
    } else {
        format!("../{root_book_id}/{ROOT_BOOKSHELF_HTML_PATH}")
    }
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
    fn inject_bookshelf_return_assets_preserves_existing_output_assets() {
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
            .inject_bookshelf_return_assets(&mut config, "parser", "meta")
            .expect("asset injection should succeed");

        let css_assets = config
            .get::<Vec<PathBuf>>("output.html.additional-css")
            .expect("css assets should deserialize")
            .expect("css assets should exist");
        assert_eq!(css_assets[0], PathBuf::from("shared/site.css"));
        assert!(css_assets[1].starts_with(Path::new(BOOKSHELF_UI_ASSET_DIR)));
        assert!(css_assets[1].ends_with(Path::new(BOOKSHELF_RETURN_CSS_NAME)));
        assert!(temp_root.join(&css_assets[1]).exists());

        let js_assets = config
            .get::<Vec<PathBuf>>("output.html.additional-js")
            .expect("js assets should deserialize")
            .expect("js assets should exist");
        assert_eq!(js_assets[0], PathBuf::from("shared/site.js"));
        assert!(js_assets[1].starts_with(Path::new(BOOKSHELF_UI_ASSET_DIR)));
        assert!(js_assets[1].ends_with(Path::new("parser").join(BOOKSHELF_RETURN_JS_NAME)));
        assert!(temp_root.join(&js_assets[1]).exists());

        ui_assets.cleanup().expect("asset root should be removed");
        assert!(!temp_root.join(BOOKSHELF_UI_ASSET_DIR).exists());
        fs::remove_dir_all(temp_root).expect("temporary asset directory should be removed");
    }

    #[test]
    fn cleanup_removes_transient_asset_root_on_drop() {
        let temp_root = make_temp_dir("mdbook-bookshelf-bookshelf-ui-drop");

        {
            let ui_assets =
                TransientBookshelfUiAssets::new(&temp_root).expect("asset root should be created");
            let mut config = Config::default();
            ui_assets
                .inject_bookshelf_return_assets(&mut config, "meta", "meta")
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
            .inject_bookshelf_return_assets(&mut config, "meta", "meta")
            .expect("asset injection should succeed");

        ui_assets.cleanup().expect("cleanup should succeed");

        assert!(!temp_root.join(BOOKSHELF_UI_ASSET_DIR).exists());
        fs::remove_dir_all(temp_root).expect("temporary asset directory should be removed");
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
