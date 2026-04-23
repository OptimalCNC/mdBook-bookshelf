# mdbook-bookshelf

`mdbook-bookshelf` is an mdBook-first multi-book site builder. It keeps mdBook
responsible for single-book loading and rendering, while this repo adds the
bookshelf composition layer on top.

The project is under active development. The intended authoring model matters
more than preserving temporary routing seams.

## Quick Start

Build the checked-in self-contained example:

```bash
cargo run --bin book -- build bookshelf/examples/self-contained/bookshelf.toml --dest-dir .tmp/bookshelf-site
```

Serve the same example locally:

```bash
cargo run --bin book -- serve bookshelf/examples/self-contained/bookshelf.toml --dest-dir .tmp/bookshelf-site --hostname 127.0.0.1 --port 3000
```

Then open `http://127.0.0.1:3000`.

Run the test suite:

```bash
cargo test
```

## Config Model

`bookshelf.toml` is the single human-owned site config.

Example:

```toml
[book]
title = "MetaNC"
src = "docs"

[bookshelf]

[[bookshelf.book]]
title = "G-code Parser"
src = "modules/gcode-parser/docs"

[[bookshelf.book]]
title = "HMI"
src = "modules/hmi/docs"
```

Rules:

- The directory containing `bookshelf.toml` is the config root and site root.
- All `src` values are relative to that directory.
- `src` means only the docs directory for that book.
- The canonical summary for a book is `<src>/SUMMARY.md`.
- The stable entry page for a book is `<src>/index.md`.
- If the implementation needs a per-book filesystem root, it should derive it
  internally from `dirname(src)`.
- Do not introduce a separate configurable site root. If a repository wants a
  different root, move `bookshelf.toml`.

In other words, `src = "modules/parser/docs"` means "this book's markdown lives
here". It does not also mean public mount root, book identifier, or link
resolution root.

## Layouts

For a repo-root bookshelf, keep the root book at `docs/` and place additional
books under feature directories with their own `docs/` trees:

```text
my-repo/
  bookshelf.toml
  docs/
    SUMMARY.md
    index.md
  modules/
    parser/
      docs/
        SUMMARY.md
        index.md
    ui/
      docs/
        SUMMARY.md
        index.md
```

For a nested workspace layout, every book can live under its own directory as
long as each `src` still points at that book's `docs/` directory:

```text
my-repo/
  bookshelf.toml
  root-book/
    docs/
      SUMMARY.md
      index.md
  modules/
    child-book/
      docs/
        SUMMARY.md
        index.md
```

Example config for that layout:

```toml
[book]
title = "Root Book"
src = "root-book/docs"

[bookshelf]

[[bookshelf.book]]
title = "Parser"
src = "modules/parser/docs"
```

Concrete examples live in
[`bookshelf/examples/self-contained/`](./bookshelf/examples/self-contained/)
and [`tests/fixtures/input-catalog/`](./tests/fixtures/input-catalog/).

## Rooting And Links

Use one rooting rule everywhere: the parent directory of `bookshelf.toml` is
the site root.

That root is used for:

- resolving `src` values in `bookshelf.toml`
- resolving leading-`/` links in markdown
- interpreting other config-root-relative paths unless a field says otherwise

Authoring rules:

- Use leading `/` for site-root-relative links, especially across books.
- Use `./` or `../` for file-relative links inside one local source tree.
- Author links against markdown source paths, not build-only aliases.

Examples:

```md
See [Parser Root](/modules/gcode-parser/docs/index.md).
See [Sibling Page](./grammar.md).
```

The canonical published URI for an authored page is its site-root-relative
source path with `.md` changed to `.html`.

Examples:

- `docs/index.md` -> `/docs/index.html`
- `modules/gcode-parser/docs/index.md` -> `/modules/gcode-parser/docs/index.html`

This keeps raw markdown, repository structure, and published page identity in
sync for humans and AI agents.

Do not make these canonical in authored markdown:

- stripped child-book routes such as `/modules/gcode-parser/index.html`
- temporary root-book seams such as `books/<root-id>/...`
- generated `.html` links when the authored target is a markdown page

The detailed routing note is
[`bookshelf/01a-routing-and-linking-contract.md`](./bookshelf/01a-routing-and-linking-contract.md).

## Current State

The implemented routing contract is source-derived:

- root-book pages publish under `book.src`, for example `/docs/...`
- child-book pages publish under full child `src`, for example
  `/modules/gcode-parser/docs/...`
- `/` and `/index.html` remain reserved for the synthetic bookshelf landing
  entry and redirect to `<root-src>/bookshelf.html`

Authors should think in terms of source paths under the `bookshelf.toml` root,
not hidden output aliases.

## TODO

- [ ] Cross-book reference
- [ ] Support `mdBook` plugins `mdbook-mermaid`
- [ ] Support `mdBook` plugins `mdbook-variables`
