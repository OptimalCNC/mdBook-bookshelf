# mdbook-bookshelf

`mdbook-bookshelf` builds multi-book documentation sites on top of stock
mdBook.

Stock mdBook still owns loading and rendering each individual book.
`mdbook-bookshelf` adds the multi-book routing, navigation, search, and
bookshelf UI around that mdBook core.

## Start Here

- Evaluators: start with [docs/index.md](./docs/index.md) for the project
  shape, then [docs/site-behavior.md](./docs/site-behavior.md) for what the
  generated site currently does.
- Site authors: use [docs/configuration.md](./docs/configuration.md),
  [docs/authoring.md](./docs/authoring.md), and
  [docs/linking.md](./docs/linking.md) to set up source trees and links.
- CLI operators: use [docs/operations.md](./docs/operations.md) for current
  `book build` and `book serve` behavior.
- Contributors: use [docs/contributing.md](./docs/contributing.md) for the
  implementation map and mdBook-first guardrails.

## Quick CLI Check

Install the `book` binary from a local checkout:

```bash
cargo install --locked --path .
```

The repository docs configure `mdbook-mermaid`, so install that preprocessor
before building this docs set:

```bash
cargo install mdbook-mermaid
book build bookshelf.toml
book serve bookshelf.toml --port 3000
```
