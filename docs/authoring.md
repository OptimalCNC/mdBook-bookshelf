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
documentation index cards and book entry pages.

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

## Root Book

The root book is just the top-level `[book]` entry from `bookshelf.toml`.

It behaves like a normal book:

- the root book still has its own `SUMMARY.md`
- the root book still has its own `index.md`
- the root book is listed in exactly one `[[bookshelf.category]]`
- the root book appears on the generated documentation index like any other
  categorized book

## Child Books

Each child book is declared once in `[[bookshelf.book]]` and keeps its own
independent `SUMMARY.md`.

Do not:

- merge child chapters into the root summary
- duplicate child-book reading order under the root book
- invent separate output aliases for child books

## Documentation Index

The documentation index is generated at the site root as `index.html`.
It is not authored inside any book, and it does not reserve a markdown page
inside root or child books.

Authored `bookshelf.md` is not reserved by the documentation index.

## Recommended Authoring Style

- keep `index.md` as the stable entry page for each book
- put every book ID in exactly one configured category
- use `description` fields so documentation index cards have useful copy
- use `/...` markdown links for cross-book references
- use `./...` and `../...` links for nearby local pages
- keep each book's reading order authoritative in its own `SUMMARY.md`

## Layout Variants

The root book does not have to live at `docs/`, but it does need its own docs
directory just like every other book.

This is also valid:

```text
my-repo/
  bookshelf.toml
  root-book/
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
title = "Root Book"
src = "root-book/docs"

[bookshelf]
root-book-id = "root"

[[bookshelf.book]]
id = "parser"
title = "Parser"
src = "modules/parser/docs"

[[bookshelf.category]]
title = "Overview"
books = ["root"]

[[bookshelf.category]]
title = "Reference"
books = ["parser"]
```

## Generated Output To Expect

Given the layout above:

- the documentation index publishes at `/index.html`
- the root book entry page publishes at `/root-book/docs/index.html`
- the parser entry page publishes at `/modules/parser/docs/index.html`

The published URL layout stays aligned with the source tree.
