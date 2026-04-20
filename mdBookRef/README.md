# mdBook Reference Notes

This folder collects implementation-relevant notes from the local stock mdBook
clone in `mdBook-repo/mdBook` and the public API docs for the mdBook library
crates.

These notes are written for the bookshelf feature, not as general mdBook
documentation.

## Files

- `build-render.md`: How stock `mdbook build` reaches `mdbook_driver`, how a
  single book is loaded, and where the HTML renderer encodes single-book
  behavior.
- `serve-watch.md`: How `mdbook serve` and `mdbook watch` work, and which parts
  are reusable for a bookshelf-specific serve loop.
- `mdbook-driver-api.md`: The main public API types in `mdbook_driver`,
  `mdbook_renderer`, and `mdbook_preprocessor`, with notes about what is
  reusable versus what is single-book oriented.

## Main Takeaways

1. Stock mdBook CLI commands are thin wrappers over `mdbook_driver::MDBook`.
2. `MDBook` is strongly centered on one `root`, one `Config`, one loaded
   `Book`, and one canonical `book.toml` plus `SUMMARY.md` pair.
3. Custom preprocessors and backends are real extension seams, but both operate
   on a single already-loaded `Book`.
4. The built-in HTML renderer contains several assumptions that conflict with a
   multi-book bookshelf shell:
   - one global `chapters` list for the sidebar
   - previous and next computed across the full chapter stream
   - first rendered chapter copied to root `index.html`
5. The serve/watch path is easier to reuse than the stock HTML renderer. It is
   basically:
   - build once
   - serve one output directory over HTTP
   - watch source/theme/config files
   - rebuild and broadcast a live-reload message

## Source Set

Local source and guides:

- `mdBook-repo/mdBook/src/main.rs`
- `mdBook-repo/mdBook/src/cmd/build.rs`
- `mdBook-repo/mdBook/src/cmd/serve.rs`
- `mdBook-repo/mdBook/src/cmd/watch.rs`
- `mdBook-repo/mdBook/src/cmd/watch/native.rs`
- `mdBook-repo/mdBook/src/cmd/watch/poller.rs`
- `mdBook-repo/mdBook/src/cmd/command_prelude.rs`
- `mdBook-repo/mdBook/crates/mdbook-driver/src/lib.rs`
- `mdBook-repo/mdBook/crates/mdbook-driver/src/mdbook.rs`
- `mdBook-repo/mdBook/crates/mdbook-driver/src/load.rs`
- `mdBook-repo/mdBook/crates/mdbook-driver/src/builtin_renderers/mod.rs`
- `mdBook-repo/mdBook/crates/mdbook-driver/src/builtin_preprocessors/cmd.rs`
- `mdBook-repo/mdBook/crates/mdbook-renderer/src/lib.rs`
- `mdBook-repo/mdBook/crates/mdbook-preprocessor/src/lib.rs`
- `mdBook-repo/mdBook/crates/mdbook-html/src/html_handlebars/hbs_renderer.rs`
- `mdBook-repo/mdBook/crates/mdbook-html/src/html/mod.rs`
- `mdBook-repo/mdBook/guide/src/for_developers/backends.md`
- `mdBook-repo/mdBook/guide/src/for_developers/preprocessors.md`
- `mdBook-repo/mdBook/guide/src/format/configuration/renderers.md`
- `mdBook-repo/mdBook/guide/src/format/configuration/preprocessors.md`

Public API docs:

- https://docs.rs/mdbook-driver/latest/mdbook_driver/
- https://docs.rs/mdbook-driver/latest/mdbook_driver/struct.MDBook.html
- https://docs.rs/mdbook-renderer/latest/mdbook_renderer/struct.RenderContext.html
- https://docs.rs/mdbook-preprocessor/latest/mdbook_preprocessor/struct.PreprocessorContext.html
