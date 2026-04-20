# Stock mdBook Build And Render Notes

## CLI To Library Flow

The CLI entrypoint dispatches subcommands in `src/main.rs`. `build` goes to
`cmd::build::execute()` and `serve` goes to `cmd::serve::execute()`.

Relevant files:

- `mdBook-repo/mdBook/src/main.rs:18-31`
- `mdBook-repo/mdBook/src/cmd/build.rs:17-35`

The `build` command is intentionally thin:

1. Resolve the book root with `get_book_dir()`.
2. Call `MDBook::load(book_dir)`.
3. Optionally override `build.build_dir` via `set_dest_dir()`.
4. Call `book.build()`.

That means the real orchestration logic lives in `mdbook_driver::MDBook`, not
in the CLI wrapper.

## What `MDBook` Owns

`MDBook` stores:

- one `root: PathBuf`
- one `config: Config`
- one loaded `book: Book`
- one renderer set
- one preprocessor set

Relevant files:

- `mdBook-repo/mdBook/crates/mdbook-driver/src/mdbook.rs:28-44`
- `mdBook-repo/mdBook/crates/mdbook-driver/src/lib.rs:1-30`

This is the core single-book shape. For bookshelf, the important observation is
that mdBook's high-level library API is not centered on "site with many books";
it is centered on "one book root with plugins".

## How A Book Is Loaded

`MDBook::load()`:

1. Reads `book.toml` from `root/book.toml` if it exists.
2. Applies environment overrides.
3. Delegates to `load_with_config()`.

`load_with_config()` then:

1. Computes `src_dir = root.join(config.book.src)`.
2. Loads a single `Book` from that source directory.
3. Determines renderers from `[output.*]`.
4. Determines preprocessors from `[preprocessor.*]`.

Relevant files:

- `mdBook-repo/mdBook/crates/mdbook-driver/src/mdbook.rs:46-109`

The actual book loading path is:

- read one `SUMMARY.md`
- parse it with `mdbook_summary::parse_summary()`
- resolve linked chapters relative to one `src_dir`
- build one in-memory `Book`

Relevant files:

- `mdBook-repo/mdBook/crates/mdbook-driver/src/load.rs:9-23`
- `mdBook-repo/mdBook/crates/mdbook-driver/src/load.rs:56-138`

Important single-book assumptions encoded here:

- one canonical `SUMMARY.md` at `src_dir/SUMMARY.md`
- all chapter links resolve relative to one source tree
- chapter parent names and numbering are built from one summary tree

## Custom Summary Injection

There is one notable seam: `MDBook::load_with_config_and_summary()`.

Relevant file:

- `mdBook-repo/mdBook/crates/mdbook-driver/src/mdbook.rs:89-109`

This allows callers to:

- supply a custom `Config`
- supply a custom parsed `Summary`
- still let mdBook load chapter files from disk and build a `Book`

This is useful for bookshelf because it means you are not forced to let mdBook
own summary discovery. You can build or adjust a summary yourself and still use
mdBook's chapter loading path afterward.

Limits:

- it still loads exactly one `Book`
- it still assumes one `root` and one `config.book.src`

## Preprocessor And Renderer Registration

`MDBook::build()` iterates registered renderers and runs
`execute_build_process()` for each one.

`execute_build_process()`:

1. Runs preprocessors for the target renderer.
2. Creates a `RenderContext { root, book, config, destination }`.
3. Calls `renderer.render(&render_context)`.

Relevant files:

- `mdBook-repo/mdBook/crates/mdbook-driver/src/mdbook.rs:160-209`

Renderers are determined from `[output.*]`:

- `html` maps to `mdbook_html::HtmlHandlebars`
- `markdown` maps to `MarkdownRenderer`
- everything else maps to `CmdRenderer`, which shells out

Relevant files:

- `mdBook-repo/mdBook/crates/mdbook-driver/src/mdbook.rs:401-423`
- `mdBook-repo/mdBook/guide/src/format/configuration/renderers.md:20-69`
- `mdBook-repo/mdBook/guide/src/for_developers/backends.md:28-43`
- `mdBook-repo/mdBook/crates/mdbook-driver/src/builtin_renderers/mod.rs:15-87`

Preprocessors are determined from `[preprocessor.*]`, topologically ordered,
and filtered per renderer.

Relevant files:

- `mdBook-repo/mdBook/crates/mdbook-driver/src/mdbook.rs:445-569`
- `mdBook-repo/mdBook/guide/src/format/configuration/preprocessors.md:26-99`
- `mdBook-repo/mdBook/guide/src/for_developers/preprocessors.md:15-29`
- `mdBook-repo/mdBook/crates/mdbook-driver/src/builtin_preprocessors/cmd.rs:57-146`

## Plugin Protocol Reality

Backends and preprocessors are both JSON protocols over stdin/stdout:

- renderers receive a serialized `RenderContext`
- preprocessors receive `(PreprocessorContext, Book)` and emit a `Book`

Relevant files:

- `mdBook-repo/mdBook/crates/mdbook-renderer/src/lib.rs:22-88`
- `mdBook-repo/mdBook/crates/mdbook-preprocessor/src/lib.rs:23-84`
- `mdBook-repo/mdBook/guide/src/for_developers/backends.md:28-43`
- `mdBook-repo/mdBook/guide/src/for_developers/preprocessors.md:20-29`

This is a strong seam for custom behavior, but it is still single-book shaped.
There is no built-in protocol for "here are three books plus a synthetic site
root plus active-book-specific navigation state".

## HTML Renderer Assumptions That Matter For Bookshelf

The built-in HTML renderer is `HtmlHandlebars`.

Relevant files:

- `mdBook-repo/mdBook/crates/mdbook-html/src/html_handlebars/hbs_renderer.rs:19-28`
- `mdBook-repo/mdBook/crates/mdbook-html/src/html/mod.rs:88-107`

Important behavior:

1. It builds one `chapter_trees` vector from `book.chapters()`.
   - `mdBook-repo/mdBook/crates/mdbook-html/src/html_handlebars/hbs_renderer.rs:358-360`
   - `mdBook-repo/mdBook/crates/mdbook-html/src/html/mod.rs:88-107`

2. It computes previous and next by flat position inside that one vector.
   - `mdBook-repo/mdBook/crates/mdbook-html/src/html_handlebars/hbs_renderer.rs:415-417`

3. It injects one global `chapters` list into template data for the sidebar.
   - `mdBook-repo/mdBook/crates/mdbook-html/src/html_handlebars/hbs_renderer.rs:455-620`

4. It writes `index.html` by copying the first rendered chapter.
   - `mdBook-repo/mdBook/crates/mdbook-html/src/html_handlebars/hbs_renderer.rs:126-133`

5. It emits search, toc, print, redirects, and then copies non-markdown assets
   from one source tree into one destination tree.
   - `mdBook-repo/mdBook/crates/mdbook-html/src/html_handlebars/hbs_renderer.rs:367-447`

These are the core reasons stock `HtmlHandlebars` is a bad direct fit for the
bookshelf shell:

- sidebar data is site-global rather than active-book-scoped
- previous and next cross the whole chapter stream
- root `index.html` is tied to "first chapter" semantics
- search and breadcrumbs are derived from one book tree

## What Seems Reusable

Useful to reuse directly:

- `mdbook_summary` parsing via mdBook's load path
- default preprocessor pipeline
- plugin discovery and command execution logic
- `RenderContext` and `PreprocessorContext` data shapes

Useful conceptually but not directly enough as-is:

- `HtmlHandlebars` output pipeline
- `build_trees()` and markdown rendering internals

Why only conceptually:

- the useful helpers inside `mdbook_html` are largely not public
- externally, the main stable thing you get is `HtmlHandlebars` as a whole

## Bookshelf Implications

The most realistic mdBook-first options are:

1. Custom driver around mdBook crates.
   - Load multiple books yourself.
   - Reuse summary parsing and possibly per-book chapter loading.
   - Build a bookshelf-owned site model.
   - Render your own multi-book shell.

2. Custom renderer inside a custom driver.
   - Still use mdBook plugin APIs where useful.
   - Do not rely on stock `HtmlHandlebars` to express multi-book navigation.

The least realistic option is "just use stock extension points from normal
`mdbook build` without an outer driver", because the plugin payload is already
single-book by the time your code runs.
