# Internals

This page records the asset and root model used by `mdbook-bookshelf`.

## Shared Root

The directory containing `bookshelf.toml` is the mdBook root for every book.
The root book and each `[[bookshelf.book]]` differ only by `book.src`.

For example:

```toml
[book]
title = "Core"
src = "docs"

[[bookshelf.book]]
title = "Parser"
src = "modules/parser/docs"
```

Both books are loaded from the config root. The second book's mdBook config
keeps `book.src = "modules/parser/docs"` instead of rebasing to
`src = "docs"` under `modules/parser`.

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

## Bookshelf Assets

Bookshelf-owned runtime files live under `[bookshelf].asset-dir`, which
defaults to `.mdbook/bookshelf`.

These files are generated source assets, not authored docs. The build writes
the bookshelf UI CSS, return-button JavaScript, and shared-search override
there, then appends those paths to mdBook's `additional-css` and
`additional-js` lists. mdBook copies them into each book output at the same
relative path.

The build also writes a `README.md` file explaining that the directory is
managed by `mdbook-bookshelf`, plus a `.gitignore` file containing `*`. The
ignore file keeps generated runtime files, the generated README, and the
generated `.gitignore` itself out of version control.

`mdbook-bookshelf` intentionally does not provide an `install`-style command,
like `mdbook-mermaid` does, for writing these runtime files ahead of time. The
files are currently stable, but they remain implementation details of the
bookshelf renderer integration. Generating them during build keeps that
boundary clear.

The serve watcher ignores only the configured asset directory and the output
directory, so unrelated `.mdbook` files remain watchable.

## Page Metadata

Some runtime values are page-specific: the current page's relative link back to
the root `Bookshelf` page and the localized shared search index path.

Those values are injected by a small preprocessor as JSON metadata on each
rendered page. The shared runtime JavaScript reads that metadata. This avoids
generating per-book JavaScript files with hardcoded paths.

## Scope

`mdbook-bookshelf` adds bookshelf behavior around stock mdBook: multiple
source trees, a synthetic shelf page, source-root link rewriting, and
site-wide search data.

It should not grow unrelated authoring features. Those belong in standalone
mdBook preprocessors, such as `mdbook-mermaid`, and can be enabled through
normal mdBook configuration.
