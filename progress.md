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
- Commit `24a370978f8132fd03229e52e83e85e75f2ac5b9` added the stock-style serve loop: serve-mode live reload config overlay, static server, `/__livereload`, poll rebuilds, and focused source-edit tests.
- The remaining stock-alignment gap is watch root parity for shared mdBook assets: theme roots, extra watch dirs, and configured HTML CSS/JS assets.

# Open Risks
- Watch root discovery must avoid watching generated output directories in a way that triggers rebuild loops.
- Live-reload assets/endpoints must match the HTML rendered by the existing build engine.

# Active Chunk
```yaml
chunk_id: serve-watch-002
title: Stock poll root categories for bookshelf serve
objective: Extend the existing serve poll watcher root discovery to include the stock mdBook watch categories that apply to bookshelf: theme roots, extra watch dirs, and configured HTML CSS/JS assets.
why_now: The serve loop now rebuilds and reloads on book source changes, but shared mdBook assets from bookshelf.toml can still change without triggering a rebuild.
depends_on:
  - serve-watch-001
touchpoints:
  - `crates/mdbook-bookshelf/src/serve.rs::PollWatcher::set_roots_from_config`
  - `mdBook-repo/mdBook/src/cmd/watch/poller.rs::Watcher::set_roots` stock root category list
  - `mdbook_driver::config::Config::html_config` for output.html.additional-css/additional-js/theme
  - `crates/mdbook-bookshelf/src/bookshelf_ui.rs::TransientBookshelfUiAssets::stage_config_root_output_assets` shared config-root asset behavior
scope_in:
  - Add testable watch-root collection for bookshelf.toml, every catalog book source dir, shared configured theme, default per-book theme dirs, build.extra-watch-dirs, and output.html.additional-css/additional-js.
  - Resolve shared mdBook config roots relative to catalog.config_dir, preserving absolute paths.
  - Keep excluding the serve output directory from roots and scans.
  - Skip generated transient .mdbook-bookshelf asset trees during scans so broad roots such as extra-watch-dirs = ["."] do not self-trigger rebuild loops.
  - Add focused unit coverage for root discovery categories and generated-root exclusion.
  - Add focused serve CLI coverage proving an edited configured additional CSS asset is recopied and served without restarting serve.
scope_out:
  - Native notify watcher or `--watcher` CLI flag.
  - `.gitignore` parity.
  - Per-book build/output config support beyond the current shared bookshelf.toml model.
  - Incremental rebuilds or any build/render architecture change.
  - Browser automation or websocket client assertions.
target_files:
  - `crates/mdbook-bookshelf/src/serve.rs`
  - `tests/serve_cli.rs`
implementation_tasks:
  - Refactor PollWatcher root setup into a small helper that returns normalized, deduplicated watch roots from the loaded InputCatalog.
  - Add shared config-root path resolution for build.extra_watch_dirs and output.html additional CSS/JS assets.
  - Add theme root discovery: use output.html.theme from the shared mdBook config when configured, otherwise include each catalog book root's default theme directory.
  - Preserve the existing output-dir exclusion and add scan-time exclusion for .mdbook-bookshelf generated asset directories.
  - Add serve.rs unit tests for discovered roots and exclusion behavior using temporary test fixtures.
  - Extend serve_cli tests with a copied fixture containing output.html.additional-css, edit the CSS while serve is running, and poll the served CSS URL until the marker appears.
acceptance_criteria:
  - Poll watcher roots include bookshelf.toml, every catalog book source dir, applicable theme dirs, build.extra-watch-dirs, and output.html.additional-css/additional-js.
  - Relative shared mdBook config paths resolve from the bookshelf.toml directory.
  - The serve output directory is not watched or scanned.
  - Generated .mdbook-bookshelf transient asset trees do not trigger rebuilds when a broad watched root contains them.
  - Editing a configured additional CSS asset while `book serve` is running rebuilds and serves the updated asset without restarting.
  - No native watcher, renderer fork, or build engine change is introduced.
verification:
  - command: `cargo test watch_roots`
    expect: New focused root discovery and exclusion unit tests pass.
  - command: `cargo test --test serve_cli serve_cli_rebuilds_changed_configured_html_asset`
    expect: The running serve process notices a shared CSS edit and serves the updated asset.
  - command: `cargo test --test serve_cli`
    expect: Existing serve behavior and the new asset-watch proof pass together.
review_focus:
  - Confirm the root categories mirror upstream poller categories where bookshelf's shared config model makes them meaningful.
  - Confirm shared asset paths are resolved against catalog.config_dir, not transient staged child-book locations.
  - Confirm broad roots cannot watch serve output or .mdbook-bookshelf generated assets.
  - Confirm the change remains limited to poll watcher root discovery and tests.
```

# Chunk Ledger
- `serve-watch-001`: approved in commit `24a370978f8132fd03229e52e83e85e75f2ac5b9`; added shared-engine serve-mode build overlay, static serving, `/__livereload`, poll rebuilds for config/source changes, and focused source-edit CLI coverage.

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
2026-04-24T04:07:52Z [reviewer-subagent] [serve-watch-001] [approved] Serve watch/rebuild/reload meets active chunk criteria; focused and full tests pass.
2026-04-24T04:07:40Z [reviewer] [serve-watch-001] [approved] Direction matches stock-style shared-engine serve watch/rebuild/reload scope.
2026-04-24T04:09:19Z [researcher-subagent] [watch-root-seam] [done] Derive shared mdBook asset watch roots against config_dir, then exclude site output and transient staged UI asset trees.
2026-04-24T04:09:21Z [planner] [serve-watch-002] [planned] Planned stock-category poll roots for shared theme, extra watch dirs, and configured HTML assets.
2026-04-24T04:11:08Z [coordinator] [serve-watch-002] [accepted] Accepted stock poll root category follow-up after first chunk approval.
2026-04-24T04:13:58Z [developer] [serve-watch-002] [started] Implementing stock mdBook poll root categories and transient asset exclusions.
2026-04-24T04:16:11Z [developer] [serve-watch-002] [completed] Added stock poll root categories, generated asset scan exclusion, CSS serve coverage, and passed requested/full tests.
