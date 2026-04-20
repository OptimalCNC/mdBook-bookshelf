# mdbook_driver API Notes

## Public Entry Points

The high-level crate is `mdbook_driver`.

Useful docs:

- https://docs.rs/mdbook-driver/latest/mdbook_driver/
- https://docs.rs/mdbook-driver/latest/mdbook_driver/struct.MDBook.html
- https://docs.rs/mdbook-renderer/latest/mdbook_renderer/struct.RenderContext.html
- https://docs.rs/mdbook-preprocessor/latest/mdbook_preprocessor/struct.PreprocessorContext.html

Local source:

- `mdBook-repo/mdBook/crates/mdbook-driver/src/lib.rs:1-80`

The crate explicitly presents itself as the high-level library for:

- integrating mdBook into another project
- extending mdBook
- doing extra processing before build
- helping create new renderers

## Main Type: `MDBook`

Local source:

- `mdBook-repo/mdBook/crates/mdbook-driver/src/mdbook.rs:28-44`

The main methods that matter for bookshelf are:

- `MDBook::load()`
- `MDBook::load_with_config()`
- `MDBook::load_with_config_and_summary()`
- `MDBook::build()`
- `MDBook::preprocess_book()`
- `MDBook::execute_build_process()`
- `MDBook::with_renderer()`
- `MDBook::with_preprocessor()`
- `MDBook::build_dir_for()`
- `MDBook::source_dir()`
- `MDBook::theme_dir()`

Relevant source:

- `mdBook-repo/mdBook/crates/mdbook-driver/src/mdbook.rs:46-109`
- `mdBook-repo/mdBook/crates/mdbook-driver/src/mdbook.rs:160-225`
- `mdBook-repo/mdBook/crates/mdbook-driver/src/mdbook.rs:371-392`

## Best Reusable API Surfaces

### `load_with_config_and_summary()`

This is the most interesting API for bookshelf.

Why:

- it lets you keep control of config and summary selection
- it still reuses mdBook's chapter loading path

Limit:

- the result is still one `MDBook`

### `with_renderer()` and `with_preprocessor()`

These let you programmatically register your own implementations rather than
relying only on `book.toml` discovery.

Relevant source:

- `mdBook-repo/mdBook/crates/mdbook-driver/src/mdbook.rs:211-225`

This is useful if a bookshelf driver wants to keep mdBook crates in-process
instead of shelling out to plugin commands.

### `build_dir_for()`

This gives stock mdBook's convention for output directory layout when there is
one renderer versus many renderers.

Relevant source:

- `mdBook-repo/mdBook/crates/mdbook-driver/src/mdbook.rs:371-379`

## Renderer API

The public renderer trait is intentionally small:

- `fn name(&self) -> &str`
- `fn render(&self, ctx: &RenderContext) -> Result<()>`

Relevant source:

- `mdBook-repo/mdBook/crates/mdbook-renderer/src/lib.rs:22-35`

`RenderContext` contains:

- `version`
- `root`
- `book`
- `config`
- `destination`
- `chapter_titles` internal map

Relevant source:

- `mdBook-repo/mdBook/crates/mdbook-renderer/src/lib.rs:37-88`

What is reusable:

- stable high-level payload for one loaded book
- a clean in-process renderer interface

What is opinionated:

- exactly one `Book`
- exactly one `root`
- destination scoped to one renderer output

## Preprocessor API

The public preprocessor trait is also small:

- `fn name(&self) -> &str`
- `fn run(&self, ctx: &PreprocessorContext, book: Book) -> Result<Book>`
- `fn supports_renderer(&self, renderer: &str) -> Result<bool>`

Relevant source:

- `mdBook-repo/mdBook/crates/mdbook-preprocessor/src/lib.rs:23-45`

`PreprocessorContext` contains:

- `root`
- `config`
- `renderer`
- `mdbook_version`
- internal `chapter_titles`

Relevant source:

- `mdBook-repo/mdBook/crates/mdbook-preprocessor/src/lib.rs:47-84`

What is reusable:

- good shape for in-process content transformation
- explicit renderer targeting

What is opinionated:

- input is one already-loaded `Book`
- output must still be one `Book`

## Command-Based Plugins

`mdbook_driver` also supports command plugins by convention:

- custom renderers become `CmdRenderer`
- custom preprocessors become `CmdPreprocessor`

Relevant source:

- `mdBook-repo/mdBook/crates/mdbook-driver/src/builtin_renderers/mod.rs:15-87`
- `mdBook-repo/mdBook/crates/mdbook-driver/src/builtin_preprocessors/cmd.rs:57-146`

Useful details:

- renderer commands receive JSON `RenderContext` on stdin
- preprocessor commands first get `supports <renderer>`, then JSON
  `(PreprocessorContext, Book)`
- command names default to `mdbook-<name>` unless overridden in config

Relevant docs:

- `mdBook-repo/mdBook/guide/src/for_developers/backends.md:82-156`
- `mdBook-repo/mdBook/guide/src/for_developers/preprocessors.md:15-29`

## Single-Book Pressure Points

The places where the public API clearly assumes one book are:

1. `MDBook` stores one `root`, one `config`, one `book`.
2. `RenderContext` exposes one `book`.
3. `PreprocessorContext` describes one `root` and one renderer pass.
4. `load()` discovers one `book.toml` and one `SUMMARY.md` tree.

This does not mean the API is unusable for bookshelf. It means the custom
driver has to own the multi-book composition layer above mdBook.

## Recommended Use For Bookshelf

The most promising way to use these APIs is:

1. Use a bookshelf-owned top-level config and catalog loader.
2. For each participating book, either:
   - call `MDBook::load()` if stock discovery is acceptable, or
   - call `MDBook::load_with_config_and_summary()` if you need to control the
     summary tree.
3. Reuse preprocessing where it helps.
4. Build one explicit multi-book site model above mdBook's single-book `Book`
   values.
5. Render the final shelf, page chrome, search labeling, and navigation from
   that site model.

## What Not To Expect From `mdbook_driver`

`mdbook_driver` does not already give you:

- a multi-book site model
- active-book-scoped sidebar state
- cross-book search labels
- root-only synthetic bookshelf page support
- multi-book previous and next constraints

Those remain bookshelf-owned responsibilities.
