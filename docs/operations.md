# Operations

This document covers the current `book build` and `book serve` behavior for
CLI operators.

## Common Inputs

Both subcommands take an optional `BOOKSHELF_TOML` positional argument:

```bash
book build [BOOKSHELF_TOML]
book serve [BOOKSHELF_TOML]
```

When the config path is omitted, it defaults to `bookshelf.toml` in the
invoking directory.

Both subcommands also accept:

```bash
--dest-dir DIR
```

If `--dest-dir` is provided, site output is written there. A relative
`--dest-dir` is resolved from the current working directory.

If `--dest-dir` is omitted, output uses the mdBook `[build].build-dir` value
from the loaded config. A relative build directory is resolved from the
directory containing `BOOKSHELF_TOML`.

Both subcommands print the resolved bookshelf root, the site output directory,
and each content book's source directory before building. Source directories
are printed as a mapping from book title to source path. Paths are shown
relative to the directory where the command was invoked when possible.

## Build

Build a bookshelf site:

```bash
book build bookshelf.toml
```

Build to an explicit output directory:

```bash
book build bookshelf.toml --dest-dir .tmp/site
```

The build loads the root `[book]` and each `[[bookshelf.book]]`, lets mdBook
load and render each book, then writes the bookshelf routing, navigation,
search, and generated UI assets into the site output.

The repository's own `bookshelf.toml` configures the `mdbook-mermaid`
preprocessor:

```bash
cargo install mdbook-mermaid
cargo run --bin book -- build bookshelf.toml --dest-dir .tmp/project-docs-site
```

That requirement comes from this documentation site's config. Other bookshelf
sites only need the preprocessors they configure.

## Serve

Serve builds the site first, then serves the built static files over HTTP:

```bash
book serve bookshelf.toml
```

Network flags:

```bash
--hostname HOST  # default: localhost
--port PORT      # optional; when omitted, first available port from 3000 to 3100
```

Example:

```bash
book serve bookshelf.toml --hostname localhost
```

`serve` accepts the same optional `BOOKSHELF_TOML` and `--dest-dir` inputs as
`build`.

During `serve`, the build uses mdBook's live-reload endpoint and the process
polls for changes once per second. A detected change triggers a rebuild and
sends a reload message to connected pages.

`serve` prints `Serving on:` after the HTTP listener is bound, then starts the
watcher and prints `Watching for changes...`.

Watched inputs currently include:

- the bookshelf config file
- every configured book source directory
- the configured shared HTML theme, or each book's default `theme` directory
- `[build].extra-watch-dirs`
- configured `output.html.additional-css` and `output.html.additional-js`

The watcher excludes the output directory and generated `.mdbook-bookshelf`
asset trees so generated files do not trigger rebuild loops.
