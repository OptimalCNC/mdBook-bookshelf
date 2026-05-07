# mdbook-bookshelf

`mdbook-bookshelf` builds one multi-book documentation site on top of stock
mdBook.

It keeps mdBook responsible for loading and rendering each individual book, and
adds a documentation index layer for multi-book routing, navigation, search,
and return UI.

## Start By Role

- Evaluators: read this overview, then [Site Behavior](./site-behavior.md) to
  see the generated documentation index, scoped navigation, and search
  behavior.
- Site authors: start with [Configuration](./configuration.md) and
  [Authoring](./authoring.md), then use [Linking](./linking.md) for local and
  cross-book links.
- CLI operators: use [Operations](./operations.md) for the current `book build`
  and `book serve` commands, flags, output rules, and serve watcher behavior.
- Contributors: use [Contributing](./contributing.md) for the code map,
  pipeline, tests, and mdBook-first guardrails.

## What It Does

- uses one human-owned `bookshelf.toml`
- keeps top-level mdBook settings as shared site defaults
- declares every visible book through `[[bookshelf.book]]`
- groups books through `[[bookshelf.category]]`
- publishes authored pages at source-derived URLs
- generates a site-root documentation index at `/index.html`
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
book serve
```

Then open the URL printed after `Serving on:`.

See [Operations](./operations.md) for command defaults, flags, output
directories, and serve rebuild behavior.

## Config Model

`bookshelf.toml` is a stock mdBook
[`Config`](https://docs.rs/mdbook-driver/latest/mdbook_driver/config/struct.Config.html)
with one extra table:

- top-level mdBook config stays stock
- `[book]` provides shared site-level mdBook metadata and defaults
- `[bookshelf]` enables documentation portal behavior
- `[bookshelf].asset-dir` configures tool-owned generated runtime assets
- `[bookshelf].index-title` configures the generated index heading and return
  button label
- `[[bookshelf.book]]` declares each visible book
- `[[bookshelf.category]]` groups every book for the documentation index

Each `[[bookshelf.book]]` entry uses mdBook
[`BookConfig`](https://docs.rs/mdbook-driver/latest/mdbook_driver/config/struct.BookConfig.html)
fields such as `title`, `description`, `language`, and `src`, plus the
documentation-index `id`.

Minimal example:

```toml
[book]
title = "MetaNC Documentation"
description = "Site-level docs metadata."

[output.html]
default-theme = "light"

[bookshelf]
asset-dir = ".mdbook/bookshelf"
index-title = "MetaNC Docs"

[[bookshelf.book]]
id = "core"
title = "MetaNC"
description = "Repository-wide docs."
src = "docs"

[[bookshelf.book]]
id = "gcode-parser"
title = "G-code Parser"
description = "Parser reference."
src = "modules/gcode-parser/docs"

[[bookshelf.book]]
id = "hmi"
title = "HMI"
description = "Operator-facing docs."
src = "modules/hmi/docs"

[[bookshelf.category]]
title = "Overview"
books = ["core"]

[[bookshelf.category]]
title = "Modules"
books = ["gcode-parser", "hmi"]
```

Current path contract:

- the directory containing `bookshelf.toml` is the site root
- every `src` is relative to that directory
- the generated site-root `index.html` is the documentation index
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
- [Authoring](./authoring.md) covers layout, `SUMMARY.md`, and book entry pages
- [Linking](./linking.md) explains how to write cross-book and local links
- [Examples](./examples.md) points at the example tree and repo-scale fixture
- [Internals](./internals.md) documents the shared root and asset model
- [Contributing](./contributing.md) explains how the implementation is put
  together
