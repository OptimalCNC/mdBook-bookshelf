# Stock mdBook Serve And Watch Notes

## High-Level Flow

`mdbook serve` is also a thin wrapper over `MDBook`.

Relevant files:

- `mdBook-repo/mdBook/src/cmd/serve.rs:50-105`
- `mdBook-repo/mdBook/src/cmd/watch.rs:36-72`

`serve` does this:

1. Load one `MDBook`.
2. Mutate config for serve mode.
3. Build once.
4. Serve the HTML output directory over HTTP.
5. If watch support is enabled, rebuild on change and push a live-reload
   websocket message.

## Serve-Specific Config Mutation

Before building, `serve` mutates the loaded book config:

- sets `output.html.live-reload-endpoint`
- applies `--dest-dir`
- forces `output.html.site-url = "/"`

Relevant file:

- `mdBook-repo/mdBook/src/cmd/serve.rs:61-70`

This matters for bookshelf because it shows mdBook already treats serve mode as
"build with a small config overlay", not as a separate rendering architecture.
That pattern is worth reusing.

## HTTP Serving Model

The actual HTTP serving layer is simple:

- an `axum` router
- one websocket endpoint at `__livereload`
- one static file fallback rooted at `build_dir_for("html")`
- `ServeFile` fallback for the configured 404 output

Relevant files:

- `mdBook-repo/mdBook/src/cmd/serve.rs:108-150`

This part is very reusable for bookshelf. If your build step can write one
output tree with `index.html`, content pages, `searchindex.js`, and `404.html`,
the stock serve pattern works with only minor changes.

## Watch Entry Points

`watch::rebuild_on_change()` chooses either:

- poll watcher
- native watcher

Relevant file:

- `mdBook-repo/mdBook/src/cmd/watch.rs:62-71`

`watch` and `serve` both reuse the same rebuild function shape:

- `book_dir`
- `update_config`
- `post_build`

This abstraction is useful. A bookshelf implementation can keep the same shape
even if the build pipeline is custom.

## Native Watcher

The native watcher:

- loads one `MDBook`
- watches `source_dir()`
- watches `theme_dir()`
- watches `book.toml`
- watches `build.extra_watch_dirs`
- filters ignored paths using `.gitignore`
- reloads `MDBook` from disk on each change and rebuilds

Relevant file:

- `mdBook-repo/mdBook/src/cmd/watch/native.rs:11-149`

Important implications:

1. Watch roots come from a single loaded `MDBook`.
2. The rebuild path is not incremental; it reloads and rebuilds the whole book.
3. `.gitignore` handling is built in and worth copying if bookshelf needs the
   same ergonomics.

## Poll Watcher

The poll watcher:

- also reloads one `MDBook` on each change
- scans source, theme, `book.toml`, `extra_watch_dirs`, and HTML
  `additional_css` / `additional_js`
- compares metadata snapshots to detect changes

Relevant file:

- `mdBook-repo/mdBook/src/cmd/watch/poller.rs:17-234`

Important details:

- roots are recomputed from the reloaded `MDBook`
- it follows symlinks during scans
- it also filters paths using `.gitignore`

## Output Directory Handling

CLI `--dest-dir` is interpreted relative to the current directory, then written
into `book.config.build.build_dir`.

Relevant file:

- `mdBook-repo/mdBook/src/cmd/command_prelude.rs:62-68`

The actual served directory is `book.build_dir_for("html")`.

Relevant file:

- `mdBook-repo/mdBook/src/cmd/serve.rs:76-78`
- `mdBook-repo/mdBook/crates/mdbook-driver/src/mdbook.rs:371-379`

This means:

- if there is only one renderer, HTML is served directly from `build.build_dir`
- if multiple renderers exist, HTML is served from the `html` subdirectory

For bookshelf, this is a useful pattern if you ever want one build directory
that can hold more than one renderer.

## Reusable Pieces For Bookshelf

Good candidates to reuse closely:

- the Axum static server layout
- websocket-driven live reload
- "build once, then rebuild on change" flow
- `update_config` hook for serve-only overrides
- watcher root categories:
  - source trees
  - theme dirs
  - config files
  - extra watch dirs
  - extra CSS and JS assets

## Pieces That Need Re-Thinking

The stock watch path assumes one `MDBook`, so bookshelf cannot reuse it
unchanged if the source of truth is `bookshelf.toml` plus many books.

A bookshelf watcher will likely need to watch:

- `bookshelf.toml`
- every participating book root
- every participating `SUMMARY.md`
- each participating theme dir, if any
- extra watch dirs and shared assets

The server side is easy to keep mdBook-like. The watcher root discovery is the
part that needs a bookshelf-owned model.

## Practical Conclusion

If we choose an mdBook-first custom driver, the best serve strategy is:

1. Keep a stock-like HTTP server and live-reload channel.
2. Replace the build callback with a bookshelf build pipeline.
3. Replace watcher root discovery with a bookshelf-aware source set.
4. Keep the user-facing CLI contract mdBook-like.
