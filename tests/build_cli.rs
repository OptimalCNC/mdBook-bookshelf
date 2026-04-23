use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const BREADCRUMB_RUNTIME_HARNESS: &str = r##"
const fs = require("node:fs");

const [scriptPath, pageHref, pathToRoot] = process.argv.slice(1);
const pageUrl = new URL(pageHref);
let existingBreadcrumb = null;

const main = {
  prepended: [],
  prepend(node) {
    this.prepended.unshift(node);
    if (node.id) {
      existingBreadcrumb = node;
    }
  },
};

globalThis.document = {
  querySelector(selector) {
    return selector === "#mdbook-content main" ? main : null;
  },
  getElementById(id) {
    return existingBreadcrumb && existingBreadcrumb.id === id ? existingBreadcrumb : null;
  },
  createElement(tagName) {
    return {
      tagName: String(tagName).toUpperCase(),
      id: "",
      className: "",
      textContent: "",
      attributes: {},
      setAttribute(name, value) {
        this.attributes[String(name)] = String(value);
      },
    };
  },
};

globalThis.window = {
  location: {
    href: pageHref,
    pathname: pageUrl.pathname,
  },
};
globalThis.path_to_root = pathToRoot;

const script = fs.readFileSync(scriptPath, "utf8");
eval(script);

const node = main.prepended[0];
const lines = [`count=${main.prepended.length}`];
if (node) {
  lines.push(`tag=${node.tagName}`);
  lines.push(`id=${node.id}`);
  lines.push(`class=${node.className}`);
  lines.push(`text=${node.textContent}`);
  lines.push(`data=${node.attributes["data-bookshelf-breadcrumb"] || ""}`);
  lines.push(`aria=${node.attributes["aria-label"] || ""}`);
}
process.stdout.write(lines.join("\n"));
"##;

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
  if (src.endsWith("bookshelf-search.js")) {
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
    } else if (script.label.endsWith("bookshelf-search.js")) {
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
fn build_cli_emits_bookshelf_ui_assets_without_fixture_residue() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_book"));
    let fixture_root = repo_root.join("bookshelf/handoffs/examples/self-contained");
    let config_path = repo_root.join("bookshelf/handoffs/examples/self-contained/bookshelf.toml");
    let output_dir = make_temp_dir("chunk-011-build-cli", &repo_root);
    let fixture_entries_before = without_bookshelf_ui_entries(collect_tree_entries(&fixture_root));

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
    assert_text_contains(
        &root_entry_html,
        "http-equiv=\"refresh\" content=\"0; url=books/meta/bookshelf.html\"",
    );
    assert_text_contains(
        &root_entry_html,
        "window.location.replace(\"books/meta/bookshelf.html\")",
    );
    assert_text_contains(&root_entry_html, "href=\"books/meta/bookshelf.html\"");
    assert_text_not_contains(&root_entry_html, "bookshelf-card__link");
    assert_text_not_contains(&root_entry_html, "Example Core");

    let bookshelf_html = assert_read_to_string(output_dir.join("books/meta/bookshelf.html"));
    assert_text_contains(&bookshelf_html, "<h1 id=\"bookshelf\">");
    assert_text_contains(&bookshelf_html, "Choose a book to enter its root page.");
    assert_text_contains(&bookshelf_html, "Example Core");
    assert_text_contains(
        &bookshelf_html,
        "Repository-wide onboarding and architecture notes.",
    );
    assert_text_contains(&bookshelf_html, "Example Parser");
    assert_text_contains(
        &bookshelf_html,
        "Parser-specific reference pages with their own reading order.",
    );
    assert_text_contains(&bookshelf_html, "Example UI");
    assert_text_contains(
        &bookshelf_html,
        "Interface and runtime guides for the UI book.",
    );
    assert_text_contains(&bookshelf_html, "href=\"index.html\">Example Core</a>");
    assert_text_contains(
        &bookshelf_html,
        "href=\"../parser/index.html\">Example Parser</a>",
    );
    assert_text_contains(&bookshelf_html, "href=\"../ui/index.html\">Example UI</a>");
    assert_text_not_contains(&bookshelf_html, "href=\"bookshelf.html\">Example Core</a>");

    let root_index_html = assert_read_to_string(output_dir.join("books/meta/index.html"));
    assert_text_contains(
        &root_index_html,
        "<title>Example Core - Example Core</title>",
    );
    assert_text_contains(
        &root_index_html,
        "Repository-wide onboarding and architecture guidance for the example project.",
    );
    assert_text_contains(&root_index_html, "href=\"onboarding.html\"");
    assert_text_contains(&root_index_html, "bookshelf-breadcrumb.css");
    assert_text_contains(&root_index_html, "bookshelf-breadcrumb.js");
    assert_text_contains(&root_index_html, "bookshelf-return.css");
    assert_text_contains(&root_index_html, "bookshelf-return.js");
    assert_stock_search_contract(&output_dir.join("books/meta"), &root_index_html);
    assert_text_not_contains(&root_index_html, "Choose a book to enter its root page.");
    assert_text_not_contains(
        &root_index_html,
        "Parser-specific reference pages with their own reading order.",
    );

    let parser_index_html = assert_read_to_string(output_dir.join("books/parser/index.html"));
    assert_text_contains(&parser_index_html, "bookshelf-breadcrumb.css");
    assert_text_contains(&parser_index_html, "bookshelf-breadcrumb.js");
    assert_text_contains(&parser_index_html, "bookshelf-return.css");
    assert_text_contains(&parser_index_html, "bookshelf-return.js");
    assert_stock_search_contract(&output_dir.join("books/parser"), &parser_index_html);
    let architecture_html = assert_read_to_string(output_dir.join("books/meta/architecture.html"));
    assert_text_contains(&architecture_html, "bookshelf-breadcrumb.css");
    assert_text_contains(&architecture_html, "bookshelf-breadcrumb.js");
    let grammar_html = assert_read_to_string(output_dir.join("books/parser/grammar.html"));
    assert_text_contains(&grammar_html, "bookshelf-breadcrumb.css");
    assert_text_contains(&grammar_html, "bookshelf-breadcrumb.js");

    let root_toc_html = assert_read_to_string(output_dir.join("books/meta/toc.html"));
    assert_sidebar_toc_scope(
        &root_toc_html,
        &["Example Core", "Onboarding", "Architecture", "Bookshelf"],
        &["Example Parser", "Grammar", "Example UI", "Navigation"],
    );
    assert_root_bookshelf_affix(&root_toc_html);

    assert_exists(output_dir.join("books/meta/index.html"));
    assert_exists(output_dir.join("books/meta/bookshelf.html"));
    assert_exists(output_dir.join("books/meta/architecture.html"));
    assert_exists(output_dir.join("books/meta/onboarding.html"));
    assert_exists(output_dir.join("books/meta/toc.html"));
    assert_has_file_with_prefix(&output_dir.join("books/meta"), "book-", ".js");
    assert_has_file_with_prefix(&output_dir.join("books/meta"), "toc-", ".js");
    let root_return_script =
        assert_single_file_named_recursive(&output_dir.join("books/meta"), "bookshelf-return.js");
    let root_return_css =
        assert_single_file_named_recursive(&output_dir.join("books/meta"), "bookshelf-return.css");
    let root_breadcrumb_script = assert_single_file_named_recursive(
        &output_dir.join("books/meta"),
        "bookshelf-breadcrumb.js",
    );
    let root_breadcrumb_css = assert_single_file_named_recursive(
        &output_dir.join("books/meta"),
        "bookshelf-breadcrumb.css",
    );
    assert_file_contains(
        root_return_script.clone(),
        "const bookshelfTarget = \"bookshelf.html\";",
    );
    assert_file_contains(
        root_return_script.clone(),
        "link.textContent = \"Bookshelf\";",
    );
    assert_file_contains(
        root_return_script.clone(),
        "document.querySelector(\"#mdbook-menu-bar .right-buttons\")",
    );
    assert_file_contains(root_return_script, "currentPage === \"bookshelf.html\"");
    assert_file_contains(root_return_css, ".bookshelf-return-link");
    assert_runtime_breadcrumb(
        &root_breadcrumb_script,
        "https://example.test/books/meta/architecture.html",
        "Example Core / Architecture",
    );
    assert_runtime_breadcrumb_absent(
        &root_breadcrumb_script,
        "https://example.test/books/meta/bookshelf.html",
    );
    assert_runtime_breadcrumb_absent(
        &root_breadcrumb_script,
        "https://example.test/books/meta/print.html",
    );
    assert_runtime_breadcrumb_absent(
        &root_breadcrumb_script,
        "https://example.test/books/meta/404.html",
    );
    assert_runtime_breadcrumb_absent(
        &root_breadcrumb_script,
        "https://example.test/books/meta/toc.html",
    );
    assert_file_contains(root_breadcrumb_css, ".bookshelf-breadcrumb");
    assert_exists(output_dir.join("books/parser/index.html"));
    assert_exists(output_dir.join("books/parser/grammar.html"));
    assert_exists(output_dir.join("books/parser/toc.html"));
    assert_has_file_with_prefix(&output_dir.join("books/parser"), "book-", ".js");
    let parser_toc_html = assert_read_to_string(output_dir.join("books/parser/toc.html"));
    assert_sidebar_toc_scope(
        &parser_toc_html,
        &["Example Parser", "Grammar", "Runtime"],
        &[
            "Example Core",
            "Bookshelf",
            "Example UI",
            "Navigation",
            "Diagnostics",
        ],
    );
    let parser_toc_script = output_dir
        .join("books/parser")
        .join(assert_has_file_with_prefix(
            &output_dir.join("books/parser"),
            "toc-",
            ".js",
        ));
    let parser_return_script =
        assert_single_file_named_recursive(&output_dir.join("books/parser"), "bookshelf-return.js");
    let parser_return_css = assert_single_file_named_recursive(
        &output_dir.join("books/parser"),
        "bookshelf-return.css",
    );
    let parser_breadcrumb_script = assert_single_file_named_recursive(
        &output_dir.join("books/parser"),
        "bookshelf-breadcrumb.js",
    );
    let parser_breadcrumb_css = assert_single_file_named_recursive(
        &output_dir.join("books/parser"),
        "bookshelf-breadcrumb.css",
    );
    assert_file_contains(
        parser_return_script.clone(),
        "const bookshelfTarget = \"../meta/bookshelf.html\";",
    );
    assert_file_contains(parser_return_script, "link.rel = \"up\";");
    assert_file_contains(parser_return_css, ".bookshelf-return-link");
    assert_runtime_breadcrumb(
        &parser_breadcrumb_script,
        "https://example.test/books/parser/grammar.html",
        "Example Parser / Grammar",
    );
    assert_file_contains(parser_breadcrumb_css, ".bookshelf-breadcrumb");
    assert_runtime_toc(
        &parser_toc_script,
        "https://example.test/books/parser/grammar.html#deep-link",
        "",
        &["Example Parser", "Grammar", "Runtime"],
        "Grammar",
        "https://example.test/books/parser/grammar.html",
    );

    assert_exists(output_dir.join("books/ui/index.html"));
    assert_exists(output_dir.join("books/ui/navigation.html"));
    assert_exists(output_dir.join("books/ui/toc.html"));
    let ui_toc_html = assert_read_to_string(output_dir.join("books/ui/toc.html"));
    assert_sidebar_toc_scope(
        &ui_toc_html,
        &["Example UI", "Navigation", "Diagnostics"],
        &[
            "Example Core",
            "Bookshelf",
            "Example Parser",
            "Grammar",
            "Runtime",
        ],
    );
    let ui_toc_script = output_dir
        .join("books/ui")
        .join(assert_has_file_with_prefix(
            &output_dir.join("books/ui"),
            "toc-",
            ".js",
        ));
    assert_runtime_toc(
        &ui_toc_script,
        "https://example.test/books/ui/navigation.html?from=deep#section",
        "",
        &["Example UI", "Navigation", "Diagnostics"],
        "Navigation",
        "https://example.test/books/ui/navigation.html",
    );
    assert_text_not_contains(&bookshelf_html, "data-bookshelf-breadcrumb");
    assert!(!fixture_root.join(".mdbook-bookshelf").exists());
    assert_eq!(collect_tree_entries(&fixture_root), fixture_entries_before);

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

    let root_css =
        assert_has_file_with_prefix(&output_dir.join("books/root/shared"), "site-", ".css");
    let child_css_path =
        assert_single_file_with_prefix_recursive(&output_dir.join("books/child"), "site-", ".css");
    let child_css = child_css_path
        .strip_prefix(output_dir.join("books/child"))
        .expect("child css should be emitted under child book output")
        .to_string_lossy()
        .replace('\\', "/");

    assert_file_contains(
        output_dir.join("books/root/index.html"),
        &format!("shared/{root_css}"),
    );
    assert_file_contains(output_dir.join("books/child/index.html"), &child_css);
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
fn build_cli_rejects_shared_relative_input_404_for_child_roots() {
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
root-id = "root"

[[bookshelf.book]]
id = "child"
root = "modules/child"
title = "Child Book"
src = "docs"
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

    let output = Command::new(&bin)
        .arg("build")
        .arg(&config_path)
        .arg("--dest-dir")
        .arg(&output_dir)
        .output()
        .expect("build command should run");

    assert!(
        !output.status.success(),
        "build should fail when shared relative input-404 is reused across child roots"
    );
    assert_text_contains(
        &String::from_utf8_lossy(&output.stderr),
        "book 'child' failed to stage shared mdBook output assets",
    );

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
root-id = "root"

[[bookshelf.book]]
id = "child"
root = "modules/child"
title = "Child Book"
src = "docs"
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

    assert!(!output_dir.join("books/child/404.html").exists());

    fs::remove_dir_all(&fixture_root).expect("temp fixture directory should be removed");
    fs::remove_dir_all(&output_dir).expect("temp output directory should be removed");
}

#[test]
fn build_cli_uses_shared_site_wide_search_index() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_book"));
    let config_path = repo_root.join("bookshelf/handoffs/examples/self-contained/bookshelf.toml");
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
    assert_text_contains(&shared_search_js, "\"../parser/grammar.html#grammar\"");
    assert_text_contains(
        &shared_search_js,
        "\"../meta/architecture.html#architecture\"",
    );

    let root_index_html = assert_read_to_string(output_dir.join("books/meta/index.html"));
    let parser_grammar_html = assert_read_to_string(output_dir.join("books/parser/grammar.html"));
    assert_text_contains(&root_index_html, "bookshelf-search.js");
    assert_text_contains(&parser_grammar_html, "bookshelf-search.js");

    let root_search_override =
        assert_single_file_named_recursive(&output_dir.join("books/meta"), "bookshelf-search.js");
    let parser_search_override =
        assert_single_file_named_recursive(&output_dir.join("books/parser"), "bookshelf-search.js");
    assert_file_contains(
        root_search_override,
        "window.path_to_searchindex_js = `${rootPath}../../searchindex.js`;",
    );
    assert_file_contains(
        parser_search_override,
        "window.path_to_searchindex_js = `${rootPath}../../searchindex.js`;",
    );

    let elasticlunr_js = output_dir
        .join("books/meta")
        .join(assert_has_file_with_prefix(
            &output_dir.join("books/meta"),
            "elasticlunr-",
            ".min.js",
        ));

    assert_runtime_search_result(
        &elasticlunr_js,
        &shared_search_path,
        "https://example.test/books/meta/index.html",
        "",
        "concern sidebar",
        "Example Parser » Grammar » Grammar",
        "https://example.test/books/parser/grammar.html?highlight=concern%20sidebar#grammar",
    );
    assert_runtime_search_result(
        &elasticlunr_js,
        &shared_search_path,
        "https://example.test/books/parser/grammar.html",
        "",
        "scoped label",
        "Example Core » Architecture » Architecture",
        "https://example.test/books/meta/architecture.html?highlight=scoped%20label#architecture",
    );

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

    let root_entry_html = assert_read_to_string(output_dir.join("index.html"));
    assert_text_contains(
        &root_entry_html,
        "http-equiv=\"refresh\" content=\"0; url=books/metanc/bookshelf.html\"",
    );
    assert_text_contains(
        &root_entry_html,
        "window.location.replace(\"books/metanc/bookshelf.html\")",
    );
    assert_text_contains(&root_entry_html, "href=\"books/metanc/bookshelf.html\"");
    assert!(!output_dir.join("bookshelf.html").exists());

    let bookshelf_html = assert_read_to_string(output_dir.join("books/metanc/bookshelf.html"));
    assert_text_contains(&bookshelf_html, "<h1 id=\"bookshelf\">");
    assert_text_contains(&bookshelf_html, "Choose a book to enter its root page.");
    assert_text_contains(&bookshelf_html, "href=\"index.html\">MetaNC</a>");
    assert_text_contains(
        &bookshelf_html,
        "href=\"../gcode-parser/index.html\">G-code Parser</a>",
    );
    assert_text_contains(&bookshelf_html, "href=\"../hmi/index.html\">HMI</a>");
    assert_text_not_contains(&bookshelf_html, "href=\"bookshelf.html\">MetaNC</a>");
    assert_text_not_contains(&bookshelf_html, "data-bookshelf-breadcrumb");

    let metanc_index_html = assert_read_to_string(output_dir.join("books/metanc/index.html"));
    let architecture_html =
        assert_read_to_string(output_dir.join("books/metanc/architecture.html"));
    let parser_index_html = assert_read_to_string(output_dir.join("books/gcode-parser/index.html"));
    let modal_groups_html =
        assert_read_to_string(output_dir.join("books/gcode-parser/reference/modal-groups.html"));
    let hmi_index_html = assert_read_to_string(output_dir.join("books/hmi/index.html"));

    assert_stock_search_contract(&output_dir.join("books/metanc"), &metanc_index_html);
    assert_stock_search_contract(&output_dir.join("books/gcode-parser"), &parser_index_html);
    assert_stock_search_contract(&output_dir.join("books/hmi"), &hmi_index_html);
    assert_text_contains(&architecture_html, "bookshelf-breadcrumb.css");
    assert_text_contains(&architecture_html, "bookshelf-breadcrumb.js");
    assert_text_contains(&architecture_html, "bookshelf-search.js");
    assert_text_contains(&modal_groups_html, "bookshelf-breadcrumb.css");
    assert_text_contains(&modal_groups_html, "bookshelf-breadcrumb.js");
    assert_text_contains(&modal_groups_html, "bookshelf-search.js");

    let root_toc_html = assert_read_to_string(output_dir.join("books/metanc/toc.html"));
    assert_sidebar_toc_scope(
        &root_toc_html,
        &["MetaNC", "Getting Started", "Architecture", "Bookshelf"],
        &["G-code Parser", "Modal Groups", "HMI", "Operator Panels"],
    );
    assert_root_bookshelf_affix_for(
        &root_toc_html,
        "MetaNC",
        &["Getting Started", "Architecture"],
    );

    let parser_toc_html = assert_read_to_string(output_dir.join("books/gcode-parser/toc.html"));
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

    let hmi_toc_html = assert_read_to_string(output_dir.join("books/hmi/toc.html"));
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

    let metanc_breadcrumb_script = assert_single_file_named_recursive(
        &output_dir.join("books/metanc"),
        "bookshelf-breadcrumb.js",
    );
    assert_runtime_breadcrumb(
        &metanc_breadcrumb_script,
        "https://example.test/books/metanc/architecture.html",
        "MetaNC / Architecture",
    );

    let parser_breadcrumb_script = assert_single_file_named_recursive(
        &output_dir.join("books/gcode-parser"),
        "bookshelf-breadcrumb.js",
    );
    assert_runtime_breadcrumb_with_path_to_root(
        &parser_breadcrumb_script,
        "https://example.test/books/gcode-parser/reference/modal-groups.html",
        "../",
        "G-code Parser / Modal Groups",
    );

    let parser_toc_script =
        output_dir
            .join("books/gcode-parser")
            .join(assert_has_file_with_prefix(
                &output_dir.join("books/gcode-parser"),
                "toc-",
                ".js",
            ));
    assert_runtime_toc(
        &parser_toc_script,
        "https://example.test/books/gcode-parser/reference/modal-groups.html#group-one",
        "../",
        &["G-code Parser", "Grammar", "Modal Groups", "Diagnostics"],
        "Modal Groups",
        "https://example.test/books/gcode-parser/reference/modal-groups.html",
    );

    let shared_search_path = output_dir.join("searchindex.js");
    let shared_search_js = assert_read_to_string(shared_search_path.clone());
    assert_text_contains(
        &shared_search_js,
        "\"../gcode-parser/reference/modal-groups.html#modal-groups\"",
    );
    assert_text_contains(
        &shared_search_js,
        "\"../metanc/architecture.html#architecture\"",
    );
    assert_text_contains(
        &shared_search_js,
        "\"../hmi/operator-panels.html#operator-panels\"",
    );

    let elasticlunr_js = output_dir
        .join("books/metanc")
        .join(assert_has_file_with_prefix(
            &output_dir.join("books/metanc"),
            "elasticlunr-",
            ".min.js",
        ));
    assert_runtime_search_result(
        &elasticlunr_js,
        &shared_search_path,
        "https://example.test/books/metanc/index.html",
        "",
        "modal latch witness token",
        "G-code Parser » Grammar » Modal Groups » Modal Groups",
        "https://example.test/books/gcode-parser/reference/modal-groups.html?highlight=modal%20latch%20witness%20token#modal-groups",
    );
    assert_runtime_search_result(
        &elasticlunr_js,
        &shared_search_path,
        "https://example.test/books/gcode-parser/reference/modal-groups.html",
        "../",
        "site-wide breadcrumb label audit witness",
        "MetaNC » Architecture » Architecture",
        "https://example.test/books/metanc/architecture.html?highlight=site-wide%20breadcrumb%20label%20audit%20witness#architecture",
    );
    assert_runtime_search_result(
        &elasticlunr_js,
        &shared_search_path,
        "https://example.test/books/metanc/index.html",
        "",
        "interface regions",
        "HMI » Operator Panels » Operator Panels",
        "https://example.test/books/hmi/operator-panels.html?highlight=interface%20regions#operator-panels",
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

    let parser_page_html =
        assert_read_to_string(output_dir.join("books/gcode-parser/reference/modal-groups.html"));
    let local_search_index = extract_inline_searchindex_path(&parser_page_html);
    let page_path_to_root = extract_inline_path_to_root(&parser_page_html);
    assert_text_contains(
        &parser_page_html,
        &format!("window.path_to_searchindex_js = \"{local_search_index}\""),
    );
    assert_eq!(page_path_to_root, "../");

    let searcher_name =
        assert_has_file_with_prefix(&output_dir.join("books/gcode-parser"), "searcher-", ".js");
    assert_script_order(&parser_page_html, &searcher_name, "bookshelf-search.js");

    let search_override = assert_single_file_named_recursive(
        &output_dir.join("books/gcode-parser"),
        "bookshelf-search.js",
    );
    assert_file_contains(
        search_override,
        "window.path_to_searchindex_js = `${rootPath}../../searchindex.js`;",
    );

    let audit = run_search_cold_load_audit(
        &output_dir.join("books/gcode-parser/reference/modal-groups.html"),
        "https://example.test/books/gcode-parser/reference/modal-groups.html?search=modal%20latch%20witness%20token",
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
            .any(|entry| entry.ends_with("bookshelf-search.js")),
        "expected emitted page HTML to include the bookshelf search override script"
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
        Some("../../../searchindex.js"),
        "expected the later bookshelf override to switch the nested page to the shared site-wide search index path"
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

fn assert_root_bookshelf_affix(root_toc_html: &str) {
    assert_root_bookshelf_affix_for(
        root_toc_html,
        "Example Core",
        &["Onboarding", "Architecture"],
    );
}

fn assert_root_bookshelf_affix_for(root_toc_html: &str, root_title: &str, chapter_labels: &[&str]) {
    let root_index = root_toc_html
        .find(&format!("1.</strong> {root_title}"))
        .unwrap_or_else(|| panic!("root TOC should contain numbered root-book index entry"));
    let chapter_positions = chapter_labels
        .iter()
        .enumerate()
        .map(|(idx, label)| {
            root_toc_html
                .find(&format!("1.{}.</strong> {label}", idx + 1))
                .unwrap_or_else(|| panic!("root TOC should contain numbered entry for {label}"))
        })
        .collect::<Vec<_>>();
    let bookshelf_link = "<a href=\"bookshelf.html\" target=\"_parent\">Bookshelf</a>";
    let bookshelf = root_toc_html
        .find(bookshelf_link)
        .expect("root TOC should contain unnumbered Bookshelf affix entry");
    let last_link = root_toc_html
        .rfind("<a href=")
        .expect("root TOC should contain sidebar links");

    let mut previous = root_index;
    for position in chapter_positions {
        assert!(
            previous < position && position < bookshelf,
            "expected root-book numbered chapters before trailing Bookshelf affix"
        );
        previous = position;
    }
    assert_eq!(
        last_link, bookshelf,
        "expected Bookshelf affix to be the trailing root-book sidebar entry"
    );
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

fn assert_runtime_breadcrumb(script_path: &Path, page_href: &str, expected_text: &str) {
    assert_runtime_breadcrumb_with_path_to_root(script_path, page_href, "", expected_text);
}

fn assert_runtime_breadcrumb_with_path_to_root(
    script_path: &Path,
    page_href: &str,
    path_to_root: &str,
    expected_text: &str,
) {
    let result = run_breadcrumb_runtime(script_path, page_href, path_to_root);
    assert_eq!(
        result.count, 1,
        "expected exactly one breadcrumb node for {page_href}"
    );
    assert_eq!(result.tag.as_deref(), Some("NAV"));
    assert_eq!(result.id.as_deref(), Some("bookshelf-breadcrumb"));
    assert_eq!(result.class_name.as_deref(), Some("bookshelf-breadcrumb"));
    assert_eq!(result.text.as_deref(), Some(expected_text));
    assert_eq!(result.data_marker.as_deref(), Some("true"));
    assert_eq!(result.aria_label.as_deref(), Some("Breadcrumb"));
}

fn assert_runtime_breadcrumb_absent(script_path: &Path, page_href: &str) {
    let result = run_breadcrumb_runtime(script_path, page_href, "");
    assert_eq!(
        result.count, 0,
        "expected breadcrumb runtime to no-op for {page_href}"
    );
    assert_eq!(result.tag, None);
    assert_eq!(result.id, None);
    assert_eq!(result.class_name, None);
    assert_eq!(result.text, None);
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

fn run_breadcrumb_runtime(
    script_path: &Path,
    page_href: &str,
    path_to_root: &str,
) -> BreadcrumbRuntimeResult {
    let output = Command::new("node")
        .arg("-e")
        .arg(BREADCRUMB_RUNTIME_HARNESS)
        .arg(script_path)
        .arg(page_href)
        .arg(path_to_root)
        .output()
        .expect("node breadcrumb harness should run");

    if !output.status.success() {
        panic!(
            "node breadcrumb harness failed for {}\nstdout:\n{}\nstderr:\n{}",
            script_path.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    parse_breadcrumb_runtime_output(&String::from_utf8_lossy(&output.stdout))
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

fn without_bookshelf_ui_entries(entries: Vec<PathBuf>) -> Vec<PathBuf> {
    entries
        .into_iter()
        .filter(|entry| !entry.starts_with(".mdbook-bookshelf"))
        .collect()
}

#[derive(Debug, Default, PartialEq, Eq)]
struct BreadcrumbRuntimeResult {
    count: usize,
    tag: Option<String>,
    id: Option<String>,
    class_name: Option<String>,
    text: Option<String>,
    data_marker: Option<String>,
    aria_label: Option<String>,
}

fn parse_breadcrumb_runtime_output(output: &str) -> BreadcrumbRuntimeResult {
    let mut result = BreadcrumbRuntimeResult::default();

    for line in output.lines() {
        if let Some(value) = line.strip_prefix("count=") {
            result.count = value
                .parse::<usize>()
                .unwrap_or_else(|err| panic!("invalid breadcrumb count {:?}: {err}", value));
        } else if let Some(value) = line.strip_prefix("tag=") {
            result.tag = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("id=") {
            result.id = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("class=") {
            result.class_name = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("text=") {
            result.text = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("data=") {
            result.data_marker = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("aria=") {
            result.aria_label = Some(value.to_string());
        }
    }

    if result.count == 0 {
        result.tag = None;
        result.id = None;
        result.class_name = None;
        result.text = None;
        result.data_marker = None;
        result.aria_label = None;
    }

    result
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
