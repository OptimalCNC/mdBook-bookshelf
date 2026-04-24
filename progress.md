# Objective
Fix `book serve` so source/config changes are watched, rebuilt, and refreshed while keeping `serve` and `build` on the same bookshelf build engine.

# Global Constraints
- Project is under active development; prefer the best current solution over compatibility.
- Treat `mdBook-repo` as upstream reference only.
- Keep build and serve close to stock mdBook CLI shape: thin command wrappers, build once before serve, serve one output directory, rebuild on watch changes, and push live reload where practical.
- Reuse mdBook crates and existing bookshelf build orchestration; do not fork rendering logic.

# Integration Strategy
- Keep `build_bookshelf_site` as the single build engine used by both `build` and `serve`.
- Add a stock-mdBook-like serve loop: initial build, static server, live-reload websocket endpoint, watch/rebuild loop.
- Replace only stock single-book watch root discovery with bookshelf-aware roots derived from `bookshelf.toml`, catalog books, mdBook config, source roots, theme dirs, extra watch dirs, and configured HTML assets.

# Current State
- `crates/mdbook-bookshelf/src/serve.rs` builds once and serves a static directory with no watcher or live reload.
- `crates/mdbook-bookshelf/src/build.rs` is the current shared build engine and must remain the only rendering/build path.

# Open Risks
- Watch root discovery must avoid watching generated output directories in a way that triggers rebuild loops.
- Live-reload assets/endpoints must match the HTML rendered by the existing build engine.

# Active Chunk
```yaml
chunk_id: serve-watch-001
title: Stock-style serve watch/rebuild/reload loop
objective: Make `book serve` build once, serve one output tree, watch bookshelf inputs, rerun the shared bookshelf build engine on changes, and broadcast reload after successful rebuild.
why_now: Current serve already uses the shared build engine but only serves static output; this is the smallest chunk that proves an authored change becomes visible through the running server.
depends_on: []
touchpoints:
  - `mdBook-repo/mdBook/src/cmd/serve.rs` stock initial build, serve-mode config overlay, static fallback, `__livereload` websocket, and post-build reload broadcast shape.
  - `mdBook-repo/mdBook/src/cmd/watch/poller.rs` stock whole-tree reload/rebuild polling pattern, with bookshelf-owned root discovery.
  - `crates/mdbook-bookshelf/src/build.rs::build_bookshelf_site` as the only initial and watched rebuild callback.
  - `crates/mdbook-bookshelf/src/catalog.rs::build_input_catalog` for bookshelf-aware watch roots.
scope_in:
  - Add a serve-mode build option that sets `output.html.live-reload-endpoint = "__livereload"` through mdBook config before rendering.
  - Add a serve-owned poll watcher thread that scans `bookshelf.toml` and each catalog book source directory.
  - On watched changes, reload catalog/config from disk and call the same bookshelf build engine with serve-mode options.
  - Add stock-style `/__livereload` websocket route and broadcast `reload` only after successful rebuild.
  - Keep HTTP static serving rooted at the site root returned by the initial build.
  - Add focused CLI integration coverage using a copied fixture to prove served HTML changes without restarting serve.
scope_out:
  - Native notify watcher and `--watcher` CLI flag.
  - Full `.gitignore`, theme dir, `extra_watch_dirs`, `additional_css`, and `additional_js` parity.
  - Incremental rebuilds, a separate renderer, or generated HTML patching.
  - Browser automation.
target_files:
  - `crates/mdbook-bookshelf/src/build.rs`
  - `crates/mdbook-bookshelf/src/lib.rs`
  - `crates/mdbook-bookshelf/src/serve.rs`
  - `tests/serve_cli.rs`
  - `Cargo.toml`
  - `Cargo.lock`
implementation_tasks:
  - Add build options or an equivalent internal wrapper so normal `book build` does not emit live-reload assets and `book serve` does.
  - Add minimal watcher helpers in `serve.rs` to derive watch roots from `build_input_catalog`, recursively snapshot file metadata, poll, and report changed paths while excluding the output directory.
  - Start the watcher concurrently after initial build, passing cloned config path, dest dir, and reload sender.
  - Add a `/__livereload` websocket route to the existing router and send text `reload` after successful watched rebuilds.
  - Ensure both initial serve and watched rebuilds call the shared build engine.
  - Add only necessary dependency/feature changes.
  - Extend `tests/serve_cli.rs` with a temp copied fixture, source edit, live-reload script assertion, and polling HTTP assertion for new content.
acceptance_criteria:
  - Existing serve CLI tests still pass.
  - Editing a Markdown file under a catalog book source while `book serve` is running rebuilds successfully.
  - The updated content is returned over HTTP by the same running serve process.
  - Serve output includes the stock mdBook live-reload client script; build output does not.
  - Serve uses the shared bookshelf build engine for initial and watched builds; no duplicate rendering path is introduced.
  - The generated output directory is not a watched root.
  - Reload is broadcast only after successful rebuild.
verification:
  - command: `cargo test --test serve_cli`
    expect: Existing serve tests and the new watch/rebuild proof pass without early serve process exit.
  - command: `cargo test`
    expect: Full suite passes after dependency and feature changes.
review_focus:
  - Confirm serve and build share the same build engine with only a serve config overlay.
  - Confirm watcher roots include config and book source trees while excluding output.
  - Confirm failed rebuilds keep the server alive and do not broadcast reload.
  - Confirm the chunk stays limited to minimal poll-based proof, not native watcher or full watch-root parity.
```

# Chunk Ledger
None yet.

# Final Validation
Pending.

# Activity Log
2026-04-24T03:55:58Z [coordinator] [setup] [started] Created coordination artifact and captured initial serve/build state.
2026-04-24T03:58:09Z [planner] [serve-watch-001] [planned] Selected minimal stock-style serve watch/rebuild/reload chunk.
2026-04-24T03:57:57Z [explorer] [serve-watch] [completed] Mapped shared build engine, serve router, watch roots, live-reload endpoint, and missing test coverage touchpoints.
2026-04-24T03:57:05Z [researcher] [serve-watch-seam] [done] Reuse stock static serve/live-reload/build-on-change shape; replace single-MDBook watcher roots with bookshelf source set.
2026-04-24T03:59:49Z [coordinator] [serve-watch-001] [accepted] Accepted first chunk with serve-mode live-reload config overlay added to scope.
2026-04-24T04:02:39Z [developer] [serve-watch-001] [started] Implementing serve-mode live reload, poll rebuilds, and CLI coverage.
2026-04-24T04:05:50Z [developer] [serve-watch-001] [completed] Added serve rebuild watcher, live-reload route, focused CLI coverage, and passed requested tests.
