# Authoring

This document explains how to lay out a documentation portal in source form.

## Required Structure

Each book keeps its own docs tree.

Minimum catalog-build requirements per book:

- a docs directory named by that book's `src`
- `<src>/SUMMARY.md`

Also author `<src>/index.md` as each book's stable entry page. The catalog
builder validates the canonical `<src>/SUMMARY.md`; it does not currently
validate `index.md`. Runtime navigation still assumes `index.html` for
documentation index links and book entry pages.

Example:

```text
my-repo/
  bookshelf.toml
  docs/
    SUMMARY.md
    index.md
    architecture.md
  modules/
    parser/
      docs/
        SUMMARY.md
        index.md
        grammar.md
    ui/
      docs/
        SUMMARY.md
        index.md
        navigation.md
```

## Books

Every visible book is declared once in `[[bookshelf.book]]`, including a
small repository-wide book such as `Example Core`.

Each book keeps its own independent `SUMMARY.md`, has its own `index.md`, is
listed in exactly one `[[bookshelf.category]]`, and appears on the generated
documentation index like any other categorized book.

Do not:

- merge chapters from one book into another book's summary
- duplicate another book's reading order
- invent separate output aliases for books

## Documentation Index

The documentation index is generated at the site root as `index.html`.
It is not authored inside any content book, and it does not reserve a markdown
page inside any content book.

During build, `mdbook-bookshelf` writes managed source files under
`[bookshelf].asset-dir/documentation-index/` and renders that generated source
with mdBook. Do not edit those generated files; change `bookshelf.toml`
categories, book titles, and descriptions instead.

Authored `bookshelf.md` is not reserved by the documentation index.

## Recommended Authoring Style

- keep `index.md` as the stable entry page for each book
- put every book ID in exactly one configured category
- use `description` fields so documentation index entries have useful copy
- use `/...` markdown links for cross-book references
- use `./...` and `../...` links for nearby local pages
- keep each book's reading order authoritative in its own `SUMMARY.md`

## Layout Variants

A repository-wide book does not have to live at `docs/`, but it does need its
own docs directory just like every other book.

This is also valid:

```text
my-repo/
  bookshelf.toml
  core/
    docs/
      SUMMARY.md
      index.md
  modules/
    parser/
      docs/
        SUMMARY.md
        index.md
```

With config:

```toml
[book]
title = "Example Documentation"

[bookshelf]

[[bookshelf.book]]
id = "core"
title = "Core"
src = "core/docs"

[[bookshelf.book]]
id = "parser"
title = "Parser"
src = "modules/parser/docs"

[[bookshelf.category]]
title = "Overview"
books = ["core"]

[[bookshelf.category]]
title = "Reference"
books = ["parser"]
```

## Generated Output To Expect

Given the layout above:

- the documentation index publishes at `/index.html`
- the core book entry page publishes at `/core/docs/index.html`
- the parser entry page publishes at `/modules/parser/docs/index.html`

The published URL layout stays aligned with the source tree.
