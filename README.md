# mdbook-bookshelf

[![Publish Status](https://github.com/OptimalCNC/mdBook-bookshelf/actions/workflows/publish.yml/badge.svg)](https://github.com/OptimalCNC/mdBook-bookshelf/actions/workflows/publish.yml)
[![crates.io](https://img.shields.io/crates/v/mdbook-bookshelf.svg)](https://crates.io/crates/mdbook-bookshelf)
[![LICENSE](https://img.shields.io/github/license/OptimalCNC/mdBook-bookshelf.svg)](LICENSE)

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

Install the latest published CLI:

```bash
cargo install mdbook-bookshelf
```

This installs the `book` command.

Install the `book` binary from a local checkout:

```bash
cargo install --path .
```

The repository docs configure `mdbook-mermaid`, so install that preprocessor
before building this docs set:

```bash
cargo install mdbook-mermaid
book build
book serve
```

## Release to crates.io

The repository includes a GitHub Actions workflow at
`.github/workflows/publish.yml` for publishing with crates.io Trusted
Publishing.
