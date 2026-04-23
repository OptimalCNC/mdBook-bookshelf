# mdbook-bookshelf

`mdbook-bookshelf` is an mdBook-first multi-book site builder. It keeps mdBook
responsible for single-book loading and rendering, while this repo adds the
bookshelf composition layer on top.

## Quick Start

Build the checked-in self-contained example:

```bash
cargo run --bin book -- build bookshelf/handoffs/examples/self-contained/bookshelf.toml --dest-dir .tmp/bookshelf-site
```

Serve the same example locally:

```bash
cargo run --bin book -- serve bookshelf/handoffs/examples/self-contained/bookshelf.toml --dest-dir .tmp/bookshelf-site --hostname 127.0.0.1 --port 3000
```

Then open `http://127.0.0.1:3000`.

Suggested documentation layouts:

For a repo-root bookshelf, keep the root book at `docs/` and place additional
books under feature directories, each with its own `docs/` tree:

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
long as its mdBook source directory is `docs/`:

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

Across both layouts, use `src = ".../docs"` in `[[bookshelf.book]]` entries.
Each book's canonical summary is always read from `<src>/SUMMARY.md`, and
`<src>/index.md` remains its entry page. Concrete examples live in
[`bookshelf/handoffs/examples/self-contained/`](./bookshelf/handoffs/examples/self-contained/)
and [`tests/fixtures/input-catalog/`](./tests/fixtures/input-catalog/).

Run the test suite:

```bash
cargo test
```

## TODO

- [ ] Cross-book reference
- [ ] Support `mdBook` plugins `mdbook-mermaid`
- [ ] Support `mdBook` plugins `mdbook-variables`
