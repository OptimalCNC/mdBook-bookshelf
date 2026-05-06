# Internals

This page records the documentation index, asset, and root model used by
`mdbook-bookshelf`.

## Shared Root

The directory containing `bookshelf.toml` is the mdBook root for every book.
The root book and each `[[bookshelf.book]]` differ only by `book.src`.

For example:

```toml
[book]
title = "Core"
src = "docs"

[bookshelf]
root-book-id = "core"

[[bookshelf.category]]
id = "guides"
title = "Guides"
books = ["core", "parser"]

[[bookshelf.book]]
id = "parser"
title = "Parser"
src = "modules/parser/docs"
```

Both books are loaded from the config root and shown through explicit
documentation index categories. The second book's mdBook config keeps
`book.src = "modules/parser/docs"` instead of rebasing to `src = "docs"` under
`modules/parser`.

This keeps stock mdBook path resolution consistent for shared settings such as
themes, extra watch directories, and `output.html.additional-js`.

## Additional Assets

mdBook's `output.html.additional-css` and `output.html.additional-js` are
file-based. The configured paths are read from the mdBook root, copied into the
book output, and referenced from rendered HTML.

Because every book shares one mdBook root, `mdbook-bookshelf` does not stage
configured shared assets into child directories. Project-owned assets should
stay in normal config-root-relative paths and be listed in mdBook's standard
`additional-css` or `additional-js` settings.

## Documentation Index

After catalog validation and per-book build setup, `mdbook-bookshelf` writes
the generated documentation index to the site root as `index.html`.

The generated index renders the configured categories and creates one card per
categorized book. Each card links to that book's source-derived
`<src>/index.html` route.

The root book is represented in the same catalog as child books and is rendered
only through the category that references `[bookshelf].root-book-id`.

## Documentation Assets

Documentation runtime files live under `[bookshelf].asset-dir`, which
defaults to `.mdbook/bookshelf`.

These files are generated source assets, not authored docs. The build writes
`documentation-return.css`, `documentation-return.js`, and
`documentation-search.js` there, then appends those paths to mdBook's
`additional-css` and `additional-js` lists. mdBook copies them into each book
output at the same relative path.

The build also writes a `README.md` file explaining that the directory is
managed by `mdbook-bookshelf`, plus a `.gitignore` file containing `*`. The
ignore file keeps generated runtime files, the generated README, and the
generated `.gitignore` itself out of version control.

`mdbook-bookshelf` intentionally does not provide an `install`-style command,
like `mdbook-mermaid` does, for writing these runtime files ahead of time. The
files are currently stable, but they remain implementation details of the
documentation renderer integration. Generating them during build keeps that
boundary clear.

The serve watcher ignores only the configured asset directory and the output
directory, so unrelated `.mdbook` files remain watchable.

## Page Metadata

Some runtime values are page-specific: the current page's relative link back to
the documentation index and the localized shared search index path.

Those values are injected by a small preprocessor as JSON metadata on each
rendered page. The return link uses the `documentationIndexTarget` metadata
field. This avoids generating per-book JavaScript files with hardcoded paths.

## Scope

`mdbook-bookshelf` adds documentation portal behavior around stock mdBook:
multiple source trees, a generated documentation index, source-root link
rewriting, and site-wide search data.

It should not grow unrelated authoring features. Those belong in standalone
mdBook preprocessors, such as `mdbook-mermaid`, and can be enabled through
normal mdBook configuration.
