# mdbook-bookshelf

`mdbook-bookshelf` is an mdBook-first multi-book site builder. It keeps mdBook
responsible for single-book loading and rendering, while this repo adds the
bookshelf composition layer on top.

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

Suggested documentation layouts:

For a repo-root bookshelf, define the root book with top-level mdBook `[book]`
metadata and keep its source at `docs/`. Place additional books under feature
directories, each with its own `docs/` tree:

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
long as the root book's top-level `book.src` points at that directory:

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

The root book identity is bookshelf-only:

```toml
[book]
title = "Root Book"
src = "docs"

[bookshelf]
root-id = "root"

[[bookshelf.book]]
title = "Parser"
src = "modules/parser/docs"
```

Child `src` values are mdBook-native and relative to the shared
`bookshelf.toml` directory. Each book's canonical summary is read from
`<src>/SUMMARY.md`, and `<src>/index.md` remains its entry page. Concrete
examples live in
[`bookshelf/examples/self-contained/`](./bookshelf/examples/self-contained/)
and [`tests/fixtures/input-catalog/`](./tests/fixtures/input-catalog/).

`root-id` remains a temporary bookshelf-only seam for the root book's routed
output under `books/<root-id>/...`, and that key is reserved from child
source-derived mounts.

Run the test suite:

```bash
cargo test
```

## TODO

- [ ] Cross-book reference
- [ ] Support `mdBook` plugins `mdbook-mermaid`
- [ ] Support `mdBook` plugins `mdbook-variables`
