# Documentation Source

This directory is the source tree for the project's own `mdbook-bookshelf`
documentation.

After installing `book`, install the Mermaid preprocessor configured by the
repository root `bookshelf.toml`, then build it from the repository root with:

```bash
cargo install mdbook-mermaid
book build bookshelf.toml --dest-dir .tmp/project-docs-site
```

The built book entry page is [index.md](./index.md).

## Read This First

- [index.md](./index.md) gives the overview, quick start, and doc map
- [site-behavior.md](./site-behavior.md) explains what readers see in a built
  site
- [linking.md](./linking.md) explains how authored links resolve
- [configuration.md](./configuration.md) explains how to write
  `bookshelf.toml`
- [authoring.md](./authoring.md) explains source layout, `SUMMARY.md`, and the
  generated documentation index
- [examples.md](./examples.md) points at the example tree and repo-scale fixture
- [contributing.md](./contributing.md) explains how the implementation is put
  together and how it stays mdBook-first
