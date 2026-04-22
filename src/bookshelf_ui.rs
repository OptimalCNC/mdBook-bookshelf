use crate::root_bookshelf_preprocessor::ROOT_BOOKSHELF_HTML_PATH;
use anyhow::{Context, Result};
use mdbook_driver::config::Config;
use std::fs;
use std::path::{Path, PathBuf};

const BOOKSHELF_UI_ASSET_DIR: &str = ".mdbook-bookshelf";
const BOOKSHELF_UI_SHARED_DIR: &str = "shared";
const BOOKSHELF_UI_BOOKS_DIR: &str = "books";
const BOOKSHELF_RETURN_CSS_NAME: &str = "bookshelf-return.css";
const BOOKSHELF_RETURN_JS_NAME: &str = "bookshelf-return.js";
const BOOKSHELF_RETURN_LINK_CLASS: &str = "bookshelf-return-link";
const BOOKSHELF_RETURN_LINK_ID: &str = "bookshelf-return-link";

pub fn inject_bookshelf_return_assets(
    config: &mut Config,
    config_root: &Path,
    book_id: &str,
    root_book_id: &str,
) -> Result<()> {
    let css_rel_path = bookshelf_return_css_path();
    let js_rel_path = bookshelf_return_js_path(book_id);
    let js_abs_path = config_root.join(&js_rel_path);
    let css_abs_path = config_root.join(&css_rel_path);

    write_asset_file(&css_abs_path, &render_bookshelf_return_css())?;
    write_asset_file(
        &js_abs_path,
        &render_bookshelf_return_js(book_id, root_book_id),
    )?;

    append_output_asset(config, "output.html.additional-css", css_rel_path)
        .context("failed to append bookshelf return stylesheet to output.html.additional-css")?;
    append_output_asset(config, "output.html.additional-js", js_rel_path)
        .context("failed to append bookshelf return script to output.html.additional-js")?;

    Ok(())
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

fn bookshelf_return_css_path() -> PathBuf {
    PathBuf::from(BOOKSHELF_UI_ASSET_DIR)
        .join(BOOKSHELF_UI_SHARED_DIR)
        .join(BOOKSHELF_RETURN_CSS_NAME)
}

fn bookshelf_return_js_path(book_id: &str) -> PathBuf {
    PathBuf::from(BOOKSHELF_UI_ASSET_DIR)
        .join(BOOKSHELF_UI_BOOKS_DIR)
        .join(book_id)
        .join(BOOKSHELF_RETURN_JS_NAME)
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

        let temp_root = std::env::temp_dir().join("mdbook-bookshelf-bookshelf-ui");
        inject_bookshelf_return_assets(&mut config, &temp_root, "parser", "meta")
            .expect("asset injection should succeed");

        let css_assets = config
            .get::<Vec<PathBuf>>("output.html.additional-css")
            .expect("css assets should deserialize")
            .expect("css assets should exist");
        assert_eq!(
            css_assets,
            vec![
                PathBuf::from("shared/site.css"),
                bookshelf_return_css_path(),
            ]
        );

        let js_assets = config
            .get::<Vec<PathBuf>>("output.html.additional-js")
            .expect("js assets should deserialize")
            .expect("js assets should exist");
        assert_eq!(
            js_assets,
            vec![
                PathBuf::from("shared/site.js"),
                bookshelf_return_js_path("parser"),
            ]
        );

        fs::remove_dir_all(temp_root).expect("temporary asset directory should be removed");
    }
}
