# Routing And Linking Contract

This note defines the canonical URI contract for authored pages and the
authoring rules for links between books.

The goal is simple:

- raw markdown should identify the same page a reader opens after build
- authors should not need to know hidden output aliases to write a correct link
- humans and AI agents should be able to read a markdown link and infer the
  target directly from the repository tree

## Principles

- Treat the directory that contains `bookshelf.toml` as the site root for
  authored links.
- Give every authored markdown page one canonical published URI.
- Derive that canonical URI directly from the markdown source path.
- Do not strip path segments such as `docs/` from authored page routes.
- Do not introduce book-specific route aliases as part of the authoring model.

## Canonical URI Mapping

For authored markdown pages, the published URI is the site-root-relative source
path with only the file extension changed from `.md` to `.html`.

Examples:

- `docs/index.md` -> `/docs/index.html`
- `docs/architecture.md` -> `/docs/architecture.html`
- `modules/gcode-parser/docs/index.md` -> `/modules/gcode-parser/docs/index.html`
- `modules/gcode-parser/docs/reference/modal-groups.md` -> `/modules/gcode-parser/docs/reference/modal-groups.html`

This is the canonical rule for all authored pages, including the root book.

Consequences:

- the root book must not require a special authored-page seam such as
  `books/<root-id>/...`
- child books must not require stripped mounts such as `modules/parser/...`
  when the authored source actually lives at `modules/parser/docs/...`
- one source path means one public page identity

## Synthetic Pages

Synthetic pages are the exception because they do not come from authored
markdown files.

Reserved synthetic routes:

- `/` and `/index.html` are the bookshelf landing entry
- other generated assets or machine-facing files may live at reserved generated
  paths outside authored markdown trees

Synthetic routes must stay separate from the authored-page mapping above.

## Authoring Rules

### Type 1: Site-root-relative markdown path

Use a leading `/` for links that cross book boundaries or when you want a link
to remain stable regardless of the current page's directory.

The target must be written as the authored markdown path relative to the
`bookshelf.toml` directory.

```md
See [G-code Parser](/modules/gcode-parser/docs/index.md) for parser details.
```

This is the default rule for cross-book links.

### Type 2: File-relative markdown path

Use `./` or `../` for links within the same local source tree.

```md
See [sibling page](./sibling.md) for details.
```

Use this for nearby pages. Do not rely on `../` traversal as the main
cross-book authoring style when a stable site-root-relative link is clearer.

### Type 3: Variables as path macros

Variables are optional ergonomics, not a different routing model.

If `mdbook-variables` is used, variables must expand to authored source paths,
not to generated `.html` paths and not to hidden book aliases.

```toml
[preprocessor.variables.variables]
ParserDocs = "/modules/gcode-parser/docs"
```

```md
See [parser root]({{ParserDocs}}/index.md) for the entry page.
```

This is acceptable because the expanded value still names the authored target
unambiguously.

## Disallowed Shortcuts

Do not make these part of the canonical authoring contract:

- extensionless book aliases such as `/nrt/gcode_parser`
- generated-path links written directly as `.html`
- links that depend on stripping `docs/` from the source path
- links that depend on a temporary root-book routing seam

Those forms describe implementation details or aliases, not the authored page
identity.

## Link Resolution Rule

When building links:

1. Leave external URLs and pure fragment links unchanged.
2. Resolve `/...` against the site root, which is the `bookshelf.toml`
   directory.
3. Resolve `./...` and `../...` against the current markdown file.
4. If the resolved target is an authored markdown page inside the bookshelf
   site, emit the same path with `.html`.
5. If the resolved target is a published non-markdown asset, keep its published
   asset path unchanged.
6. Reject links that resolve outside the site root.

## Why This Contract

The alternative is to make authors write links against build-only route aliases.
That is the wrong abstraction.

For example, if raw markdown links to `/modules/parser/docs/index.md`, readers,
authors, and tools all infer the parser book entry page under that source tree.
If the built site instead publishes the page at `/modules/parser/index.html`,
the build has changed the page identity. That mismatch is exactly what makes
cross-book references brittle.

The correct contract is therefore:

- authored links name authored pages
- published URIs preserve that identity
- generated aliases, if they exist at all, are secondary and never canonical
