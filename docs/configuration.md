# Configuration

This document explains how to write `bookshelf.toml`.

## Config Model

`bookshelf.toml` is a stock mdBook `Config` with one extra table:

- top-level mdBook config stays stock
- `[book]` is the root book
- `[bookshelf]` is required and enables bookshelf behavior
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

## Validation Rules

These rules are enforced when `mdbook-bookshelf` parses `bookshelf.toml`:

- `[bookshelf]` is required, even when there are no child books
- `book.title` is required and must not be empty
- each `bookshelf.book.title` is required and must not be empty
- `book.src` and `bookshelf.book.src` must not be empty
- `book.src` and `bookshelf.book.src` must be relative paths
- `book.src` and `bookshelf.book.src` must not contain `..`
- `book.src` and `bookshelf.book.src` must not resolve to `.`
- `book.src` and `bookshelf.book.src` must not name `SUMMARY.md`
- child book output roots must not duplicate the root book or another child
  book output root
- child book output roots must not overlap the root book or another child book
  output root by nesting one output root inside another
- legacy `bookshelf.root-id` is rejected

`./` prefixes are allowed and normalized away, so `./modules/parser/docs` is
stored as `modules/parser/docs`.

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

Output root conflict terms:

- duplicate output root: two books resolve to the same output directory
- overlapping output root: one book's output directory is nested inside another

Examples of invalid layouts:

- root book at `.`
- child book at `docs` when the root book already uses `docs`
- child book at `docs/api` when the root book already uses `docs`
- child book at `modules/parser/docs/reference` when another child already uses
  `modules/parser/docs`

## Shared mdBook Output Settings

Top-level mdBook output settings remain shared site configuration.
`mdbook-bookshelf` applies them to every built book before invoking mdBook.

That includes settings such as:

- `output.html.default-theme`
- `output.html.preferred-dark-theme`
- `output.html.additional-css`
- `output.html.additional-js`
- `output.html.input-404`
- configured `[preprocessor.*]` command plugins

Configured `[preprocessor.*]` command plugins run for each book. Entries in
`output.html.additional-js` are also included in each book, so plugins such as
Mermaid can be configured once at the bookshelf root.

Relative `output.html.additional-css` and `output.html.additional-js` paths are
resolved from the bookshelf config root, the directory containing
`bookshelf.toml`. They are not resolved from each child book's `src`
directory.

Before building a child book whose book root differs from the config root,
`mdbook-bookshelf` stages those configured CSS and JavaScript files under that
child book root in `bookshelf-config-assets/`. The generated child book output
then references the emitted `bookshelf-config-assets/...` copies.

`output.html.input-404` is different: mdBook treats a relative value as an
input page under a single book root. When a shared relative `input-404` would be
reused for a book with a different root, the build is rejected. Set
`input-404 = ""` to disable custom 404 generation for multi-book builds that
cannot share one input page.

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
