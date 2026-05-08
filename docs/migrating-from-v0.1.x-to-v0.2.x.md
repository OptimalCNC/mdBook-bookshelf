# Migrating From v0.1.x To v0.2.x

This guide is for repositories that already build docs with
`mdbook-bookshelf` v0.1.x and need to move to the v0.2.x configuration model.

The main change is that top-level `[book]` is no longer the root content book.
It is now shared site-level mdBook metadata and defaults. Every visible book,
including the repository-wide book that used to live in `[book]`, must be
declared explicitly in `[[bookshelf.book]]` and assigned to exactly one
`[[bookshelf.category]]`.

## What Changed

- `[book]` is shared site metadata and mdBook defaults, not a visible book.
- The old root book must move into its own `[[bookshelf.book]]` entry.
- Every `[[bookshelf.book]]` requires an `id`, `title`, and `src`.
- Every book ID must appear in exactly one `[[bookshelf.category]]`.
- `[bookshelf].entry-page` is removed and rejected.
- `/index.html` is now the generated documentation index, not a redirect.
- `<root-book-src>/bookshelf.html` is no longer generated.
- `bookshelf.md` is no longer a reserved authored page name.
- `[bookshelf].index-title` optionally controls the generated index title and
  the return button label. It defaults to `Documentation`.
- `[bookshelf].asset-dir` now also stores generated documentation-index source
  under `<asset-dir>/documentation-index/`, so it must not overlap any book
  source directory.
- If `[build].build-dir` is omitted, output now defaults to `site/`.

## Config Migration

Before v0.1.x-style config:

```toml
[book]
title = "MetaNC"
description = "Repository-wide architecture, onboarding, and integration notes."
language = "en"
src = "docs"

[output.html]
default-theme = "light"
preferred-dark-theme = "ayu"

[bookshelf]
entry-page = "bookshelf"
asset-dir = ".mdbook/bookshelf"

[[bookshelf.book]]
title = "G-code Parser"
description = "Grammar, modal semantics, and parser diagnostics."
src = "modules/gcode-parser/docs"

[[bookshelf.book]]
title = "HMI"
description = "Operator workflows and runtime notes for the interface book."
src = "modules/hmi/docs"
```

After v0.2.x config:

```toml
[book]
title = "MetaNC Documentation"
description = "Repository documentation portal."
language = "en"

[output.html]
default-theme = "light"
preferred-dark-theme = "ayu"

[bookshelf]
asset-dir = ".mdbook/bookshelf"
index-title = "MetaNC Docs"

[[bookshelf.book]]
id = "metanc"
title = "MetaNC"
description = "Repository-wide architecture, onboarding, and integration notes."
src = "docs"

[[bookshelf.book]]
id = "gcode-parser"
title = "G-code Parser"
description = "Grammar, modal semantics, and parser diagnostics."
src = "modules/gcode-parser/docs"

[[bookshelf.book]]
id = "hmi"
title = "HMI"
description = "Operator workflows and runtime notes for the interface book."
src = "modules/hmi/docs"

[[bookshelf.category]]
title = "Overview"
books = ["metanc"]

[[bookshelf.category]]
title = "Modules"
books = ["gcode-parser", "hmi"]
```

## Step By Step

1. Keep normal mdBook settings at the top level.

   Preserve `[output.html]`, `[preprocessor.*]`, `[build]`, and shared
   `[book]` defaults such as `language` and `authors`.

2. Change top-level `[book]` to describe the whole documentation portal.

   Remove `src = "docs"` from `[book]` unless you deliberately use it as a
   shared mdBook default. It no longer creates a visible book.

3. Add the former root book as an explicit book.

   Give it a stable ID and copy the old root book title, description, and
   `src` into `[[bookshelf.book]]`.

4. Add IDs to the old child books.

   Book IDs may contain ASCII letters, numbers, `-`, `_`, and `.`. Prefer
   stable IDs because categories and generated metadata refer to them.

5. Add categories.

   Add one or more `[[bookshelf.category]]` tables and list every book ID
   exactly once. The category order controls the generated documentation index
   grouping and order.

6. Remove old root-entry settings.

   Delete `[bookshelf].entry-page`, `root-id`, `root-book-id`, and any
   `[bookshelf.root-book]` table. These fields are rejected by the v0.2.x
   parser.

7. Check the generated routes.

   The documentation index is now `/index.html`. Content books keep their
   source-derived routes, for example `/docs/index.html` and
   `/modules/gcode-parser/docs/index.html`. Update links, tests, and deployment
   checks that expected `/index.html` to redirect or expected
   `/docs/bookshelf.html` to exist.

8. Run the build.

   ```bash
   book build bookshelf.toml
   ```

   If CI expects mdBook's stock `book/` output and the config omits
   `[build].build-dir`, either update CI to use `site/` or set:

   ```toml
   [build]
   build-dir = "book"
   ```

## Common Build Errors

- `missing required key bookshelf.book.id`: add `id` to every
  `[[bookshelf.book]]`.
- `bookshelf must define at least one [[bookshelf.category]]`: add categories.
- `book id '...' must appear in exactly one bookshelf.category`: list each book
  ID once, and only once, across all categories.
- `unknown field 'entry-page'`: remove `[bookshelf].entry-page`; the generated
  documentation index is always the site-root entry page.
- `[bookshelf].asset-dir ... must not overlap book source directory`: keep
  generated assets outside every configured `src` tree, for example
  `.mdbook/bookshelf`.

## Output To Expect

After migration, a typical build produces:

```text
site/
  index.html                         # generated documentation index
  searchindex.js                     # shared site-wide search data
  docs/
    index.html                       # former root book
    bookshelf-searchindex.js
  modules/
    gcode-parser/
      docs/
        index.html
        bookshelf-searchindex.js
```

There should be no generated `docs/bookshelf.html`. Use `/` or `/index.html`
when linking readers back to the documentation index.
