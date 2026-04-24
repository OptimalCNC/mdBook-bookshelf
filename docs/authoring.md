# Authoring

This document explains how to lay out a bookshelf site in source form.

## Required Structure

Each book keeps its own docs tree.

Minimum catalog-build requirements per book:

- a docs directory named by that book's `src`
- `<src>/SUMMARY.md`

Also author `<src>/index.md` as each book's stable entry page. The catalog
builder validates the canonical `<src>/SUMMARY.md`; it does not currently
validate `index.md`. Runtime navigation still assumes `index.html` for
site-root redirects, synthetic shelf links, and book entry pages.

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

It behaves like a normal book with one extra generated page:

- the root book still has its own `SUMMARY.md`
- the root book still has its own `index.md`
- `Bookshelf` is generated into that book at `bookshelf.html`
- `Bookshelf` must not be authored as a normal summary chapter

## Child Books

Each child book is declared once in `[[bookshelf.book]]` and keeps its own
independent `SUMMARY.md`.

Do not:

- merge child chapters into the root summary
- duplicate child-book reading order under the root book
- invent separate output aliases for child books

## Reserved Path

`bookshelf.md` is reserved for the synthetic shelf page.

Current implementation rejects any authored chapter whose file name is
`bookshelf.md`, including nested paths such as `guide/bookshelf.md`.

## Recommended Authoring Style

- keep `index.md` as the stable entry page for each book
- use `description` fields so the shelf page has useful copy
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

[[bookshelf.book]]
title = "Parser"
src = "modules/parser/docs"
```

## Generated Output To Expect

Given the layout above:

- the root book entry page publishes at `/root-book/docs/index.html`
- the synthetic shelf page publishes at `/root-book/docs/bookshelf.html`
- the parser entry page publishes at `/modules/parser/docs/index.html`

The published URL layout stays aligned with the source tree.
