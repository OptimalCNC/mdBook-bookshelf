# Configuration

This document explains how to write `bookshelf.toml`.

## Config Model

`bookshelf.toml` is a stock mdBook `Config` with one extra table:

- top-level mdBook config stays stock
- `[book]` is the root book
- `[bookshelf]` is required and enables documentation portal behavior
- `[bookshelf].root-book-id` gives the root book its catalog ID
- `[bookshelf].asset-dir` selects the tool-owned generated asset directory
- `[[bookshelf.book]]` adds child books
- `[[bookshelf.category]]` groups every book for the documentation index

The root `[book]` uses mdBook's own `BookConfig` fields. Each
`[[bookshelf.book]]` entry uses those same mdBook fields, such as `title`,
`description`, `language`, and `src`, plus documentation-index metadata such as
`id` and `cover`. Child book entries are applied as overrides on top of the
root `[book]` configuration, then their `src` is kept as the full
config-root-relative source path.

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
root-book-id = "core"
asset-dir = ".mdbook/bookshelf"

[[bookshelf.book]]
id = "parser"
title = "Parser"
description = "Parser-specific reference pages."
src = "modules/parser/docs"

[[bookshelf.category]]
title = "Start Here"
books = ["core"]

[[bookshelf.category]]
title = "Reference"
books = ["parser"]
```

## Validation Rules

These rules are enforced when `mdbook-bookshelf` parses `bookshelf.toml`:

- `[bookshelf]` is required, even when there are no child books
- `[bookshelf].root-book-id` is required
- `book.title` is required and must not be empty
- every book ID must be unique and must not be `.` or `..`
- every `[[bookshelf.book]]` requires a unique `id`
- each `bookshelf.book.title` is required and must not be empty
- every category title must be non-empty
- every category must list at least one book
- category book references must point at known book IDs
- every book must appear in exactly one category
- `book.src` and `bookshelf.book.src` must not be empty
- `book.src` and `bookshelf.book.src` must be relative paths
- `book.src` and `bookshelf.book.src` must not contain `..`
- `book.src` and `bookshelf.book.src` must not resolve to `.`
- `book.src` and `bookshelf.book.src` must not name `SUMMARY.md`
- `bookshelf.asset-dir` must not be empty
- `bookshelf.asset-dir` must be relative to the config root
- `bookshelf.asset-dir` must not contain `..`
- `bookshelf.asset-dir` must name a directory below the config root
- `bookshelf.asset-dir` must not overlap any book source directory
- child book output roots must not duplicate the root book or another child
  book output root
- child book output roots must not overlap the root book or another child book
  output root by nesting one output root inside another
- configured cover paths must be relative to the config root and must not
  contain `..`
- configured cover paths must name files below the config root
- legacy `bookshelf.root-id` is rejected

`./` prefixes are allowed and normalized away, so `./modules/parser/docs` is
stored as `modules/parser/docs`.

## Path Rules

- the parent directory of `bookshelf.toml` is the config root and site root
- every `src` is relative to that directory
- every `src` must point to a docs directory
- the canonical summary for a book is `<src>/SUMMARY.md`
- the stable entry page for a book is `<src>/index.md`

## Asset Directory

`[bookshelf].asset-dir` configures the tool-owned source asset directory used
by `mdbook-bookshelf`.

Default:

```toml
[bookshelf]
asset-dir = ".mdbook/bookshelf"
```

The path is relative to the directory containing `bookshelf.toml`.
`mdbook-bookshelf` writes its runtime CSS and JavaScript there, then appends
those files to `output.html.additional-css` and `output.html.additional-js` for
each book. mdBook copies those files into each book output using the same
relative path.

Treat this directory as generated state. Do not put human-owned assets under
it; use normal `output.html.additional-css` and `output.html.additional-js`
paths for project assets.

The asset directory is rejected if it overlaps any configured book source
directory, because the build rewrites generated files in that directory.

The build writes a `README.md` explaining that the directory is generated, and
a `.gitignore` file containing `*`. That keeps generated runtime files, the
generated README, and the generated `.gitignore` itself out of version control.

Output root conflict terms:

- duplicate output root: two books resolve to the same output directory
- overlapping output root: one book's output directory is nested inside another

Examples of invalid layouts:

- root book at `.`
- child book at `docs` when the root book already uses `docs`
- child book at `docs/api` when the root book already uses `docs`
- child book at `modules/parser/docs/reference` when another child already uses
  `modules/parser/docs`

## Documentation Index Covers

Covers are optional. Configure the root book cover under
`[bookshelf.root-book]`, and configure child book covers directly on
`[[bookshelf.book]]`.

Books without a configured cover render a fallback card in the documentation
index.

```toml
[bookshelf.root-book]
cover = "assets/covers/core.png"

[[bookshelf.book]]
id = "parser"
title = "Parser"
src = "modules/parser/docs"
cover = "assets/covers/parser.png"
```

Configured cover paths:

- are relative to the directory containing `bookshelf.toml`
- must not contain `..`
- must name a file below the config root
- must not end in `.html` or `.htm`

HTML cover paths are rejected because covers render as images and HTML cover
paths can collide with generated pages.

During build, configured covers are copied to generated output under
`.mdbook/bookshelf/covers/<book-id>/...`.

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
Mermaid can be configured once at the config root.

Relative `output.html.additional-css` and `output.html.additional-js` paths are
resolved from the config root, the directory containing
`bookshelf.toml`. They are not resolved from each child book's `src`
directory.

Every book now uses the config root as its mdBook root while keeping its own
`book.src`, catalog ID, and documentation-index metadata. That means shared
relative mdBook output settings can stay relative to the config root for every
book. There is no `bookshelf-config-assets/` staging directory.

`output.html.input-404` remains stock mdBook behavior: a relative value is
resolved by mdBook for the active book during rendering. Set `input-404 = ""`
to disable custom 404 generation.

## Descriptions

Descriptions are optional, but they are the text shown on documentation index
book cards. In practice they are worth filling in for every book.

## What Not To Configure

`mdbook-bookshelf` does not use author-facing fields for:

- root-book route slugs
- separate site roots
- temporary build staging directories
- breadcrumb UI
- duplicated child-book summaries inside the root book

Keep the config human-owned and close to stock mdBook.
