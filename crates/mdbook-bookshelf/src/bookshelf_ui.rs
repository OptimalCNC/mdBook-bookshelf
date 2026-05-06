use crate::route_paths::{path_to_string, relative_path};
use anyhow::{Context, Result};
use mdbook_driver::config::Config;
use mdbook_preprocessor::book::{Book, Chapter};
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

const PREPROCESSOR_NAME: &str = "bookshelf-page-metadata";
const BOOKSHELF_RETURN_CSS_NAME: &str = "bookshelf-return.css";
const BOOKSHELF_RETURN_JS_NAME: &str = "bookshelf-return.js";
const BOOKSHELF_RETURN_LINK_CLASS: &str = "bookshelf-return-link";
const BOOKSHELF_RETURN_LINK_ID: &str = "bookshelf-return-link";
const BOOKSHELF_SEARCH_JS_NAME: &str = "bookshelf-search.js";
const BOOKSHELF_PAGE_METADATA_ID: &str = "mdbook-bookshelf-page-metadata";
const BOOKSHELF_ASSET_GITIGNORE_NAME: &str = ".gitignore";
const BOOKSHELF_ASSET_GITIGNORE_CONTENTS: &str = "*\n";
const BOOKSHELF_ASSET_README_NAME: &str = "README.md";
const BOOKSHELF_ASSET_README_CONTENTS: &str = "\
# mdbook-bookshelf Generated Assets

This directory is managed by mdbook-bookshelf.

Files here are generated runtime source assets passed to mdBook through
output.html.additional-css and output.html.additional-js. They can be
rewritten on any build.

Do not edit these files or place project-owned assets here. Put your own assets
outside this directory and configure them with stock mdBook settings.
";

pub struct BookshelfAssets {
    config_root: PathBuf,
    asset_dir: PathBuf,
}

impl BookshelfAssets {
    pub fn new(config_root: &Path, asset_dir: &Path) -> Self {
        Self {
            config_root: config_root.to_path_buf(),
            asset_dir: asset_dir.to_path_buf(),
        }
    }

    pub fn inject_bookshelf_runtime_assets(&self, config: &mut Config) -> Result<()> {
        let return_css_rel_path = self.asset_dir.join(BOOKSHELF_RETURN_CSS_NAME);
        let return_js_rel_path = self.asset_dir.join(BOOKSHELF_RETURN_JS_NAME);
        let search_js_rel_path = self.asset_dir.join(BOOKSHELF_SEARCH_JS_NAME);
        let gitignore_rel_path = self.asset_dir.join(BOOKSHELF_ASSET_GITIGNORE_NAME);
        let readme_rel_path = self.asset_dir.join(BOOKSHELF_ASSET_README_NAME);

        write_asset_file(
            &self.config_root.join(&gitignore_rel_path),
            BOOKSHELF_ASSET_GITIGNORE_CONTENTS,
        )?;
        write_asset_file(
            &self.config_root.join(&readme_rel_path),
            BOOKSHELF_ASSET_README_CONTENTS,
        )?;
        write_asset_file(
            &self.config_root.join(&return_css_rel_path),
            &render_bookshelf_return_css(),
        )?;
        write_asset_file(
            &self.config_root.join(&return_js_rel_path),
            &render_bookshelf_return_js(),
        )?;
        write_asset_file(
            &self.config_root.join(&search_js_rel_path),
            &render_bookshelf_search_js(),
        )?;

        append_output_asset(config, "output.html.additional-css", return_css_rel_path).context(
            "failed to append bookshelf return stylesheet to output.html.additional-css",
        )?;
        append_output_asset(config, "output.html.additional-js", return_js_rel_path)
            .context("failed to append bookshelf return script to output.html.additional-js")?;
        append_output_asset(config, "output.html.additional-js", search_js_rel_path)
            .context("failed to append bookshelf search override to output.html.additional-js")?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct BookshelfPageMetadataPreprocessor {
    book_output_rel: PathBuf,
    root_bookshelf_rel: PathBuf,
    search_index_rel: PathBuf,
}

impl BookshelfPageMetadataPreprocessor {
    pub(crate) fn new(
        book_output_rel: PathBuf,
        root_bookshelf_rel: PathBuf,
        search_index_rel: PathBuf,
    ) -> Self {
        Self {
            book_output_rel,
            root_bookshelf_rel,
            search_index_rel,
        }
    }

    fn inject_chapter_metadata(&self, chapter: &mut Chapter) -> Result<()> {
        if chapter.content.contains(BOOKSHELF_PAGE_METADATA_ID) {
            return Ok(());
        }

        let Some(chapter_path) = chapter.path.as_deref() else {
            return Ok(());
        };

        let chapter_output_path = self
            .book_output_rel
            .join(chapter_path.with_extension("html"));
        let chapter_output_dir = chapter_output_path
            .parent()
            .unwrap_or_else(|| Path::new(""));
        let bookshelf_target = Some(path_to_string(&relative_path(
            chapter_output_dir,
            &self.root_bookshelf_rel,
        )));
        let search_index_target =
            path_to_string(&relative_path(chapter_output_dir, &self.search_index_rel));
        let metadata = PageMetadata {
            bookshelf_target,
            search_index_target,
        };
        let json = serde_json::to_string(&metadata).context("failed to serialize page metadata")?;
        let prefix = format!(
            "<script type=\"application/json\" id=\"{BOOKSHELF_PAGE_METADATA_ID}\">{json}</script>\n\n"
        );
        chapter.content.insert_str(0, &prefix);
        Ok(())
    }
}

impl Preprocessor for BookshelfPageMetadataPreprocessor {
    fn name(&self) -> &str {
        PREPROCESSOR_NAME
    }

    fn run(
        &self,
        _ctx: &PreprocessorContext,
        mut book: Book,
    ) -> mdbook_preprocessor::errors::Result<Book> {
        let mut error = None;
        book.for_each_chapter_mut(|chapter| {
            if error.is_some() {
                return;
            }

            if let Err(err) = self.inject_chapter_metadata(chapter).with_context(|| {
                format!(
                    "chapter '{}' failed to inject bookshelf metadata",
                    chapter.name
                )
            }) {
                error = Some(err);
            }
        });

        if let Some(error) = error {
            return Err(error);
        }

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> mdbook_preprocessor::errors::Result<bool> {
        Ok(renderer == "html")
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PageMetadata {
    bookshelf_target: Option<String>,
    search_index_target: String,
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

fn render_bookshelf_return_js() -> String {
    r##"(() => {
const metadata = readBookshelfPageMetadata();
const bookshelfTarget = typeof metadata.bookshelfTarget === "string" ? metadata.bookshelfTarget : "";
if (!bookshelfTarget) {
    return;
}
const buttonContainer = document.querySelector("#mdbook-menu-bar .right-buttons");
if (!buttonContainer || document.getElementById("__LINK_ID__")) {
    return;
}
const svgNamespace = "http://www.w3.org/2000/svg";
const link = document.createElement("a");
link.id = "__LINK_ID__";
link.className = "__LINK_CLASS__";
link.href = bookshelfTarget;
link.rel = "up";
link.title = "Return to Bookshelf";
link.setAttribute("aria-label", "Return to Bookshelf");

const icon = document.createElementNS(svgNamespace, "svg");
icon.setAttribute("class", "bookshelf-return-icon");
icon.setAttribute("viewBox", "0 0 24 24");
icon.setAttribute("fill", "none");
icon.setAttribute("stroke", "currentColor");
icon.setAttribute("stroke-width", "2");
icon.setAttribute("stroke-linecap", "round");
icon.setAttribute("stroke-linejoin", "round");
icon.setAttribute("aria-hidden", "true");
icon.setAttribute("focusable", "false");

const shapes = [
    ["rect", { x: "3", y: "4", width: "3.5", height: "14", rx: "0.5" }],
    ["rect", { x: "8.5", y: "4", width: "3.5", height: "14", rx: "0.5" }],
    ["rect", { x: "14", y: "4", width: "3.5", height: "14", rx: "0.5" }],
    ["line", { x1: "2", y1: "20", x2: "22", y2: "20" }],
];
for (const [tag, attrs] of shapes) {
    const node = document.createElementNS(svgNamespace, tag);
    for (const name in attrs) {
        node.setAttribute(name, attrs[name]);
    }
    icon.appendChild(node);
}

const label = document.createElement("span");
label.className = "bookshelf-return-label";
label.textContent = "Bookshelf";

link.appendChild(icon);
link.appendChild(label);
buttonContainer.prepend(link);

function readBookshelfPageMetadata() {
    const node = document.getElementById("__METADATA_ID__");
    if (!node) {
        return {};
    }
    try {
        return JSON.parse(node.textContent || "{}");
    } catch (_err) {
        return {};
    }
}
})();
"##
    .replace("__LINK_CLASS__", BOOKSHELF_RETURN_LINK_CLASS)
    .replace("__LINK_ID__", BOOKSHELF_RETURN_LINK_ID)
    .replace("__METADATA_ID__", BOOKSHELF_PAGE_METADATA_ID)
}

fn render_bookshelf_return_css() -> String {
    r#".bookshelf-list {
    display: grid;
    gap: 1rem;
    grid-template-columns: repeat(auto-fill, minmax(22rem, 1fr));
    list-style: none;
    margin: 1.5rem 0 0;
    padding: 0;
}

.bookshelf-list > li {
    margin: 0;
    padding: 0;
}

.bookshelf-book {
    background: var(--quote-bg, var(--sidebar-bg, var(--bg)));
    border: 1px solid var(--table-border-color);
    border-inline-start: 3px solid var(--links);
    border-radius: 3px;
    color: var(--fg);
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    height: 100%;
    padding: 0.85rem 1rem 0.9rem 1.1rem;
    position: relative;
    text-decoration: none;
    transition: border-inline-start-width 120ms ease, box-shadow 120ms ease, transform 120ms ease;
}

.content .bookshelf-book:link,
.content .bookshelf-book:visited {
    color: var(--fg);
}

.bookshelf-book:hover,
.bookshelf-book:focus-visible {
    border-inline-start-width: 5px;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.12);
    text-decoration: none;
    transform: translateY(-1px);
}

.bookshelf-book:focus-visible {
    outline: 2px solid var(--links);
    outline-offset: 2px;
}

.bookshelf-book-title {
    color: var(--links);
    font-size: 1.05em;
    font-weight: 600;
    line-height: 1.3;
}

.bookshelf-book-description {
    color: var(--fg);
    font-size: 0.95em;
    line-height: 1.4;
    opacity: 0.78;
}

@media (max-width: 480px) {
    .bookshelf-list {
        grid-template-columns: 1fr;
    }
}

@media (prefers-reduced-motion: reduce) {
    .bookshelf-book {
        transition: none;
    }

    .bookshelf-book:hover,
    .bookshelf-book:focus-visible {
        transform: none;
    }
}

#mdbook-menu-bar .right-buttons .__LINK_CLASS__ {
    align-items: center;
    background: transparent;
    border: 0;
    border-radius: 4px;
    color: var(--icons);
    display: inline-flex;
    gap: 0.4rem;
    height: var(--menu-bar-height);
    line-height: 1;
    margin-right: 0.25rem;
    padding: 0 0.5rem;
    text-decoration: none;
    transition: background-color 150ms ease, color 150ms ease;
    white-space: nowrap;
}

#mdbook-menu-bar .right-buttons .__LINK_CLASS__:hover,
#mdbook-menu-bar .right-buttons .__LINK_CLASS__:focus-visible {
    color: var(--icons-hover);
    text-decoration: none;
}

#mdbook-menu-bar .right-buttons .__LINK_CLASS__:focus-visible {
    outline: 2px solid var(--icons-hover);
    outline-offset: -2px;
}

#mdbook-menu-bar .right-buttons .bookshelf-return-icon {
    display: block;
    flex-shrink: 0;
    height: 1.1em;
    width: 1.1em;
}

#mdbook-menu-bar .right-buttons .bookshelf-return-label {
    font-size: 0.85em;
    font-weight: 500;
}

@media (max-width: 700px) {
    #mdbook-menu-bar .right-buttons .bookshelf-return-label {
        border: 0;
        clip: rect(0 0 0 0);
        height: 1px;
        margin: -1px;
        overflow: hidden;
        padding: 0;
        position: absolute;
        white-space: nowrap;
        width: 1px;
    }
}
"#
    .replace("__LINK_CLASS__", BOOKSHELF_RETURN_LINK_CLASS)
}

fn render_bookshelf_search_js() -> String {
    format!(
        "(() => {{\n\
const metadata = readBookshelfPageMetadata();\n\
if (typeof metadata.searchIndexTarget === \"string\" && metadata.searchIndexTarget !== \"\") {{\n\
    window.path_to_searchindex_js = metadata.searchIndexTarget;\n\
}}\n\
\n\
function readBookshelfPageMetadata() {{\n\
    const node = document.getElementById(\"{metadata_id}\");\n\
    if (!node) {{\n\
        return {{}};\n\
    }}\n\
    try {{\n\
        return JSON.parse(node.textContent || \"{{}}\");\n\
    }} catch (_err) {{\n\
        return {{}};\n\
    }}\n\
}}\n\
}})();\n",
        metadata_id = BOOKSHELF_PAGE_METADATA_ID,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn inject_bookshelf_runtime_assets_preserves_existing_output_assets() {
        let temp_root = make_temp_dir("mdbook-bookshelf-bookshelf-ui");
        let ui_assets = BookshelfAssets::new(&temp_root, Path::new(".generated/bookshelf"));
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
            .inject_bookshelf_runtime_assets(&mut config)
            .expect("asset injection should succeed");

        let css_assets = config
            .get::<Vec<PathBuf>>("output.html.additional-css")
            .expect("css assets should deserialize")
            .expect("css assets should exist");
        assert_eq!(css_assets[0], PathBuf::from("shared/site.css"));
        assert_eq!(
            css_assets[1],
            PathBuf::from(".generated/bookshelf/bookshelf-return.css")
        );
        assert!(temp_root.join(&css_assets[1]).exists());

        let js_assets = config
            .get::<Vec<PathBuf>>("output.html.additional-js")
            .expect("js assets should deserialize")
            .expect("js assets should exist");
        assert_eq!(js_assets[0], PathBuf::from("shared/site.js"));
        assert_eq!(
            js_assets[1],
            PathBuf::from(".generated/bookshelf/bookshelf-return.js")
        );
        assert_eq!(
            js_assets[2],
            PathBuf::from(".generated/bookshelf/bookshelf-search.js")
        );
        assert_file_contents(
            temp_root.join(".generated/bookshelf/.gitignore"),
            BOOKSHELF_ASSET_GITIGNORE_CONTENTS,
        );
        assert_file_contents(
            temp_root.join(".generated/bookshelf/README.md"),
            BOOKSHELF_ASSET_README_CONTENTS,
        );
        assert!(temp_root.join(&js_assets[1]).exists());
        assert!(temp_root.join(&js_assets[2]).exists());

        let search_js = fs::read_to_string(temp_root.join(&js_assets[2]))
            .expect("search override should be readable");
        assert!(search_js.contains("window.path_to_searchindex_js"));
        assert!(search_js.contains("searchIndexTarget"));

        fs::remove_dir_all(temp_root).expect("temporary asset directory should be removed");
    }

    #[test]
    fn page_metadata_preprocessor_uses_page_relative_targets() {
        let preprocessor = BookshelfPageMetadataPreprocessor::new(
            PathBuf::from("modules/parser/docs"),
            PathBuf::from("index.html"),
            PathBuf::from("modules/parser/docs/bookshelf-searchindex.js"),
        );
        let mut chapter = Chapter::new(
            "Modal Groups",
            "# Modal Groups".to_string(),
            "reference/modal-groups.md",
            Vec::new(),
        );

        preprocessor
            .inject_chapter_metadata(&mut chapter)
            .expect("metadata should inject");

        assert!(chapter.content.contains(BOOKSHELF_PAGE_METADATA_ID));
        assert!(chapter
            .content
            .contains("\"bookshelfTarget\":\"../../../../index.html\""));
        assert!(chapter
            .content
            .contains("\"searchIndexTarget\":\"../bookshelf-searchindex.js\""));
    }

    #[test]
    fn page_metadata_preprocessor_includes_return_target_on_authored_bookshelf_page() {
        let preprocessor = BookshelfPageMetadataPreprocessor::new(
            PathBuf::from("docs"),
            PathBuf::from("index.html"),
            PathBuf::from("docs/bookshelf-searchindex.js"),
        );
        let mut chapter = Chapter::new(
            "Bookshelf",
            "# Bookshelf".to_string(),
            "bookshelf.md",
            Vec::new(),
        );

        preprocessor
            .inject_chapter_metadata(&mut chapter)
            .expect("metadata should inject");

        assert!(chapter
            .content
            .contains("\"bookshelfTarget\":\"../index.html\""));
        assert!(chapter
            .content
            .contains("\"searchIndexTarget\":\"bookshelf-searchindex.js\""));
    }

    #[test]
    fn render_bookshelf_search_js_uses_page_metadata() {
        let script = render_bookshelf_search_js();

        assert!(script.contains("searchIndexTarget"));
        assert!(script.contains("window.path_to_searchindex_js = metadata.searchIndexTarget;"));
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

    fn assert_file_contents(path: PathBuf, expected: &str) {
        let actual = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        assert_eq!(actual, expected);
    }
}
