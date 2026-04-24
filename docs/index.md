# mdbook-bookshelf

`mdbook-bookshelf` builds one multi-book documentation site on top of stock
mdBook.

It keeps mdBook responsible for loading and rendering each individual book, and
adds a bookshelf layer for multi-book routing, navigation, and search.

## Start By Role

- Evaluators: read this overview, then [Site Behavior](./site-behavior.md) to
  see the generated routes, shelf page, scoped navigation, and search behavior.
- Site authors: start with [Configuration](./configuration.md) and
  [Authoring](./authoring.md), then use [Linking](./linking.md) for local and
  cross-book links.
- CLI operators: use [Operations](./operations.md) for the current `book build`
  and `book serve` commands, flags, output rules, and serve watcher behavior.
- Contributors: use [Contributing](./contributing.md) for the code map,
  pipeline, tests, and mdBook-first guardrails.

## What It Does

- uses one human-owned `bookshelf.toml`
- treats the top-level `[book]` as the root book
- adds child books through `[[bookshelf.book]]`
- publishes authored pages at source-derived URLs
- generates a synthetic root `Bookshelf` page
- keeps sidebars and previous/next navigation scoped to the active book
- exposes site-wide search across all books

## Quick Start

Install the `book` binary from a local checkout:

```bash
cargo install --path .
```

This repository's own docs configure `mdbook-mermaid`, so install that
preprocessor before building this docs set:

```bash
cargo install mdbook-mermaid
book build
```

Serve the same docs locally:

```bash
book serve --port 3000
```

Then open `http://127.0.0.1:3000`.

See [Operations](./operations.md) for command defaults, flags, output
directories, and serve rebuild behavior.

## Config Model

`bookshelf.toml` is a stock mdBook
[`Config`](https://docs.rs/mdbook-driver/latest/mdbook_driver/config/struct.Config.html)
with one extra table:

- top-level mdBook config stays stock
- `[book]` is the root book
- `[bookshelf]` enables bookshelf behavior
- `[[bookshelf.book]]` adds child books

Each `[[bookshelf.book]]` entry is taken directly from mdBook's stock
[`BookConfig`](https://docs.rs/mdbook-driver/latest/mdbook_driver/config/struct.BookConfig.html).

Minimal example:

```toml
[book]
title = "MetaNC"
description = "Repository-wide docs."
src = "docs"

[output.html]
default-theme = "light"

[bookshelf]

[[bookshelf.book]]
title = "G-code Parser"
description = "Parser reference."
src = "modules/gcode-parser/docs"

[[bookshelf.book]]
title = "HMI"
description = "Operator-facing docs."
src = "modules/hmi/docs"
```

Current path contract:

- the directory containing `bookshelf.toml` is the site root
- every `src` is relative to that directory
- every authored markdown page publishes at the same path with `.md` changed to
  `.html`
- `/...` authored links resolve from the site root and are rewritten during
  build
- cross-book links can be authored against site root such as
  `/modules/gcode-parser/docs/index.md`

See [Configuration](./configuration.md) for the full config rules.

## Documentation

- [Site Behavior](./site-behavior.md) describes what readers see in the built
  site
- [Configuration](./configuration.md) covers `bookshelf.toml`
- [Operations](./operations.md) documents current `book build` and `book serve`
  behavior
- [Authoring](./authoring.md) covers layout, `SUMMARY.md`, and the generated
  `Bookshelf` page
- [Linking](./linking.md) explains how to write cross-book and local links
- [Examples](./examples.md) points at the example tree and repo-scale fixture
- [Contributing](./contributing.md) explains how the implementation is put
  together
