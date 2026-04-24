# Configuration

This document explains how to write `bookshelf.toml`.

## Config Model

`bookshelf.toml` is a stock mdBook `Config` with one extra table:

- top-level mdBook config stays stock
- `[book]` is the root book
- `[bookshelf]` enables bookshelf behavior
- `[bookshelf].entry-page` selects where `/` redirects
- `[[bookshelf.book]]` adds child books

The root `[book]` and each `[[bookshelf.book]]` entry use mdBook's own
`BookConfig` fields such as `title`, `description`, `language`, and `src`.

## Minimal Example

```toml
[book]
title = "Example Core"
description = "Repository-wide onboarding and architecture notes."
language = "en"
src = "docs"

[output.html]
default-theme = "light"
preferred-dark-theme = "ayu"

[bookshelf]

[[bookshelf.book]]
title = "Example Parser"
description = "Parser-specific reference pages with their own reading order."
src = "modules/parser/docs"

[[bookshelf.book]]
title = "Example UI"
description = "Interface and runtime guides for the UI book."
src = "modules/ui/docs"
```

## Path Rules

- the parent directory of `bookshelf.toml` is the config root and site root
- every `src` is relative to that directory
- every `src` must point to a docs directory
- the canonical summary for a book is `<src>/SUMMARY.md`
- the stable entry page for a book is `<src>/index.md`

## Entry Page

The site-root entry page is configurable with `[bookshelf].entry-page`.

Valid values:

- `root-book` redirects `/` and `/index.html` to the root book's
  `<src>/index.html`
- `bookshelf` redirects `/` and `/index.html` to the generated
  `<src>/bookshelf.html`

When omitted, `entry-page` defaults to `root-book`.

Current validation rules:

- `book.src` and `bookshelf.book.src` must not be absolute
- they must not contain `..`
- they must not resolve to `.`
- child output roots must not duplicate or overlap the root book or each other
- legacy `bookshelf.root-id` is rejected

Examples of invalid layouts:

- root book at `.`
- child book at `docs` when the root book already uses `docs`
- child book at `docs/api` when the root book already uses `docs`
- child book at `modules/parser/docs/reference` when another child already uses
  `modules/parser/docs`

## Shared mdBook Output Settings

Top-level mdBook output settings remain shared site configuration.

That includes settings such as:

- `output.html.default-theme`
- `output.html.preferred-dark-theme`
- `output.html.additional-css`
- `output.html.additional-js`
- configured `[preprocessor.*]` command plugins

Relative shared asset paths are interpreted from the config root.

## Descriptions

Descriptions are optional, but they are the text shown on the generated
`Bookshelf` page. In practice they are worth filling in for every book.

## What Not To Configure

`mdbook-bookshelf` does not use author-facing fields for:

- root-book route slugs
- separate site roots
- temporary build staging directories
- duplicated child-book summaries inside the root book

Keep the config human-owned and close to stock mdBook.
