use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const BOOKSHELF_UI_SITE_FIXTURE: &str = "tests/fixtures/bookshelf-ui-site";
const PUBLIC_SELF_CONTAINED_EXAMPLE: &str = "examples/self-contained";

const TOC_RUNTIME_HARNESS: &str = r##"
const fs = require("node:fs");

const [scriptPath, pageHref, pathToRoot] = process.argv.slice(1);
const scriptSource = fs.readFileSync(scriptPath, "utf8");
const defineMarker =
  "window.customElements.define('mdbook-sidebar-scrollbox', MDBookSidebarScrollbox);";
const defineIndex = scriptSource.indexOf(defineMarker);
if (defineIndex === -1) {
  throw new Error(`missing sidebar define marker in ${scriptPath}`);
}
const sidebarScript = scriptSource.slice(0, defineIndex + defineMarker.length);

let activeScrollbox = null;
const definedElements = new Map();

class FakeClassList {
  constructor(initialValue = "") {
    this.values = new Set(String(initialValue).split(/\s+/).filter(Boolean));
  }

  add(...names) {
    for (const name of names) {
      if (name) {
        this.values.add(String(name));
      }
    }
  }

  contains(name) {
    return this.values.has(String(name));
  }

  toggle(name) {
    const normalized = String(name);
    if (this.values.has(normalized)) {
      this.values.delete(normalized);
      return false;
    }
    this.values.add(normalized);
    return true;
  }
}

class FakeElement {
  constructor(tagName, attributes = {}) {
    this.tagName = String(tagName).toUpperCase();
    this.attributes = { ...attributes };
    this.children = [];
    this.content = [];
    this.parentElement = null;
    this.classList = new FakeClassList(attributes.class || "");
    this.scrollTop = 0;
  }

  appendChild(child) {
    child.parentElement = this;
    this.children.push(child);
    this.content.push(child);
    return child;
  }

  appendText(text) {
    this.content.push(String(text));
  }

  addEventListener() {}

  getAttribute(name) {
    return Object.prototype.hasOwnProperty.call(this.attributes, name)
      ? this.attributes[name]
      : null;
  }

  getBoundingClientRect() {
    return { top: 0 };
  }

  scrollIntoView() {}

  querySelectorAll(selector) {
    const results = [];
    this.walk((node) => {
      if (matchesSelector(node, selector)) {
        results.push(node);
      }
    });
    return results;
  }

  querySelector(selector) {
    return this.querySelectorAll(selector)[0] || null;
  }

  walk(visitor) {
    for (const child of this.children) {
      visitor(child);
      child.walk(visitor);
    }
  }

  get textContent() {
    return this.content
      .map((item) => (typeof item === "string" ? item : item.textContent))
      .join("");
  }
}

class FakeAnchorElement extends FakeElement {
  constructor(attributes = {}) {
    super("a", attributes);
    this._href = new URL(attributes.href || "", pageHref).toString();
  }

  get href() {
    return this._href;
  }

  set href(value) {
    this._href = new URL(String(value), pageHref).toString();
  }
}

class HTMLElement {
  constructor() {
    this._parsedRoot = null;
    this.scrollTop = 0;
  }

  set innerHTML(value) {
    this._parsedRoot = parseSidebarHtml(String(value));
  }

  get innerHTML() {
    return "";
  }

  addEventListener() {}

  getBoundingClientRect() {
    return { top: 0 };
  }

  querySelectorAll(selector) {
    return this._parsedRoot ? this._parsedRoot.querySelectorAll(selector) : [];
  }

  querySelector(selector) {
    return this._parsedRoot ? this._parsedRoot.querySelector(selector) : null;
  }
}

function matchesSelector(node, selector) {
  if (selector === "a") {
    return node.tagName === "A";
  }
  if (selector === ".active") {
    return node.classList.contains("active");
  }
  return false;
}

function parseSidebarHtml(html) {
  const root = new FakeElement("root");
  const stack = [root];
  const tokenPattern = /<!--[\s\S]*?-->|<\/?[^>]+>|[^<]+/g;
  let match;
  while ((match = tokenPattern.exec(html)) !== null) {
    const token = match[0];
    if (!token || token.startsWith("<!--")) {
      continue;
    }
    if (token.startsWith("</")) {
      stack.pop();
      continue;
    }
    if (token.startsWith("<")) {
      const isSelfClosing = token.endsWith("/>");
      const source = token.slice(1, token.length - (isSelfClosing ? 2 : 1)).trim();
      const whitespaceIndex = source.search(/\s/);
      const tagName =
        whitespaceIndex === -1 ? source.toLowerCase() : source.slice(0, whitespaceIndex).toLowerCase();
      const attributeSource = whitespaceIndex === -1 ? "" : source.slice(whitespaceIndex + 1);
      const attributes = {};
      const attributePattern = /([^\s=]+)(?:="([^"]*)")?/g;
      let attributeMatch;
      while ((attributeMatch = attributePattern.exec(attributeSource)) !== null) {
        attributes[attributeMatch[1]] = attributeMatch[2] || "";
      }
      const node =
        tagName === "a" ? new FakeAnchorElement(attributes) : new FakeElement(tagName, attributes);
      stack[stack.length - 1].appendChild(node);
      if (!isSelfClosing && !["br", "hr", "img", "input", "meta", "link"].includes(tagName)) {
        stack.push(node);
      }
      continue;
    }

    if (token.trim()) {
      stack[stack.length - 1].appendText(token.replace(/\s+/g, " "));
    }
  }
  return root;
}

function normalizeLabel(value) {
  return String(value)
    .replace(/\s+/g, " ")
    .trim()
    .replace(/^\d+(?:\.\d+)*\.\s*/, "");
}

globalThis.HTMLElement = HTMLElement;
globalThis.document = {
  location: {
    href: pageHref,
  },
  querySelector(selector) {
    if (selector === "#mdbook-sidebar .active") {
      return activeScrollbox ? activeScrollbox.querySelector(".active") : null;
    }
    return null;
  },
  querySelectorAll(selector) {
    if (selector === ".chapter-fold-toggle") {
      return [];
    }
    return [];
  },
};
globalThis.window = {
  customElements: {
    define(name, ctor) {
      definedElements.set(String(name), ctor);
    },
  },
};
globalThis.sessionStorage = {
  getItem() {
    return null;
  },
  removeItem() {},
  setItem() {},
};
globalThis.path_to_root = pathToRoot;

eval(sidebarScript);

const SidebarScrollbox = definedElements.get("mdbook-sidebar-scrollbox");
if (!SidebarScrollbox) {
  throw new Error(`failed to register mdbook-sidebar-scrollbox from ${scriptPath}`);
}

activeScrollbox = new SidebarScrollbox();
activeScrollbox.connectedCallback();

const links = activeScrollbox.querySelectorAll("a");
const labels = links.map((link) => normalizeLabel(link.textContent));
const activeLinks = links.filter((link) => link.classList.contains("active"));
const activeLink = activeLinks[0] || null;

process.stdout.write(
  [
    `activeCount=${activeLinks.length}`,
    `active=${activeLink ? normalizeLabel(activeLink.textContent) : ""}`,
    `activeHref=${activeLink ? activeLink.href : ""}`,
    `labels=${labels.join("|")}`,
  ].join("\n"),
);
"##;

const SEARCH_RUNTIME_HARNESS: &str = r##"
const fs = require("node:fs");

const [elasticlunrPath, searchIndexPath, pageHref, pathToRoot, query] = process.argv.slice(1);
const pageUrl = new URL(pageHref);

globalThis.window = {
  location: {
    href: pageHref,
    pathname: pageUrl.pathname,
  },
  search: {},
};
globalThis.document = {};
globalThis.path_to_root = pathToRoot;

const elasticlunrApi = require(elasticlunrPath);
eval(fs.readFileSync(searchIndexPath, "utf8"));

const index = elasticlunrApi.Index.load(window.search.index);
const results = index.search(query, window.search.search_options);
const first = results[0] || null;
let href = "";
let breadcrumbs = "";

if (first) {
  const url = String(window.search.doc_urls[first.ref] || "").split("#");
  if (url.length === 1) {
    url.push("");
  }
  const encodedSearch = encodeURIComponent(
    query
      .trim()
      .split(/\s+/)
      .filter(Boolean)
      .join(" "),
  ).replace(/'/g, "%27");
  href = new URL(
    pathToRoot + url[0] + "?highlight=" + encodedSearch + "#" + url[1],
    pageHref,
  ).toString();
  breadcrumbs = first.doc && first.doc.breadcrumbs ? first.doc.breadcrumbs : "";
}

process.stdout.write(
  [
    `count=${results.length}`,
    `breadcrumbs=${breadcrumbs}`,
    `href=${href}`,
  ].join("\n"),
);
"##;

const SEARCH_COLD_LOAD_AUDIT_HARNESS: &str = r##"
const fs = require("node:fs");
const path = require("node:path");

const [pageHtmlPath, pageHref] = process.argv.slice(1);
const pageHtml = fs.readFileSync(pageHtmlPath, "utf8");
const pageUrl = new URL(pageHref);
const appendedScripts = [];
const relevantScripts = [];

class FakeClassList {
  constructor(initialValue = "") {
    this.values = new Set(String(initialValue).split(/\s+/).filter(Boolean));
  }

  add(...names) {
    for (const name of names) {
      if (name) {
        this.values.add(String(name));
      }
    }
  }

  remove(...names) {
    for (const name of names) {
      this.values.delete(String(name));
    }
  }

  contains(name) {
    return this.values.has(String(name));
  }
}

function createElement(id, initialClassName = "") {
  return {
    id,
    textContent: "",
    value: "",
    attributes: {},
    children: [],
    classList: new FakeClassList(initialClassName),
    addEventListener() {},
    appendChild(child) {
      this.children.push(child);
      return child;
    },
    removeChild(child) {
      const index = this.children.indexOf(child);
      if (index !== -1) {
        this.children.splice(index, 1);
      }
      return child;
    },
    setAttribute(name, value) {
      this.attributes[String(name)] = String(value);
    },
    focus() {},
    select() {},
    querySelector() {
      return null;
    },
    get firstChild() {
      return this.children[0] || null;
    },
    get firstElementChild() {
      return this.children[0] || null;
    },
  };
}

function createAnchorElement() {
  let resolvedHref = pageHref;
  return {
    get href() {
      return resolvedHref;
    },
    set href(value) {
      const url = new URL(String(value), pageHref);
      resolvedHref = url.toString();
      this.protocol = url.protocol;
      this.hostname = url.hostname;
      this.port = url.port;
      this.search = url.search;
      this.pathname = url.pathname;
      this.hash = url.hash;
    },
    protocol: pageUrl.protocol,
    hostname: pageUrl.hostname,
    port: pageUrl.port,
    search: "",
    pathname: pageUrl.pathname,
    hash: "",
  };
}

const elements = new Map([
  ["mdbook-search-wrapper", createElement("mdbook-search-wrapper", "hidden")],
  ["mdbook-searchbar-outer", createElement("mdbook-searchbar-outer")],
  ["mdbook-searchbar", createElement("mdbook-searchbar")],
  ["mdbook-searchresults", createElement("mdbook-searchresults")],
  ["mdbook-searchresults-outer", createElement("mdbook-searchresults-outer", "hidden")],
  ["mdbook-searchresults-header", createElement("mdbook-searchresults-header")],
  ["mdbook-search-toggle", createElement("mdbook-search-toggle")],
  ["mdbook-content", createElement("mdbook-content")],
]);

const metadataMatch = pageHtml.match(
  /<script type="application\/json" id="mdbook-bookshelf-page-metadata">([\s\S]*?)<\/script>/,
);
if (metadataMatch) {
  const metadata = createElement("mdbook-bookshelf-page-metadata");
  metadata.textContent = metadataMatch[1] || "";
  elements.set("mdbook-bookshelf-page-metadata", metadata);
}

globalThis.Mark = function Mark() {
  return {
    mark() {},
    unmark() {},
  };
};
globalThis.elasticlunr = {};
globalThis.history = {
  pushState() {},
  replaceState() {},
};
globalThis.window = {
  location: {
    href: pageHref,
    pathname: pageUrl.pathname,
  },
  scrollTo() {},
  setTimeout() {},
  search: {},
};
globalThis.document = {
  activeElement: null,
  head: {
    append(node) {
      appendedScripts.push({
        id: node.id || "",
        src: node.src || "",
        configured: window.path_to_searchindex_js || "",
      });
    },
  },
  addEventListener() {},
  getElementById(id) {
    return elements.get(String(id)) || null;
  },
  querySelectorAll() {
    return [];
  },
  createElement(tagName) {
    const normalized = String(tagName).toLowerCase();
    if (normalized === "a") {
      return createAnchorElement();
    }
    return createElement(normalized);
  },
};
globalThis.__audit = {
  configuredAfterConfig: "",
  configuredAfterSearcher: "",
  configuredAfterOverride: "",
  pathToRoot: "",
  sequence: [],
};

const scriptPattern = /<script(?:\s+src="([^"]+)")?[^>]*>([\s\S]*?)<\/script>/g;
let match;
while ((match = scriptPattern.exec(pageHtml)) !== null) {
  const src = match[1] || "";
  const content = match[2] || "";
  if (!src && content.includes("const path_to_root =") && content.includes("window.path_to_searchindex_js")) {
    relevantScripts.push({
      label: "config",
      code: content,
    });
    continue;
  }
  if (src.includes("searcher-")) {
    relevantScripts.push({
      label: src,
      code: fs.readFileSync(path.resolve(path.dirname(pageHtmlPath), src), "utf8"),
    });
    continue;
  }
  if (src.endsWith("documentation-search.js")) {
    relevantScripts.push({
      label: src,
      code: fs.readFileSync(path.resolve(path.dirname(pageHtmlPath), src), "utf8"),
    });
  }
}

const combinedSource = relevantScripts
  .map((script) => {
    const lines = [`globalThis.__audit.sequence.push(${JSON.stringify(script.label)});`, script.code];
    if (script.label === "config") {
      lines.push(
        "globalThis.__audit.pathToRoot = typeof path_to_root === \"string\" ? path_to_root : \"\";",
      );
      lines.push(
        "globalThis.__audit.configuredAfterConfig = window.path_to_searchindex_js || \"\";",
      );
    } else if (script.label.includes("searcher-")) {
      lines.push(
        "globalThis.__audit.configuredAfterSearcher = window.path_to_searchindex_js || \"\";",
      );
    } else if (script.label.endsWith("documentation-search.js")) {
      lines.push(
        "globalThis.__audit.configuredAfterOverride = window.path_to_searchindex_js || \"\";",
      );
    }
    return lines.join("\n");
  })
  .join("\n");

eval(combinedSource);

const first = appendedScripts[0] || null;
process.stdout.write(
  [
    `sequence=${globalThis.__audit.sequence.join("|")}`,
    `pathToRoot=${globalThis.__audit.pathToRoot}`,
    `configuredAfterConfig=${globalThis.__audit.configuredAfterConfig}`,
    `configuredAfterSearcher=${globalThis.__audit.configuredAfterSearcher}`,
    `configuredAfterOverride=${globalThis.__audit.configuredAfterOverride}`,
    `requested=${first ? first.src : ""}`,
    `requestedId=${first ? first.id : ""}`,
    `requestedConfigured=${first ? first.configured : ""}`,
  ].join("\n"),
);
"##;

#[test]
fn build_cli_emits_documentation_index_and_book_runtime_assets_from_fixture_asset_dir() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_book"));
    let fixture_root =
        copy_bookshelf_ui_site_fixture(&repo_root, "chunk-011-build-cli-bookshelf-ui-site-fixture");
    let config_path = fixture_root.join("bookshelf.toml");
    write_documentation_index_bookshelf_ui_site_config(&config_path);
    let output_dir = make_temp_dir("chunk-011-build-cli", &repo_root);
    let fixture_entries_before =
        without_bookshelf_asset_entries(collect_tree_entries(&fixture_root));

    let output = Command::new(&bin)
        .arg("build")
        .arg(&config_path)
        .arg("--dest-dir")
        .arg(&output_dir)
        .output()
        .expect("build command should run");

    if !output.status.success() {
        panic!(
            "build command failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let root_entry_html = assert_read_to_string(output_dir.join("index.html"));
    assert_text_contains(&root_entry_html, "Table of Contents");
    assert_text_contains(&root_entry_html, "class=\"documentation-index\"");
    assert_text_contains(&root_entry_html, "class=\"documentation-index-toc\"");
    assert_text_contains(&root_entry_html, "class=\"documentation-category\"");
    assert_documentation_index_category(
        &root_entry_html,
        "category-core",
        "Core",
        &[("Fixture Core", "docs/index.html")],
    );
    assert_documentation_index_category(
        &root_entry_html,
        "category-modules",
        "Modules",
        &[
            ("Fixture Parser", "modules/parser/docs/index.html"),
            ("Fixture UI", "modules/ui/docs/index.html"),
        ],
    );
    assert_text_not_contains(&root_entry_html, "http-equiv=\"refresh\"");
    assert_text_not_contains(&root_entry_html, "Bookshelf");
    assert!(!output_dir.join("docs/bookshelf.html").exists());

    let root_index_html = assert_read_to_string(output_dir.join("docs/index.html"));
    assert_text_contains(
        &root_index_html,
        "<title>Fixture Core - Fixture Core</title>",
    );
    assert_text_contains(
        &root_index_html,
        "Repository-wide onboarding and architecture guidance for the fixture project.",
    );
    assert_text_contains(&root_index_html, "href=\"./onboarding.html\"");
    assert_text_contains(&root_index_html, "documentation-return.css");
    assert_text_contains(&root_index_html, "documentation-return.js");
    assert_text_contains(&root_index_html, "documentation-search.js");
    assert_text_not_contains(&root_index_html, "bookshelf-return.css");
    assert_text_not_contains(&root_index_html, "bookshelf-return.js");
    assert_text_not_contains(&root_index_html, "bookshelf-search.js");
    assert_text_contains(&root_index_html, "id=\"mdbook-bookshelf-page-metadata\"");
    assert_text_contains(
        &root_index_html,
        "\"documentationIndexTarget\":\"../index.html\"",
    );
    assert_text_contains(
        &root_index_html,
        "\"searchIndexTarget\":\"bookshelf-searchindex.js\"",
    );
    assert_text_not_contains(&root_index_html, "bookshelf-breadcrumb");
    assert_stock_search_contract(&output_dir.join("docs"), &root_index_html);
    assert_text_not_contains(&root_index_html, "Choose a book to enter its root page.");
    assert_text_not_contains(
        &root_index_html,
        "Parser-specific reference pages with their own reading order.",
    );

    let onboarding_html = assert_read_to_string(output_dir.join("docs/onboarding.html"));
    assert_text_contains(
        &onboarding_html,
        "href=\"../modules/parser/docs/index.html\">Fixture Parser</a>",
    );
    assert_text_not_contains(&onboarding_html, "href=\"/modules/parser/docs/index.md\"");

    let parser_index_html =
        assert_read_to_string(output_dir.join("modules/parser/docs/index.html"));
    assert_text_contains(&parser_index_html, "documentation-return.css");
    assert_text_contains(&parser_index_html, "documentation-return.js");
    assert_text_contains(&parser_index_html, "documentation-search.js");
    assert_text_not_contains(&parser_index_html, "bookshelf-return.css");
    assert_text_not_contains(&parser_index_html, "bookshelf-return.js");
    assert_text_not_contains(&parser_index_html, "bookshelf-search.js");
    assert_text_contains(
        &parser_index_html,
        "\"documentationIndexTarget\":\"../../../index.html\"",
    );
    assert_text_contains(
        &parser_index_html,
        "\"searchIndexTarget\":\"bookshelf-searchindex.js\"",
    );
    assert_text_not_contains(&parser_index_html, "bookshelf-breadcrumb");
    assert_stock_search_contract(&output_dir.join("modules/parser/docs"), &parser_index_html);
    let architecture_html = assert_read_to_string(output_dir.join("docs/architecture.html"));
    assert_text_not_contains(&architecture_html, "bookshelf-breadcrumb");
    let grammar_html = assert_read_to_string(output_dir.join("modules/parser/docs/grammar.html"));
    assert_text_not_contains(&grammar_html, "bookshelf-breadcrumb");

    let root_toc_html = assert_read_to_string(output_dir.join("docs/toc.html"));
    let root_toc_script = output_dir.join("docs").join(assert_has_file_with_prefix(
        &output_dir.join("docs"),
        "toc-",
        ".js",
    ));
    assert_sidebar_toc_scope(
        &root_toc_html,
        &["Fixture Core", "Onboarding", "Architecture"],
        &[
            "Documentation",
            "Bookshelf",
            "Fixture Parser",
            "Grammar",
            "Fixture UI",
            "Navigation",
        ],
    );
    assert_runtime_toc(
        &root_toc_script,
        "https://example.test/docs/index.html#what-it-does",
        "",
        &["Fixture Core", "Onboarding", "Architecture"],
        "Fixture Core",
        "https://example.test/docs/index.html",
    );

    assert_exists(output_dir.join("docs/index.html"));
    assert_exists(output_dir.join("docs/architecture.html"));
    assert_exists(output_dir.join("docs/onboarding.html"));
    assert_exists(output_dir.join("docs/toc.html"));
    assert_has_file_with_prefix(&output_dir.join("docs"), "book-", ".js");
    let root_return_script =
        assert_single_file_named_recursive(&output_dir.join("docs"), "documentation-return.js");
    let root_return_css =
        assert_single_file_named_recursive(&output_dir.join("docs"), "documentation-return.css");
    assert_file_contains(root_return_script.clone(), "readDocumentationPageMetadata");
    assert_file_contains(
        root_return_script.clone(),
        "label.textContent = \"Documentation\";",
    );
    assert_file_contains(
        root_return_script.clone(),
        "document.createElementNS(svgNamespace, \"svg\")",
    );
    assert_file_contains(
        root_return_script.clone(),
        "document.querySelector(\"#mdbook-menu-bar .right-buttons\")",
    );
    assert_file_contains(root_return_script, "metadata.documentationIndexTarget");
    assert_file_contains(root_return_css, ".documentation-return-link");
    assert_no_file_named_recursive(&output_dir.join("docs"), "bookshelf-return.js");
    assert_no_file_named_recursive(&output_dir.join("docs"), "bookshelf-return.css");
    assert_no_file_named_recursive(&output_dir.join("docs"), "bookshelf-search.js");
    assert_no_file_named_recursive(&output_dir.join("docs"), "bookshelf-breadcrumb.js");
    assert_no_file_named_recursive(&output_dir.join("docs"), "bookshelf-breadcrumb.css");
    assert_exists(output_dir.join("modules/parser/docs/index.html"));
    assert_exists(output_dir.join("modules/parser/docs/grammar.html"));
    assert_exists(output_dir.join("modules/parser/docs/toc.html"));
    assert_has_file_with_prefix(&output_dir.join("modules/parser/docs"), "book-", ".js");
    let parser_toc_html = assert_read_to_string(output_dir.join("modules/parser/docs/toc.html"));
    assert_sidebar_toc_scope(
        &parser_toc_html,
        &["Fixture Parser", "Grammar", "Runtime"],
        &[
            "Fixture Core",
            "Bookshelf",
            "Fixture UI",
            "Navigation",
            "Diagnostics",
        ],
    );
    let parser_toc_script =
        output_dir
            .join("modules/parser/docs")
            .join(assert_has_file_with_prefix(
                &output_dir.join("modules/parser/docs"),
                "toc-",
                ".js",
            ));
    let parser_return_script = assert_single_file_named_recursive(
        &output_dir.join("modules/parser/docs"),
        "documentation-return.js",
    );
    let parser_return_css = assert_single_file_named_recursive(
        &output_dir.join("modules/parser/docs"),
        "documentation-return.css",
    );
    assert_file_contains(
        parser_return_script.clone(),
        "metadata.documentationIndexTarget",
    );
    assert_file_contains(parser_return_script, "link.rel = \"up\";");
    assert_file_contains(parser_return_css, ".documentation-return-link");
    let parser_runtime_html =
        assert_read_to_string(output_dir.join("modules/parser/docs/runtime.html"));
    assert_text_contains(
        &parser_runtime_html,
        "href=\"../../ui/docs/index.html\">Fixture UI</a>",
    );
    assert_text_not_contains(&parser_runtime_html, "href=\"/modules/ui/docs/index.md\"");
    assert_runtime_toc(
        &parser_toc_script,
        "https://example.test/modules/parser/docs/grammar.html#deep-link",
        "",
        &["Fixture Parser", "Grammar", "Runtime"],
        "Grammar",
        "https://example.test/modules/parser/docs/grammar.html",
    );

    assert_exists(output_dir.join("modules/ui/docs/index.html"));
    assert_exists(output_dir.join("modules/ui/docs/navigation.html"));
    assert_exists(output_dir.join("modules/ui/docs/toc.html"));
    let ui_navigation_html =
        assert_read_to_string(output_dir.join("modules/ui/docs/navigation.html"));
    assert_text_contains(
        &ui_navigation_html,
        "href=\"../../../index.html\">site root</a>",
    );
    let ui_toc_html = assert_read_to_string(output_dir.join("modules/ui/docs/toc.html"));
    assert_sidebar_toc_scope(
        &ui_toc_html,
        &["Fixture UI", "Navigation", "Diagnostics"],
        &[
            "Fixture Core",
            "Bookshelf",
            "Fixture Parser",
            "Grammar",
            "Runtime",
        ],
    );
    let ui_toc_script = output_dir
        .join("modules/ui/docs")
        .join(assert_has_file_with_prefix(
            &output_dir.join("modules/ui/docs"),
            "toc-",
            ".js",
        ));
    assert_runtime_toc(
        &ui_toc_script,
        "https://example.test/modules/ui/docs/navigation.html?from=deep#section",
        "",
        &["Fixture UI", "Navigation", "Diagnostics"],
        "Navigation",
        "https://example.test/modules/ui/docs/navigation.html",
    );
    assert_no_authored_root_relative_markdown_links(&output_dir);
    assert!(!fixture_root.join(".mdbook-bookshelf").exists());
    assert!(fixture_root
        .join(".mdbook/bookshelf/documentation-return.js")
        .exists());
    assert!(fixture_root
        .join(".mdbook/bookshelf/documentation-search.js")
        .exists());
    assert_eq!(
        without_bookshelf_asset_entries(collect_tree_entries(&fixture_root)),
        fixture_entries_before
    );

    fs::remove_dir_all(&fixture_root).expect("temp fixture directory should be removed");
    fs::remove_dir_all(&output_dir).expect("temp output directory should be removed");
}

#[test]
fn build_cli_rejects_html_documentation_index_cover_paths() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_book"));
    let fixture_root = copy_bookshelf_ui_site_fixture(&repo_root, "build-cli-cover-collision");
    let config_path = fixture_root.join("bookshelf.toml");
    write_documentation_index_html_cover_config(&config_path);
    fs::write(
        fixture_root.join("docs/index.html"),
        "cover collision sentinel\n",
    )
    .expect("collision cover source should be written");
    let output_dir = make_temp_dir("build-cli-cover-collision-output", &repo_root);

    let output = Command::new(&bin)
        .arg("build")
        .arg(&config_path)
        .arg("--dest-dir")
        .arg(&output_dir)
        .output()
        .expect("build command should run");

    assert!(
        !output.status.success(),
        "build should reject HTML documentation index cover paths"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_text_contains(
        &stderr,
        "book 'core' documentation index cover 'docs/index.html' must not be an HTML file",
    );
    let root_output = output_dir.join("docs/index.html");
    if root_output.exists() {
        let root_output_html = assert_read_to_string(root_output);
        assert_text_not_contains(&root_output_html, "cover collision sentinel");
    }

    fs::remove_dir_all(&fixture_root).expect("temp fixture directory should be removed");
    fs::remove_dir_all(&output_dir).expect("temp output directory should be removed");
}

#[test]
fn build_cli_smoke_builds_public_self_contained_example_from_default_config_path() {
    require_mdbook_variables();

    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_book"));
    let fixture_root =
        copy_public_self_contained_example_fixture(&repo_root, "chunk-011-public-example-fixture");
    let output_dir = make_temp_dir("chunk-011-build-cli-default-config", &repo_root);

    let output = Command::new(&bin)
        .current_dir(&fixture_root)
        .arg("build")
        .arg("--dest-dir")
        .arg(&output_dir)
        .output()
        .expect("build command should run");

    if !output.status.success() {
        panic!(
            "build command failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let documentation_index_html = assert_read_to_string(output_dir.join("index.html"));
    assert_text_contains(&documentation_index_html, "Table of Contents");
    assert_text_contains(&documentation_index_html, "class=\"documentation-index\"");
    assert_documentation_index_category(
        &documentation_index_html,
        "category-core",
        "Core",
        &[("Example Core", "docs/index.html")],
    );
    assert_documentation_index_category(
        &documentation_index_html,
        "category-modules",
        "Modules",
        &[
            ("Example Parser", "modules/parser/docs/index.html"),
            ("Example UI", "modules/ui/docs/index.html"),
        ],
    );
    assert!(!output_dir.join("docs/bookshelf.html").exists());

    let onboarding_html = assert_read_to_string(output_dir.join("docs/onboarding.html"));
    assert_text_contains(
        &onboarding_html,
        "href=\"../modules/parser/docs/index.html\">Example Parser</a>",
    );
    assert_text_contains(
        &onboarding_html,
        "href=\"../modules/parser/docs/grammar.html\">Example Parser Grammar</a>",
    );
    assert_text_not_contains(&onboarding_html, "{{ParserRoot}}");

    let parser_runtime_html =
        assert_read_to_string(output_dir.join("modules/parser/docs/runtime.html"));
    assert_text_contains(
        &parser_runtime_html,
        "href=\"../../ui/docs/index.html\">Example UI</a>",
    );
    assert_text_contains(
        &parser_runtime_html,
        "href=\"../../ui/docs/index.html\">Example UI Variable</a>",
    );
    assert_text_not_contains(&parser_runtime_html, "{{UiRoot}}");

    fs::remove_dir_all(&fixture_root).expect("temp fixture directory should be removed");
    fs::remove_dir_all(&output_dir).expect("temp output directory should be removed");
}

#[test]
fn build_cli_prints_concise_relative_progress() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_book"));
    let fixture_root = copy_bookshelf_ui_site_fixture(&repo_root, "build-cli-layout-log");
    write_documentation_index_bookshelf_ui_site_config(&fixture_root.join("bookshelf.toml"));
    let output_dir = PathBuf::from(".site-log");

    let output = Command::new(&bin)
        .current_dir(&fixture_root)
        .arg("build")
        .arg("--dest-dir")
        .arg(&output_dir)
        .output()
        .expect("build command should run");

    if !output.status.success() {
        panic!(
            "build command failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_text_contains(&stderr, "Build");
    assert_text_contains(&stderr, "  config: bookshelf.toml");
    assert_text_contains(&stderr, "  root: .");
    assert_text_contains(&stderr, "  output: .site-log");
    assert_text_contains(&stderr, "  books: 3");
    assert_text_contains(&stderr, "[1/3] Fixture Core: docs");
    assert_text_contains(&stderr, "[2/3] Fixture Parser: modules/parser/docs");
    assert_text_contains(&stderr, "[3/3] Fixture UI: modules/ui/docs");
    let core_line = stderr
        .lines()
        .find(|line| line.contains("[1/3] Fixture Core: docs"))
        .expect("core progress line should be present");
    let parser_line = stderr
        .lines()
        .find(|line| line.contains("[2/3] Fixture Parser: modules/parser/docs"))
        .expect("parser progress line should be present");
    let ui_line = stderr
        .lines()
        .find(|line| line.contains("[3/3] Fixture UI: modules/ui/docs"))
        .expect("ui progress line should be present");
    assert_text_contains(core_line, ", 3 pages)");
    assert_text_contains(parser_line, ", 3 pages)");
    assert_text_contains(ui_line, ", 3 pages)");
    assert_text_contains(&stderr, "documentation index:");
    assert_text_contains(&stderr, "search index:");
    assert_text_contains(&stderr, "Finished: .site-log");
    assert_text_not_contains(&stderr, "Loading bookshelf config and source catalog...");
    assert_text_not_contains(&stderr, "Catalog contains 3 books.");
    assert_text_not_contains(&stderr, "sources:");
    assert_text_not_contains(&stderr, "Preparing bookshelf link metadata...");
    assert_text_not_contains(&stderr, "mapping site-root links");
    assert_text_not_contains(&stderr, "Building 3 books with mdBook...");
    assert_text_not_contains(&stderr, "Writing site-root redirect...");
    assert_text_not_contains(&stderr, ".site-log/modules/parser/docs");

    fs::remove_dir_all(&fixture_root).expect("temp fixture directory should be removed");
}

#[test]
fn build_cli_prints_absolute_output_path_outside_current_directory() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_book"));
    let fixture_root = copy_bookshelf_ui_site_fixture(&repo_root, "build-cli-outside-cwd");
    let output_dir = make_temp_dir("build-cli-outside-cwd-output", &repo_root);
    let output_display = output_dir.to_string_lossy().replace('\\', "/");

    let output = Command::new(&bin)
        .current_dir(&fixture_root)
        .arg("build")
        .arg("--dest-dir")
        .arg(&output_dir)
        .output()
        .expect("build command should run");

    if !output.status.success() {
        panic!(
            "build command failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_text_contains(&stderr, &format!("  output: {output_display}"));
    assert_text_contains(&stderr, &format!("Finished: {output_display}"));
    assert_text_not_contains(&stderr, "../build-cli-outside-cwd-output");

    fs::remove_dir_all(&fixture_root).expect("temp fixture directory should be removed");
    fs::remove_dir_all(&output_dir).expect("temp output directory should be removed");
}

#[test]
fn build_cli_prints_error_cause_chain_for_authoring_failures() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_book"));
    let config_path =
        repo_root.join("tests/fixtures/multi-book-load/invalid-summary-parse/bookshelf.toml");
    let output_dir = make_temp_dir("build-cli-error-chain", &repo_root);

    let output = Command::new(&bin)
        .arg("build")
        .arg(&config_path)
        .arg("--dest-dir")
        .arg(&output_dir)
        .output()
        .expect("build command should run");

    assert!(
        !output.status.success(),
        "invalid summary fixture should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_text_contains(
        &stderr,
        "error: failed to build site-root Markdown link map",
    );
    assert_text_contains(&stderr, "Caused by:");
    assert_text_contains(
        &stderr,
        "failed to load books for site-root Markdown link map",
    );
    assert_text_contains(&stderr, "book 'broken' failed to parse canonical summary");
    assert_text_contains(&stderr, "broken-book/docs/SUMMARY.md");
    assert_text_contains(&stderr, "failed to parse SUMMARY.md line");

    fs::remove_dir_all(&output_dir).expect("temp output directory should be removed");
}

#[test]
fn build_cli_resolves_relative_mdbook_paths_from_bookshelf_config_dir() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_book"));
    let fixture_root = repo_root.join("tests/fixtures/build-cli/shared-config-root");
    let config_path = fixture_root.join("bookshelf.toml");
    let output_dir = make_temp_dir("chunk-011-shared-config-root", &repo_root);
    let preserved_asset_dir = fixture_root.join("modules/child/bookshelf-config-assets");
    let preserved_asset_file = preserved_asset_dir.join("user.txt");

    fs::create_dir_all(&preserved_asset_dir).expect("preexisting asset directory should exist");
    fs::write(&preserved_asset_file, "keep me\n")
        .expect("preexisting staged asset file should be written");

    let output = Command::new(&bin)
        .arg("build")
        .arg(&config_path)
        .arg("--dest-dir")
        .arg(&output_dir)
        .output()
        .expect("build command should run");

    if !output.status.success() {
        panic!(
            "build command failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let root_css = assert_has_file_with_prefix(&output_dir.join("docs/shared"), "site-", ".css");
    let child_css_path = assert_single_file_with_prefix_recursive(
        &output_dir.join("modules/child/docs"),
        "site-",
        ".css",
    );
    let child_css = child_css_path
        .strip_prefix(output_dir.join("modules/child/docs"))
        .expect("child css should be emitted under child book output")
        .to_string_lossy()
        .replace('\\', "/");

    assert_file_contains(
        output_dir.join("docs/index.html"),
        &format!("shared/{root_css}"),
    );
    assert_file_contains(output_dir.join("modules/child/docs/index.html"), &child_css);
    assert_file_contains(child_css_path, "border-top: 4px solid #0b7285");
    assert!(
        !child_css.contains(".mdbook-bookshelf/"),
        "child staged mdBook assets should not use hidden output paths"
    );
    assert!(
        !fixture_root
            .join("modules/child/.mdbook-bookshelf")
            .exists(),
        "transient staged mdBook assets should not remain in the child book root"
    );
    assert_file_contains(preserved_asset_file.clone(), "keep me");
    assert_eq!(
        vec![PathBuf::from("user.txt")],
        collect_tree_entries(&preserved_asset_dir),
        "temporary staged config assets should be cleaned without deleting user-owned files"
    );

    fs::remove_file(&preserved_asset_file)
        .expect("preexisting staged asset file should be removed");
    fs::remove_dir(&preserved_asset_dir)
        .expect("preexisting staged asset directory should be removable after cleanup");

    fs::remove_dir_all(&output_dir).expect("temp output directory should be removed");
}

#[test]
fn build_cli_supports_configured_mdbook_mermaid_preprocessor_and_additional_js() {
    require_mdbook_mermaid();

    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_book"));
    let fixture_root = make_temp_dir("plugin-regression-mermaid", &repo_root);
    let config_path = fixture_root.join("bookshelf.toml");
    let output_dir = make_temp_dir("plugin-regression-mermaid-out", &repo_root);

    fs::create_dir_all(fixture_root.join("docs")).expect("root docs directory should be created");
    fs::create_dir_all(fixture_root.join("modules/child/docs"))
        .expect("child docs directory should be created");
    fs::write(
        &config_path,
        r#"
[book]
title = "Mermaid Root"
language = "en"
src = "docs"

[preprocessor.mermaid]
command = "mdbook-mermaid"

[output.html]
additional-js = ["mermaid.min.js", "mermaid-init.js"]

[bookshelf]
root-book-id = "root"

[[bookshelf.book]]
id = "child"
title = "Mermaid Child"
src = "modules/child/docs"

[[bookshelf.category]]
title = "All Docs"
books = ["root", "child"]
"#,
    )
    .expect("bookshelf config should be written");
    fs::write(
        fixture_root.join("mermaid.min.js"),
        "window.__bookshelfMermaidRuntime = true;\n",
    )
    .expect("mermaid runtime fixture should be written");
    fs::write(
        fixture_root.join("mermaid-init.js"),
        "window.__bookshelfMermaidInit = true;\n",
    )
    .expect("mermaid init fixture should be written");
    fs::write(
        fixture_root.join("docs/SUMMARY.md"),
        "# Summary\n\n- [Root Diagram](index.md)\n",
    )
    .expect("root summary should be written");
    fs::write(
        fixture_root.join("docs/index.md"),
        r#"# Root Diagram

```mermaid
flowchart TD
  Root[Root catalog book] --> Shared[Shared configured preprocessor]
```
"#,
    )
    .expect("root index should be written");
    fs::write(
        fixture_root.join("modules/child/docs/SUMMARY.md"),
        "# Summary\n\n- [Child Diagram](index.md)\n",
    )
    .expect("child summary should be written");
    fs::write(
        fixture_root.join("modules/child/docs/index.md"),
        r#"# Child Diagram

```mermaid
sequenceDiagram
  participant Child
  participant Shared
  Child->>Shared: configured preprocessor
```
"#,
    )
    .expect("child index should be written");

    run_build_cli(&bin, &config_path, &output_dir);

    let root_output = output_dir.join("docs");
    let child_output = output_dir.join("modules/child/docs");
    let root_index_html = assert_read_to_string(root_output.join("index.html"));
    let child_index_html = assert_read_to_string(child_output.join("index.html"));

    assert_text_contains(&root_index_html, "<pre class=\"mermaid\">");
    assert_text_contains(&child_index_html, "<pre class=\"mermaid\">");
    assert_text_not_contains(&root_index_html, "language-mermaid");
    assert_text_not_contains(&child_index_html, "language-mermaid");

    let root_mermaid_js = assert_has_file_with_prefix(&root_output, "mermaid-", ".min.js");
    let root_mermaid_init_js = assert_has_file_with_prefix(&root_output, "mermaid-init-", ".js");
    assert_text_contains(&root_index_html, &root_mermaid_js);
    assert_text_contains(&root_index_html, &root_mermaid_init_js);
    assert_file_contains(
        root_output.join(&root_mermaid_js),
        "__bookshelfMermaidRuntime",
    );
    assert_file_contains(
        root_output.join(&root_mermaid_init_js),
        "__bookshelfMermaidInit",
    );

    let child_mermaid_js = child_output.join(assert_has_file_with_prefix(
        &child_output,
        "mermaid-",
        ".min.js",
    ));
    let child_mermaid_init_js = child_output.join(assert_has_file_with_prefix(
        &child_output,
        "mermaid-init-",
        ".js",
    ));
    let child_mermaid_ref = child_mermaid_js
        .strip_prefix(&child_output)
        .expect("child mermaid script should be staged under child output")
        .to_string_lossy()
        .replace('\\', "/");
    let child_mermaid_init_ref = child_mermaid_init_js
        .strip_prefix(&child_output)
        .expect("child mermaid init script should be staged under child output")
        .to_string_lossy()
        .replace('\\', "/");
    assert_text_not_contains(&child_mermaid_ref, "bookshelf-config-assets/");
    assert_text_not_contains(&child_mermaid_init_ref, "bookshelf-config-assets/");
    assert_text_contains(&child_index_html, &child_mermaid_ref);
    assert_text_contains(&child_index_html, &child_mermaid_init_ref);
    assert_file_contains(child_mermaid_js, "__bookshelfMermaidRuntime");
    assert_file_contains(child_mermaid_init_js, "__bookshelfMermaidInit");

    assert_text_contains(&root_index_html, "documentation-return.js");
    assert_text_contains(&root_index_html, "documentation-search.js");
    assert_text_contains(&child_index_html, "documentation-return.js");
    assert_text_contains(&child_index_html, "documentation-search.js");
    assert_single_file_named_recursive(&root_output, "documentation-return.js");
    assert_single_file_named_recursive(&root_output, "documentation-search.js");
    assert_single_file_named_recursive(&child_output, "documentation-return.js");
    assert_single_file_named_recursive(&child_output, "documentation-search.js");
    assert_no_file_named_recursive(&root_output, "bookshelf-breadcrumb.js");
    assert_no_file_named_recursive(&child_output, "bookshelf-breadcrumb.js");

    fs::remove_dir_all(&fixture_root).expect("temp fixture directory should be removed");
    fs::remove_dir_all(&output_dir).expect("temp output directory should be removed");
}

#[test]
fn build_cli_supports_mdbook_variables_before_site_root_link_rewrites() {
    require_mdbook_variables();

    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_book"));
    let fixture_root = make_temp_dir("plugin-regression-variables", &repo_root);
    let config_path = fixture_root.join("bookshelf.toml");
    let output_dir = make_temp_dir("plugin-regression-variables-out", &repo_root);

    fs::create_dir_all(fixture_root.join("docs")).expect("root docs directory should be created");
    fs::create_dir_all(fixture_root.join("modules/parser/docs"))
        .expect("parser docs directory should be created");
    fs::create_dir_all(fixture_root.join("modules/ui/docs"))
        .expect("ui docs directory should be created");
    fs::write(
        &config_path,
        r#"
[book]
title = "Variables Root"
language = "en"
src = "docs"

[preprocessor.variables.variables]
ParserRoot = "/modules/parser/docs"
UiRoot = "/modules/ui/docs"

[bookshelf]
root-book-id = "root"

[[bookshelf.book]]
id = "parser"
title = "Variables Parser"
src = "modules/parser/docs"

[[bookshelf.book]]
id = "ui"
title = "Variables UI"
src = "modules/ui/docs"

[[bookshelf.category]]
title = "All Docs"
books = ["root", "parser", "ui"]
"#,
    )
    .expect("bookshelf config should be written");
    fs::write(
        fixture_root.join("docs/SUMMARY.md"),
        "# Summary\n\n- [Root Variables](index.md)\n",
    )
    .expect("root summary should be written");
    fs::write(
        fixture_root.join("docs/index.md"),
        r#"# Root Variables

Leading slash route: [Parser Absolute](/modules/parser/docs/index.md).

Variable route: [Parser Variable]({{ParserRoot}}/grammar.md).
"#,
    )
    .expect("root index should be written");
    fs::write(
        fixture_root.join("modules/parser/docs/SUMMARY.md"),
        "# Summary\n\n- [Parser Variables](index.md)\n  - [Grammar](grammar.md)\n",
    )
    .expect("parser summary should be written");
    fs::write(
        fixture_root.join("modules/parser/docs/index.md"),
        r#"# Parser Variables

Variable route: [UI Variable]({{UiRoot}}/index.md).
"#,
    )
    .expect("parser index should be written");
    fs::write(
        fixture_root.join("modules/parser/docs/grammar.md"),
        "# Grammar\n",
    )
    .expect("parser grammar should be written");
    fs::write(
        fixture_root.join("modules/ui/docs/SUMMARY.md"),
        "# Summary\n\n- [UI Variables](index.md)\n",
    )
    .expect("ui summary should be written");
    fs::write(
        fixture_root.join("modules/ui/docs/index.md"),
        "# UI Variables\n",
    )
    .expect("ui index should be written");

    run_build_cli(&bin, &config_path, &output_dir);

    let root_index_html = assert_read_to_string(output_dir.join("docs/index.html"));
    assert_text_contains(
        &root_index_html,
        "href=\"../modules/parser/docs/index.html\">Parser Absolute</a>",
    );
    assert_text_contains(
        &root_index_html,
        "href=\"../modules/parser/docs/grammar.html\">Parser Variable</a>",
    );
    assert_text_not_contains(&root_index_html, "{{ParserRoot}}");
    assert_text_not_contains(&root_index_html, "href=\"/modules/parser/docs/grammar.md\"");

    let parser_index_html =
        assert_read_to_string(output_dir.join("modules/parser/docs/index.html"));
    assert_text_contains(
        &parser_index_html,
        "href=\"../../ui/docs/index.html\">UI Variable</a>",
    );
    assert_text_not_contains(&parser_index_html, "{{UiRoot}}");
    assert_text_not_contains(&parser_index_html, "href=\"/modules/ui/docs/index.md\"");

    fs::remove_dir_all(&fixture_root).expect("temp fixture directory should be removed");
    fs::remove_dir_all(&output_dir).expect("temp output directory should be removed");
}

#[test]
fn build_cli_allows_relative_input_404_from_shared_root_model() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_book"));
    let fixture_root = make_temp_dir("chunk-022-shared-input-404", &repo_root);
    let config_path = fixture_root.join("bookshelf.toml");
    let output_dir = make_temp_dir("chunk-022-shared-input-404-out", &repo_root);

    fs::create_dir_all(fixture_root.join("docs")).expect("root docs directory should be created");
    fs::create_dir_all(fixture_root.join("modules/child/docs"))
        .expect("child docs directory should be created");
    fs::write(
        &config_path,
        r#"
[book]
title = "Root Book"
src = "docs"

[output.html]
input-404 = "missing.md"

[bookshelf]
root-book-id = "root"

[[bookshelf.book]]
id = "child"
title = "Child Book"
src = "modules/child/docs"

[[bookshelf.category]]
title = "All Docs"
books = ["root", "child"]
"#,
    )
    .expect("bookshelf config should be written");
    fs::write(
        fixture_root.join("docs/SUMMARY.md"),
        "# Summary\n\n- [Root](index.md)\n",
    )
    .expect("root summary should be written");
    fs::write(fixture_root.join("docs/index.md"), "# Root\n")
        .expect("root index should be written");
    fs::write(
        fixture_root.join("docs/missing.md"),
        "# Shared Missing Page\n",
    )
    .expect("shared 404 input should be written");
    fs::write(
        fixture_root.join("modules/child/docs/SUMMARY.md"),
        "# Summary\n\n- [Child](index.md)\n",
    )
    .expect("child summary should be written");
    fs::write(
        fixture_root.join("modules/child/docs/index.md"),
        "# Child\n",
    )
    .expect("child index should be written");
    fs::write(
        fixture_root.join("modules/child/docs/missing.md"),
        "# Child Missing Page\n",
    )
    .expect("child 404 input should be written");

    let output = Command::new(&bin)
        .arg("build")
        .arg(&config_path)
        .arg("--dest-dir")
        .arg(&output_dir)
        .output()
        .expect("build command should run");

    if !output.status.success() {
        panic!(
            "build command failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    assert_exists(output_dir.join("docs/index.html"));
    assert_exists(output_dir.join("modules/child/docs/index.html"));

    fs::remove_dir_all(&fixture_root).expect("temp fixture directory should be removed");
    fs::remove_dir_all(&output_dir).expect("temp output directory should be removed");
}

#[test]
fn build_cli_allows_disabled_input_404_for_child_roots() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_book"));
    let fixture_root = make_temp_dir("chunk-022-disabled-input-404", &repo_root);
    let config_path = fixture_root.join("bookshelf.toml");
    let output_dir = make_temp_dir("chunk-022-disabled-input-404-out", &repo_root);

    fs::create_dir_all(fixture_root.join("docs")).expect("root docs directory should be created");
    fs::create_dir_all(fixture_root.join("modules/child/docs"))
        .expect("child docs directory should be created");
    fs::write(
        &config_path,
        r#"
[book]
title = "Root Book"
src = "docs"

[output.html]
input-404 = ""

[bookshelf]
root-book-id = "root"

[[bookshelf.book]]
id = "child"
title = "Child Book"
src = "modules/child/docs"

[[bookshelf.category]]
title = "All Docs"
books = ["root", "child"]
"#,
    )
    .expect("bookshelf config should be written");
    fs::write(
        fixture_root.join("docs/SUMMARY.md"),
        "# Summary\n\n- [Root](index.md)\n",
    )
    .expect("root summary should be written");
    fs::write(fixture_root.join("docs/index.md"), "# Root\n")
        .expect("root index should be written");
    fs::write(
        fixture_root.join("modules/child/docs/SUMMARY.md"),
        "# Summary\n\n- [Child](index.md)\n",
    )
    .expect("child summary should be written");
    fs::write(
        fixture_root.join("modules/child/docs/index.md"),
        "# Child\n",
    )
    .expect("child index should be written");

    let output = Command::new(&bin)
        .arg("build")
        .arg(&config_path)
        .arg("--dest-dir")
        .arg(&output_dir)
        .output()
        .expect("build command should run");

    if !output.status.success() {
        panic!(
            "build command failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    assert!(!output_dir.join("modules/child/docs/404.html").exists());

    fs::remove_dir_all(&fixture_root).expect("temp fixture directory should be removed");
    fs::remove_dir_all(&output_dir).expect("temp output directory should be removed");
}

#[test]
fn build_cli_uses_shared_site_wide_search_index() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_book"));
    let fixture_root =
        copy_bookshelf_ui_site_fixture(&repo_root, "chunk-019-build-cli-bookshelf-ui-site-fixture");
    let config_path = fixture_root.join("bookshelf.toml");
    let output_dir = make_temp_dir("chunk-019-build-cli", &repo_root);

    let output = Command::new(&bin)
        .arg("build")
        .arg(&config_path)
        .arg("--dest-dir")
        .arg(&output_dir)
        .output()
        .expect("build command should run");

    if !output.status.success() {
        panic!(
            "build command failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let shared_search_path = output_dir.join("searchindex.js");
    let shared_search_js = assert_read_to_string(shared_search_path.clone());
    assert_text_contains(
        &shared_search_js,
        "window.search = Object.assign(window.search, {",
    );
    assert_text_contains(
        &shared_search_js,
        "\"modules/parser/docs/grammar.html#grammar\"",
    );
    assert_text_contains(&shared_search_js, "\"docs/architecture.html#architecture\"");

    let root_index_html = assert_read_to_string(output_dir.join("docs/index.html"));
    let parser_grammar_html =
        assert_read_to_string(output_dir.join("modules/parser/docs/grammar.html"));
    assert_text_contains(&root_index_html, "documentation-search.js");
    assert_text_contains(&parser_grammar_html, "documentation-search.js");

    let root_search_override =
        assert_single_file_named_recursive(&output_dir.join("docs"), "documentation-search.js");
    let parser_search_override = assert_single_file_named_recursive(
        &output_dir.join("modules/parser/docs"),
        "documentation-search.js",
    );
    assert_file_contains(
        root_search_override,
        "window.path_to_searchindex_js = metadata.searchIndexTarget;",
    );
    assert_file_contains(
        parser_search_override,
        "window.path_to_searchindex_js = metadata.searchIndexTarget;",
    );
    assert_text_contains(
        &parser_grammar_html,
        "\"searchIndexTarget\":\"bookshelf-searchindex.js\"",
    );
    assert_text_contains(
        &parser_grammar_html,
        "\"documentationIndexTarget\":\"../../../index.html\"",
    );

    let elasticlunr_js = output_dir.join("docs").join(assert_has_file_with_prefix(
        &output_dir.join("docs"),
        "elasticlunr-",
        ".min.js",
    ));

    assert_runtime_search_result(
        &elasticlunr_js,
        &output_dir.join("docs/bookshelf-searchindex.js"),
        "https://example.test/docs/index.html",
        "",
        "concern sidebar",
        "Fixture Parser » Grammar » Grammar",
        "https://example.test/modules/parser/docs/grammar.html?highlight=concern%20sidebar#grammar",
    );
    assert_runtime_search_result(
        &elasticlunr_js,
        &output_dir.join("modules/parser/docs/bookshelf-searchindex.js"),
        "https://example.test/modules/parser/docs/grammar.html",
        "",
        "scoped label",
        "Fixture Core » Architecture » Architecture",
        "https://example.test/docs/architecture.html?highlight=scoped%20label#architecture",
    );

    fs::remove_dir_all(&fixture_root).expect("temp fixture directory should be removed");
    fs::remove_dir_all(&output_dir).expect("temp output directory should be removed");
}

#[test]
fn build_cli_repo_scale_whole_system_acceptance_audit() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_book"));
    let fixture_root = repo_root.join("tests/fixtures/build-cli/repo-scale");
    let config_path = fixture_root.join("bookshelf.toml");
    let output_dir = make_temp_dir("chunk-020-repo-scale", &repo_root);

    run_build_cli(&bin, &config_path, &output_dir);

    let documentation_index_html = assert_read_to_string(output_dir.join("index.html"));
    assert_text_contains(&documentation_index_html, "Table of Contents");
    assert_text_contains(&documentation_index_html, "class=\"documentation-index\"");
    assert_documentation_index_category(
        &documentation_index_html,
        "category-overview",
        "Overview",
        &[("MetaNC", "docs/index.html")],
    );
    assert_documentation_index_category(
        &documentation_index_html,
        "category-modules",
        "Modules",
        &[
            ("G-code Parser", "modules/gcode-parser/docs/index.html"),
            ("HMI", "modules/hmi/docs/index.html"),
        ],
    );
    assert!(!output_dir.join("docs/bookshelf.html").exists());
    assert!(!output_dir.join("bookshelf.html").exists());

    let metanc_index_html = assert_read_to_string(output_dir.join("docs/index.html"));
    let architecture_html = assert_read_to_string(output_dir.join("docs/architecture.html"));
    let parser_index_html =
        assert_read_to_string(output_dir.join("modules/gcode-parser/docs/index.html"));
    let modal_groups_html = assert_read_to_string(
        output_dir.join("modules/gcode-parser/docs/reference/modal-groups.html"),
    );
    let hmi_index_html = assert_read_to_string(output_dir.join("modules/hmi/docs/index.html"));

    assert_text_contains(
        &metanc_index_html,
        "\"documentationIndexTarget\":\"../index.html\"",
    );
    assert_text_contains(
        &parser_index_html,
        "\"documentationIndexTarget\":\"../../../index.html\"",
    );
    assert_text_contains(
        &hmi_index_html,
        "\"documentationIndexTarget\":\"../../../index.html\"",
    );
    assert_stock_search_contract(&output_dir.join("docs"), &metanc_index_html);
    assert_stock_search_contract(
        &output_dir.join("modules/gcode-parser/docs"),
        &parser_index_html,
    );
    assert_stock_search_contract(&output_dir.join("modules/hmi/docs"), &hmi_index_html);
    assert_text_contains(&architecture_html, "documentation-search.js");
    assert_text_contains(&modal_groups_html, "documentation-search.js");
    assert_text_not_contains(&architecture_html, "bookshelf-breadcrumb");
    assert_text_not_contains(&modal_groups_html, "bookshelf-breadcrumb");

    let root_toc_html = assert_read_to_string(output_dir.join("docs/toc.html"));
    assert_sidebar_toc_scope(
        &root_toc_html,
        &["MetaNC", "Getting Started", "Architecture"],
        &[
            "Bookshelf",
            "G-code Parser",
            "Grammar",
            "HMI",
            "Operator Panels",
        ],
    );

    let parser_toc_html =
        assert_read_to_string(output_dir.join("modules/gcode-parser/docs/toc.html"));
    assert_sidebar_toc_scope(
        &parser_toc_html,
        &["G-code Parser", "Grammar", "Modal Groups", "Diagnostics"],
        &[
            "MetaNC",
            "Bookshelf",
            "HMI",
            "Operator Panels",
            "Alarm Flow",
        ],
    );
    assert_text_contains(&parser_toc_html, "1.1.1.</strong> Modal Groups");
    assert_text_contains(&parser_toc_html, "1.2.</strong> Diagnostics");

    let hmi_toc_html = assert_read_to_string(output_dir.join("modules/hmi/docs/toc.html"));
    assert_sidebar_toc_scope(
        &hmi_toc_html,
        &["HMI", "Operator Panels", "Alarm Flow"],
        &[
            "MetaNC",
            "Bookshelf",
            "G-code Parser",
            "Grammar",
            "Modal Groups",
        ],
    );

    assert_no_file_named_recursive(&output_dir.join("docs"), "bookshelf-breadcrumb.js");
    assert_no_file_named_recursive(
        &output_dir.join("modules/gcode-parser/docs"),
        "bookshelf-breadcrumb.js",
    );

    let parser_toc_script =
        output_dir
            .join("modules/gcode-parser/docs")
            .join(assert_has_file_with_prefix(
                &output_dir.join("modules/gcode-parser/docs"),
                "toc-",
                ".js",
            ));
    assert_runtime_toc(
        &parser_toc_script,
        "https://example.test/modules/gcode-parser/docs/reference/modal-groups.html#group-one",
        "../",
        &["G-code Parser", "Grammar", "Modal Groups", "Diagnostics"],
        "Modal Groups",
        "https://example.test/modules/gcode-parser/docs/reference/modal-groups.html",
    );

    let shared_search_path = output_dir.join("searchindex.js");
    let shared_search_js = assert_read_to_string(shared_search_path.clone());
    assert_text_contains(
        &shared_search_js,
        "\"modules/gcode-parser/docs/reference/modal-groups.html#modal-groups\"",
    );
    assert_text_contains(&shared_search_js, "\"docs/architecture.html#architecture\"");
    assert_text_contains(
        &shared_search_js,
        "\"modules/hmi/docs/operator-panels.html#operator-panels\"",
    );

    let elasticlunr_js = output_dir.join("docs").join(assert_has_file_with_prefix(
        &output_dir.join("docs"),
        "elasticlunr-",
        ".min.js",
    ));
    assert_runtime_search_result(
        &elasticlunr_js,
        &output_dir.join("docs/bookshelf-searchindex.js"),
        "https://example.test/docs/index.html",
        "",
        "modal latch witness token",
        "G-code Parser » Grammar » Modal Groups » Modal Groups",
        "https://example.test/modules/gcode-parser/docs/reference/modal-groups.html?highlight=modal%20latch%20witness%20token#modal-groups",
    );
    assert_runtime_search_result(
        &elasticlunr_js,
        &output_dir.join("modules/gcode-parser/docs/bookshelf-searchindex.js"),
        "https://example.test/modules/gcode-parser/docs/reference/modal-groups.html",
        "../",
        "site-wide breadcrumb label audit witness",
        "MetaNC » Architecture » Architecture",
        "https://example.test/docs/architecture.html?highlight=site-wide%20breadcrumb%20label%20audit%20witness#architecture",
    );
    assert_runtime_search_result(
        &elasticlunr_js,
        &output_dir.join("docs/bookshelf-searchindex.js"),
        "https://example.test/docs/index.html",
        "",
        "interface regions",
        "HMI » Operator Panels » Operator Panels",
        "https://example.test/modules/hmi/docs/operator-panels.html?highlight=interface%20regions#operator-panels",
    );

    fs::remove_dir_all(&output_dir).expect("temp output directory should be removed");
}

#[test]
fn build_cli_audits_search_cold_load_residual_on_repo_scale_output() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_book"));
    let fixture_root = repo_root.join("tests/fixtures/build-cli/repo-scale");
    let config_path = fixture_root.join("bookshelf.toml");
    let output_dir = make_temp_dir("chunk-020-search-cold-load", &repo_root);

    run_build_cli(&bin, &config_path, &output_dir);

    let parser_page_html = assert_read_to_string(
        output_dir.join("modules/gcode-parser/docs/reference/modal-groups.html"),
    );
    let local_search_index = extract_inline_searchindex_path(&parser_page_html);
    let page_path_to_root = extract_inline_path_to_root(&parser_page_html);
    assert_text_contains(
        &parser_page_html,
        &format!("window.path_to_searchindex_js = \"{local_search_index}\""),
    );
    assert_eq!(page_path_to_root, "../");

    let searcher_name = assert_has_file_with_prefix(
        &output_dir.join("modules/gcode-parser/docs"),
        "searcher-",
        ".js",
    );
    assert_script_order(&parser_page_html, &searcher_name, "documentation-search.js");

    let search_override = assert_single_file_named_recursive(
        &output_dir.join("modules/gcode-parser/docs"),
        "documentation-search.js",
    );
    assert_file_contains(
        search_override,
        "window.path_to_searchindex_js = metadata.searchIndexTarget;",
    );
    assert_text_contains(
        &parser_page_html,
        "\"searchIndexTarget\":\"../bookshelf-searchindex.js\"",
    );
    assert_text_contains(
        &parser_page_html,
        "\"documentationIndexTarget\":\"../../../../index.html\"",
    );

    let audit = run_search_cold_load_audit(
        &output_dir.join("modules/gcode-parser/docs/reference/modal-groups.html"),
        "https://example.test/modules/gcode-parser/docs/reference/modal-groups.html?search=modal%20latch%20witness%20token",
    );
    assert!(
        audit.script_sequence.iter().any(|entry| entry == "config"),
        "expected emitted page HTML to include the inline search config script"
    );
    assert!(
        audit
            .script_sequence
            .iter()
            .any(|entry| entry.contains("searcher-")),
        "expected emitted page HTML to include the stock searcher script"
    );
    assert!(
        audit
            .script_sequence
            .iter()
            .any(|entry| entry.ends_with("documentation-search.js")),
        "expected emitted page HTML to include the documentation search override script"
    );
    assert_eq!(
        audit.path_to_root.as_deref(),
        Some("../"),
        "expected nested parser page to emit a non-empty path_to_root"
    );
    assert_eq!(
        audit.configured_after_config.as_deref(),
        Some(local_search_index.as_str()),
        "expected emitted page config to seed the local per-book search index path"
    );
    assert_eq!(
        audit.requested_src.as_deref(),
        Some(local_search_index.as_str()),
        "expected initial ?search= cold load to request the page-local search index before the bookshelf override runs"
    );
    assert_eq!(
        audit.requested_id.as_deref(),
        Some("mdbook-search-index"),
        "expected searcher.js to request the initial cold-load search index script"
    );
    assert_eq!(
        audit.requested_configured.as_deref(),
        Some(local_search_index.as_str()),
        "expected the emitted page to remain configured for the local per-book index when searcher.js issues the initial cold-load request"
    );
    assert_eq!(
        audit.configured_after_searcher.as_deref(),
        Some(local_search_index.as_str()),
        "expected searcher.js startup to preserve the page-local search index path until the later override script runs"
    );
    assert_eq!(
        audit.configured_after_override.as_deref(),
        Some("../bookshelf-searchindex.js"),
        "expected the later bookshelf override to switch the nested page to the localized shared search index path"
    );

    fs::remove_dir_all(&output_dir).expect("temp output directory should be removed");
}

fn run_build_cli(bin: &Path, config_path: &Path, output_dir: &Path) {
    let output = Command::new(bin)
        .arg("build")
        .arg(config_path)
        .arg("--dest-dir")
        .arg(output_dir)
        .output()
        .expect("build command should run");

    if !output.status.success() {
        panic!(
            "build command failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn require_mdbook_mermaid() {
    match Command::new("mdbook-mermaid").arg("--version").output() {
        Ok(output) if output.status.success() => {}
        Ok(output) => panic!(
            "mdbook-mermaid test dependency failed: `mdbook-mermaid --version` exited unsuccessfully\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
        Err(err) => panic!("mdbook-mermaid test dependency is unavailable: {err}"),
    }
}

fn require_mdbook_variables() {
    match Command::new("mdbook-variables")
        .arg("supports")
        .arg("html")
        .output()
    {
        Ok(output) if output.status.success() => {}
        Ok(output) => panic!(
            "mdbook-variables test dependency failed: `mdbook-variables supports html` exited unsuccessfully\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
        Err(err) => panic!("mdbook-variables test dependency is unavailable: {err}"),
    }
}

fn assert_exists(path: PathBuf) {
    assert!(path.exists(), "expected {} to exist", path.display());
}

fn assert_has_file_with_prefix(dir: &Path, prefix: &str, suffix: &str) -> String {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|err| panic!("failed to read {}: {err}", dir.display()));
    let matched = entries.filter_map(|entry| entry.ok()).find_map(|entry| {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        (name.starts_with(prefix) && name.ends_with(suffix)).then(|| name.into_owned())
    });

    assert!(
        matched.is_some(),
        "expected {} to contain a file matching {}*{}",
        dir.display(),
        prefix,
        suffix
    );

    matched.expect("matching file should exist")
}

fn assert_single_file_named_recursive(dir: &Path, name: &str) -> PathBuf {
    let mut matches = Vec::new();
    collect_files_named_recursive(dir, name, &mut matches);
    matches.sort();
    assert_eq!(
        matches.len(),
        1,
        "expected {} to contain exactly one file named {}",
        dir.display(),
        name
    );
    matches.pop().expect("matching file should exist")
}

fn assert_single_file_with_prefix_recursive(dir: &Path, prefix: &str, suffix: &str) -> PathBuf {
    let matches = collect_tree_entries(dir)
        .into_iter()
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(prefix) && name.ends_with(suffix))
        })
        .map(|path| dir.join(path))
        .collect::<Vec<_>>();

    assert_eq!(
        matches.len(),
        1,
        "expected {} to contain exactly one file matching {}*{} recursively",
        dir.display(),
        prefix,
        suffix
    );

    matches
        .into_iter()
        .next()
        .expect("matching file should exist")
}

fn assert_file_contains(path: PathBuf, needle: &str) {
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    assert!(
        content.contains(needle),
        "expected {} to contain {:?}",
        path.display(),
        needle
    );
}

fn assert_sidebar_toc_scope(toc_html: &str, expected_labels: &[&str], unexpected_labels: &[&str]) {
    for label in expected_labels {
        assert_text_contains(toc_html, label);
    }
    for label in unexpected_labels {
        assert_text_not_contains(toc_html, label);
    }
}

fn assert_stock_search_contract(book_dir: &Path, page_html: &str) {
    let searchindex_js = assert_has_file_with_prefix(book_dir, "searchindex-", ".js");
    let searcher_js = assert_has_file_with_prefix(book_dir, "searcher-", ".js");
    let elasticlunr_js = assert_has_file_with_prefix(book_dir, "elasticlunr-", ".min.js");
    let mark_js = assert_has_file_with_prefix(book_dir, "mark-", ".min.js");

    assert_text_contains(page_html, "id=\"mdbook-search-toggle\"");
    assert_text_contains(page_html, "id=\"mdbook-search-wrapper\"");
    assert_text_contains(page_html, "window.path_to_searchindex_js");
    assert_text_contains(
        page_html,
        &format!("window.path_to_searchindex_js = \"{searchindex_js}\""),
    );
    assert_text_contains(page_html, &searcher_js);
    assert_text_contains(page_html, &elasticlunr_js);
    assert_text_contains(page_html, &mark_js);
    assert_file_contains(
        book_dir.join(searchindex_js),
        "window.search = Object.assign(window.search, JSON.parse('",
    );
}

fn assert_runtime_toc(
    script_path: &Path,
    page_href: &str,
    path_to_root: &str,
    expected_labels: &[&str],
    expected_active_label: &str,
    expected_active_href: &str,
) {
    let result = run_toc_runtime(script_path, page_href, path_to_root);
    let expected_labels = expected_labels
        .iter()
        .map(|label| (*label).to_string())
        .collect::<Vec<_>>();

    assert_eq!(
        result.labels, expected_labels,
        "unexpected runtime sidebar labels for {page_href}"
    );
    assert_eq!(
        result.active_count, 1,
        "expected exactly one active TOC entry for {page_href}"
    );
    assert_eq!(
        result.active_label.as_deref(),
        Some(expected_active_label),
        "unexpected active TOC label for {page_href}"
    );
    assert_eq!(
        result.active_href.as_deref(),
        Some(expected_active_href),
        "unexpected active TOC href for {page_href}"
    );
}

fn assert_runtime_search_result(
    elasticlunr_path: &Path,
    searchindex_path: &Path,
    page_href: &str,
    path_to_root: &str,
    query: &str,
    expected_breadcrumbs: &str,
    expected_href: &str,
) {
    let result = run_search_runtime(
        elasticlunr_path,
        searchindex_path,
        page_href,
        path_to_root,
        query,
    );

    assert!(
        result.count > 0,
        "expected shared search index to return results for query {:?} from {}",
        query,
        page_href
    );
    assert_eq!(
        result.first_breadcrumbs.as_deref(),
        Some(expected_breadcrumbs),
        "unexpected first search result label for query {:?} from {}",
        query,
        page_href
    );
    assert_eq!(
        result.first_href.as_deref(),
        Some(expected_href),
        "unexpected first search result href for query {:?} from {}",
        query,
        page_href
    );
}

fn run_toc_runtime(script_path: &Path, page_href: &str, path_to_root: &str) -> TocRuntimeResult {
    let output = Command::new("node")
        .arg("-e")
        .arg(TOC_RUNTIME_HARNESS)
        .arg(script_path)
        .arg(page_href)
        .arg(path_to_root)
        .output()
        .expect("node TOC harness should run");

    if !output.status.success() {
        panic!(
            "node TOC harness failed for {}\nstdout:\n{}\nstderr:\n{}",
            script_path.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    parse_toc_runtime_output(&String::from_utf8_lossy(&output.stdout))
}

fn run_search_runtime(
    elasticlunr_path: &Path,
    searchindex_path: &Path,
    page_href: &str,
    path_to_root: &str,
    query: &str,
) -> SearchRuntimeResult {
    let output = Command::new("node")
        .arg("-e")
        .arg(SEARCH_RUNTIME_HARNESS)
        .arg(elasticlunr_path)
        .arg(searchindex_path)
        .arg(page_href)
        .arg(path_to_root)
        .arg(query)
        .output()
        .expect("node search harness should run");

    if !output.status.success() {
        panic!(
            "node search harness failed for {}\nstdout:\n{}\nstderr:\n{}",
            searchindex_path.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    parse_search_runtime_output(&String::from_utf8_lossy(&output.stdout))
}

fn assert_read_to_string(path: PathBuf) -> String {
    fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

fn assert_text_contains(haystack: &str, needle: &str) {
    assert!(
        haystack.contains(needle),
        "expected text to contain {:?}",
        needle
    );
}

fn assert_text_not_contains(haystack: &str, needle: &str) {
    assert!(
        !haystack.contains(needle),
        "expected text not to contain {:?}",
        needle
    );
}

fn assert_documentation_index_category(
    html: &str,
    category_id: &str,
    title: &str,
    expected_books: &[(&str, &str)],
) {
    let section = documentation_index_category_section(html, category_id);
    assert_text_contains(section, &format!("<h2>{title}</h2>"));
    assert_documentation_index_books_in_order(section, expected_books);
}

fn documentation_index_category_section<'a>(html: &'a str, category_id: &str) -> &'a str {
    let marker = format!("<section class=\"documentation-category\" id=\"{category_id}\"");
    let start = html
        .find(&marker)
        .unwrap_or_else(|| panic!("expected documentation index category {category_id:?}"));
    let after_start = start + marker.len();
    let end = html[after_start..]
        .find("<section class=\"documentation-category\"")
        .map(|relative| after_start + relative)
        .unwrap_or(html.len());
    &html[start..end]
}

fn assert_documentation_index_books_in_order(section: &str, expected_books: &[(&str, &str)]) {
    let mut cursor = 0;
    for (title, href) in expected_books {
        let href_marker = format!("href=\"{href}\"");
        let href_offset = section[cursor..]
            .find(&href_marker)
            .unwrap_or_else(|| panic!("expected category section to contain {href_marker:?}"));
        let href_start = cursor + href_offset;
        let next_card = section[href_start + href_marker.len()..]
            .find("<a class=\"documentation-book-card\"")
            .map(|relative| href_start + href_marker.len() + relative)
            .unwrap_or(section.len());
        let book_card = &section[href_start..next_card];
        assert_text_contains(book_card, title);
        cursor = next_card;
    }
}

fn assert_no_authored_root_relative_markdown_links(output_dir: &Path) {
    let html_files = collect_tree_entries(output_dir)
        .into_iter()
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "html")
        })
        .collect::<Vec<_>>();

    for relative_path in html_files {
        let path = output_dir.join(&relative_path);
        let html = assert_read_to_string(path.clone());
        let html = html.replace("<base href=\"/\">", "");
        assert!(
            !html.contains("href=\"/") && !html.contains("src=\"/"),
            "expected {} not to contain authored root-relative href/src attributes",
            path.display()
        );
    }
}

fn assert_script_order(page_html: &str, earlier: &str, later: &str) {
    let earlier_index = page_html
        .find(earlier)
        .unwrap_or_else(|| panic!("expected page HTML to reference {earlier}"));
    let later_index = page_html
        .find(later)
        .unwrap_or_else(|| panic!("expected page HTML to reference {later}"));

    assert!(
        earlier_index < later_index,
        "expected {earlier} to appear before {later} in page HTML"
    );
}

fn extract_inline_searchindex_path(page_html: &str) -> String {
    let (_, rest) = page_html
        .split_once("window.path_to_searchindex_js = \"")
        .expect("page HTML should configure an initial search index path");
    let (path, _) = rest
        .split_once('"')
        .expect("initial search index assignment should terminate");
    path.to_string()
}

fn extract_inline_path_to_root(page_html: &str) -> String {
    let (_, rest) = page_html
        .split_once("const path_to_root = \"")
        .expect("page HTML should configure path_to_root");
    let (path_to_root, _) = rest
        .split_once('"')
        .expect("path_to_root assignment should terminate");
    path_to_root.to_string()
}

fn collect_files_named_recursive(dir: &Path, name: &str, matches: &mut Vec<PathBuf>) {
    let mut entries = fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", dir.display()))
        .filter_map(|entry| entry.ok())
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_files_named_recursive(&path, name, matches);
        } else if path.file_name().is_some_and(|file_name| file_name == name) {
            matches.push(path);
        }
    }
}

fn assert_no_file_named_recursive(dir: &Path, name: &str) {
    let mut matches = Vec::new();
    collect_files_named_recursive(dir, name, &mut matches);
    assert!(
        matches.is_empty(),
        "expected no files named {name} under {}, found {matches:?}",
        dir.display()
    );
}

fn collect_tree_entries(root: &Path) -> Vec<PathBuf> {
    let mut entries = Vec::new();
    collect_tree_entries_recursive(root, root, &mut entries);
    entries.sort();
    entries
}

fn collect_tree_entries_recursive(root: &Path, dir: &Path, entries: &mut Vec<PathBuf>) {
    let mut dir_entries = fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", dir.display()))
        .filter_map(|entry| entry.ok())
        .collect::<Vec<_>>();
    dir_entries.sort_by_key(|entry| entry.path());

    for entry in dir_entries {
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .unwrap_or_else(|err| panic!("failed to strip prefix {}: {err}", root.display()));
        entries.push(relative.to_path_buf());
        if path.is_dir() {
            collect_tree_entries_recursive(root, &path, entries);
        }
    }
}

fn without_bookshelf_asset_entries(entries: Vec<PathBuf>) -> Vec<PathBuf> {
    entries
        .into_iter()
        .filter(|entry| entry != Path::new(".mdbook") && !entry.starts_with(".mdbook/bookshelf"))
        .collect()
}

#[derive(Debug, Default, PartialEq, Eq)]
struct TocRuntimeResult {
    active_count: usize,
    active_label: Option<String>,
    active_href: Option<String>,
    labels: Vec<String>,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct SearchRuntimeResult {
    count: usize,
    first_breadcrumbs: Option<String>,
    first_href: Option<String>,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct SearchColdLoadAuditResult {
    script_sequence: Vec<String>,
    path_to_root: Option<String>,
    configured_after_config: Option<String>,
    configured_after_searcher: Option<String>,
    configured_after_override: Option<String>,
    requested_src: Option<String>,
    requested_id: Option<String>,
    requested_configured: Option<String>,
}

fn parse_toc_runtime_output(output: &str) -> TocRuntimeResult {
    let mut result = TocRuntimeResult::default();

    for line in output.lines() {
        if let Some(value) = line.strip_prefix("activeCount=") {
            result.active_count = value
                .parse::<usize>()
                .unwrap_or_else(|err| panic!("invalid TOC active count {:?}: {err}", value));
        } else if let Some(value) = line.strip_prefix("active=") {
            if !value.is_empty() {
                result.active_label = Some(value.to_string());
            }
        } else if let Some(value) = line.strip_prefix("activeHref=") {
            if !value.is_empty() {
                result.active_href = Some(value.to_string());
            }
        } else if let Some(value) = line.strip_prefix("labels=") {
            result.labels = if value.is_empty() {
                Vec::new()
            } else {
                value.split('|').map(|label| label.to_string()).collect()
            };
        }
    }

    result
}

fn parse_search_runtime_output(output: &str) -> SearchRuntimeResult {
    let mut result = SearchRuntimeResult::default();

    for line in output.lines() {
        if let Some(value) = line.strip_prefix("count=") {
            result.count = value
                .parse::<usize>()
                .unwrap_or_else(|err| panic!("invalid search result count {:?}: {err}", value));
        } else if let Some(value) = line.strip_prefix("breadcrumbs=") {
            if !value.is_empty() {
                result.first_breadcrumbs = Some(value.to_string());
            }
        } else if let Some(value) = line.strip_prefix("href=") {
            if !value.is_empty() {
                result.first_href = Some(value.to_string());
            }
        }
    }

    result
}

fn run_search_cold_load_audit(page_html_path: &Path, page_href: &str) -> SearchColdLoadAuditResult {
    let output = Command::new("node")
        .arg("-e")
        .arg(SEARCH_COLD_LOAD_AUDIT_HARNESS)
        .arg(page_html_path)
        .arg(page_href)
        .output()
        .expect("node search cold-load harness should run");

    if !output.status.success() {
        panic!(
            "node search cold-load harness failed for {}\nstdout:\n{}\nstderr:\n{}",
            page_html_path.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    parse_search_cold_load_audit_output(&String::from_utf8_lossy(&output.stdout))
}

fn parse_search_cold_load_audit_output(output: &str) -> SearchColdLoadAuditResult {
    let mut result = SearchColdLoadAuditResult::default();

    for line in output.lines() {
        if let Some(value) = line.strip_prefix("sequence=") {
            result.script_sequence = if value.is_empty() {
                Vec::new()
            } else {
                value.split('|').map(|entry| entry.to_string()).collect()
            };
        } else if let Some(value) = line.strip_prefix("pathToRoot=") {
            if !value.is_empty() {
                result.path_to_root = Some(value.to_string());
            }
        } else if let Some(value) = line.strip_prefix("configuredAfterConfig=") {
            if !value.is_empty() {
                result.configured_after_config = Some(value.to_string());
            }
        } else if let Some(value) = line.strip_prefix("configuredAfterSearcher=") {
            if !value.is_empty() {
                result.configured_after_searcher = Some(value.to_string());
            }
        } else if let Some(value) = line.strip_prefix("configuredAfterOverride=") {
            if !value.is_empty() {
                result.configured_after_override = Some(value.to_string());
            }
        } else if let Some(value) = line.strip_prefix("requested=") {
            if !value.is_empty() {
                result.requested_src = Some(value.to_string());
            }
        } else if let Some(value) = line.strip_prefix("requestedId=") {
            if !value.is_empty() {
                result.requested_id = Some(value.to_string());
            }
        } else if let Some(value) = line.strip_prefix("requestedConfigured=") {
            if !value.is_empty() {
                result.requested_configured = Some(value.to_string());
            }
        }
    }

    result
}

fn make_temp_dir(tag: &str, root: &Path) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let dir = std::env::temp_dir()
        .join("mdbook-bookshelf")
        .join(root.file_name().unwrap_or_default())
        .join(format!("{tag}-{nanos}"));
    fs::create_dir_all(&dir).expect("temp output directory should be created");
    dir
}

fn copy_bookshelf_ui_site_fixture(repo_root: &Path, tag: &str) -> PathBuf {
    let fixture_root = make_temp_dir(tag, repo_root);
    copy_dir_all(&repo_root.join(BOOKSHELF_UI_SITE_FIXTURE), &fixture_root)
        .expect("bookshelf UI site fixture should be copied");
    fixture_root
}

fn write_documentation_index_bookshelf_ui_site_config(config_path: &Path) {
    fs::write(
        config_path,
        r#"[book]
title = "Fixture Core"
description = "Repository-wide onboarding and architecture notes."
language = "en"
src = "docs"

[output.html]
default-theme = "light"
preferred-dark-theme = "ayu"

[bookshelf]
root-book-id = "core"

[[bookshelf.book]]
id = "parser"
title = "Fixture Parser"
description = "Parser-specific reference pages with their own reading order."
src = "modules/parser/docs"

[[bookshelf.book]]
id = "ui"
title = "Fixture UI"
description = "Interface and runtime guides for the UI book."
src = "modules/ui/docs"

[[bookshelf.category]]
title = "Core"
books = ["core"]

[[bookshelf.category]]
title = "Modules"
books = ["parser", "ui"]
"#,
    )
    .expect("temporary bookshelf config should be rewritten");
}

fn write_documentation_index_html_cover_config(config_path: &Path) {
    fs::write(
        config_path,
        r#"[book]
title = "Fixture Core"
description = "Repository-wide onboarding and architecture notes."
language = "en"
src = "docs"

[output.html]
default-theme = "light"
preferred-dark-theme = "ayu"

[bookshelf]
root-book-id = "core"

[bookshelf.root-book]
cover = "docs/index.html"

[[bookshelf.book]]
id = "parser"
title = "Fixture Parser"
description = "Parser-specific reference pages with their own reading order."
src = "modules/parser/docs"

[[bookshelf.book]]
id = "ui"
title = "Fixture UI"
description = "Interface and runtime guides for the UI book."
src = "modules/ui/docs"

[[bookshelf.category]]
title = "Core"
books = ["core"]

[[bookshelf.category]]
title = "Modules"
books = ["parser", "ui"]
"#,
    )
    .expect("HTML cover bookshelf config should be written");
}

fn copy_public_self_contained_example_fixture(repo_root: &Path, tag: &str) -> PathBuf {
    let fixture_root = make_temp_dir(tag, repo_root);
    copy_public_example_dir_all(
        &repo_root.join(PUBLIC_SELF_CONTAINED_EXAMPLE),
        &fixture_root,
    )
    .expect("public self-contained example fixture should be copied");
    fixture_root
}

fn copy_public_example_dir_all(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let destination_path = destination.join(entry.file_name());

        if file_type.is_dir() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if matches!(name.as_ref(), ".mdbook-bookshelf" | ".site" | "book") {
                continue;
            }
            copy_public_example_dir_all(&entry.path(), &destination_path)?;
        } else {
            fs::copy(entry.path(), destination_path)?;
        }
    }
    Ok(())
}

fn copy_dir_all(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let destination_path = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &destination_path)?;
        } else {
            fs::copy(entry.path(), destination_path)?;
        }
    }
    Ok(())
}
