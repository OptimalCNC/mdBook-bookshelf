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
- Commit `8718e338e1c46429e35b7f923107295ca7a71ed4` added stock poll watch root categories for bookshelf: theme roots, extra watch dirs, configured HTML CSS/JS assets, output exclusion, and transient generated asset exclusion.

# Open Risks
- Watch root discovery must avoid watching generated output directories in a way that triggers rebuild loops.
- Live-reload assets/endpoints must match the HTML rendered by the existing build engine.

# Active Chunk
None.

# Chunk Ledger
- `serve-watch-001`: approved in commit `24a370978f8132fd03229e52e83e85e75f2ac5b9`; added shared-engine serve-mode build overlay, static serving, `/__livereload`, poll rebuilds for config/source changes, and focused source-edit CLI coverage.
- `serve-watch-002`: approved in commit `8718e338e1c46429e35b7f923107295ca7a71ed4`; added stock poll watch root categories for shared mdBook theme/assets/extra watch dirs, output/generated asset exclusion, unit coverage, and shared CSS serve rebuild coverage.

# Final Validation
- `cargo test watch_roots`: passed.
- `cargo test --test serve_cli`: passed.
- `cargo test`: passed.
- Final direction review: approved.
- Final implementation review: approved.

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
2026-04-24T04:17:47Z [reviewer] [serve-watch-002] [approved] Direction matches stock poll root categories without build, CLI, watcher, or renderer scope drift.
2026-04-24T04:18:13Z [reviewer-subagent] [serve-watch-002] [approved] Roots, config-dir resolution, generated exclusions, and CSS serve rebuild coverage pass.
2026-04-24T04:19:17Z [coordinator] [final-validation] [passed] cargo test watch_roots, cargo test --test serve_cli, and cargo test passed on current workspace.
2026-04-24T04:21:40Z [reviewer-final-subagent] [serve-watch-final] APPROVED - acceptance complete; focused validation passes and no rebuild/reload/watch exclusion blockers found.
2026-04-24T04:21:34Z [reviewer-final] [serve-watch-final] [approved] Shared build engine and mdBook-shaped serve/watch architecture are intact; divergence stays in bookshelf watch roots.
2026-04-24T04:22:11Z [coordinator] [serve-watch-final] [completed] Serve/watch task completed with final validation and review gates approved.
