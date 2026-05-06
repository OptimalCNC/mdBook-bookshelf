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

Both subcommands print the resolved config root, the site output directory, and
the number of books before rendering. As each book finishes, the CLI prints
that book's title, source directory, elapsed time, and page count. Paths are
shown relative to the directory where the command was invoked when possible.

The CLI also prints build progress on stderr. It reports each book as it
finishes rendering, documentation index output, shared search index output, and
the final output directory. This makes long-running builds show which book or
site-wide phase just completed.

## Build

Build a documentation portal:

```bash
book build bookshelf.toml
```

Build to an explicit output directory:

```bash
book build bookshelf.toml --dest-dir .tmp/site
```

The build loads the root `[book]` and each `[[bookshelf.book]]`, lets mdBook
load and render each book from the config root, then writes the documentation
index, routing metadata, search, and generated runtime assets into the site
output.
Runtime source assets are generated under `[bookshelf].asset-dir`, which
defaults to `.mdbook/bookshelf`.

The repository's own `bookshelf.toml` configures the `mdbook-mermaid`
preprocessor:

```bash
cargo install mdbook-mermaid
cargo run --bin book -- build bookshelf.toml --dest-dir .tmp/project-docs-site
```

That requirement comes from this documentation site's config. Other portals
only need the preprocessors they configure.


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

`serve` prints the same concise build progress before binding HTTP. It prints
`Serving on:` after the HTTP listener is bound, then starts the watcher and
prints `Watching for changes...`. Rebuilds triggered by the watcher print the
changed-path summary and rebuild duration. Failed rebuilds still report the
full error cause chain.

CLI failures are printed as an error followed by its `Caused by:` chain when
lower-level context is available. For authoring failures, the chain usually
includes the affected book id, source path, and file path such as
`SUMMARY.md`.

Watched inputs currently include:

- the `bookshelf.toml` config file
- every configured book source directory
- the configured shared HTML theme, or each book's default `theme` directory
- `[build].extra-watch-dirs`
- configured `output.html.additional-css` and `output.html.additional-js`

The watcher excludes the output directory and the configured
`[bookshelf].asset-dir` tree so generated files do not trigger rebuild loops.
Other `.mdbook` paths are not ignored unless they are inside that configured
asset directory.
