# Objective
Deliver stock-mdBook-style support for configured mdBook preprocessors and HTML assets, proving `mdbook-mermaid` works without hardcoding Mermaid-specific behavior.

# Global Constraints
- Project is under active development; prefer the best current solution over compatibility.
- Treat upstream mdBook source as reference only; reuse published mdBook crates and public APIs.
- Keep `build` and `serve` close to stock mdBook shape: thin CLI wrappers over the shared build engine, one initial build for serve, and mdBook-owned single-book processing where practical.
- Do not special-case Mermaid or any named plugin; support should come from generic mdBook config and plugin seams.

# Integration Strategy
- Continue using `MDBook::load_with_config_and_summary()` for each catalog book so mdBook discovers configured `[preprocessor.*]` command plugins and runs them through the normal preprocessor pipeline during `mdbook.build()`.
- Preserve stock `output.html.additional-js` / `additional-css` handling by resolving shared config-root assets and staging them into child book roots before rendering.
- Add focused coverage that configures a real external preprocessor plus additional JS assets and asserts the generated HTML reflects the preprocessed Markdown and asset injection.

# Current State
- `mdbook-mermaid` is installed at `/home/huwei/.cargo/bin/mdbook-mermaid` in the local environment.
- Commit `f2559ab` adds a focused build CLI regression that configures stock `[preprocessor.mermaid]` plus `[output.html].additional-js`, builds root and child books, and asserts both mdBook command preprocessor output and staged child JS assets.
- No production code changed; support remains on the existing generic mdBook `Config` -> `MDBook::load_with_config_and_summary()` -> `mdbook.build()` seam.

# Open Risks
- The regression depends on `mdbook-mermaid` being installed to exercise the external plugin path; when unavailable it reports a clear skip.

# Active Chunk
None.

# Chunk Ledger
- `plugin-regression`: approved in commit `f2559ab`; added a temp-fixture build CLI regression for stock configured mdBook command preprocessors and generic `output.html.additional-js` staging across root and child book outputs. No production Mermaid-specific code was added.

# Final Validation
- `PATH="$HOME/.cargo/bin:$PATH" mdbook-mermaid --version`: passed (`mdbook-mermaid 0.17.0`).
- `PATH="$HOME/.cargo/bin:$PATH" cargo test --test build_cli build_cli_supports_configured_mdbook_mermaid_preprocessor_and_additional_js -- --nocapture`: passed.
- `cargo test --test build_cli build_cli_resolves_relative_mdbook_paths_from_bookshelf_config_dir`: passed.
- `cargo test`: passed when run without concurrent fixture-mutating commands.
- Chunk direction review: approved.
- Chunk implementation review: approved.
- Final direction review: approved.
- Final implementation review: approved.

# Activity Log
2026-04-24T06:03:49Z [coordinator] [setup] [started] Reset coordination artifact for generic mdBook plugin support and captured current build seam.
2026-04-24T06:05:38Z [researcher-subagent] [plugin-support-seam] [done] Verified stock mdBook preprocessor and additional-asset seams are enough; add focused regression coverage.
2026-04-24T06:04:56Z [planner] [plugin-regression] [planned] Proposed a focused build CLI regression for stock configured preprocessors and additional JS assets.
2026-04-24T06:07:08Z [coordinator] [plugin-regression] [accepted] Accepted focused regression chunk; production changes only if a generic stock seam defect appears.
2026-04-24T06:09:25Z [developer-subagent] [plugin-regression] [started] Adding focused build CLI regression for stock mdBook Mermaid preprocessor and additional JS asset flow.
2026-04-24T06:10:30Z [developer-subagent] [plugin-regression] [completed] Added temp-fixture build CLI regression; targeted Mermaid and shared-asset tests pass without production changes.
2026-04-24T06:12:41Z [reviewer-subagent] [plugin-regression] [approved] Regression exercises configured preprocessor output and child staged JS assets; focused validation passes.
2026-04-24T06:12:23Z [reviewer] [plugin-regression] APPROVED - regression stays on stock mdBook plugin and generic asset seams with no production Mermaid special-casing.
2026-04-24T06:13:27Z [coordinator] [plugin-regression] [approved] Recorded approved commit f2559ab and final validation results.
2026-04-24T06:15:01Z [reviewer-final-subagent] [plugin-support-final] APPROVED - final regression covers configured command preprocessor output and child additional-js staging when mdbook-mermaid is available.
2026-04-24T06:14:48Z [reviewer-final] [plugin-support-final] [approved] Generic mdBook command preprocessor and additional-js seams satisfy final direction with no production Mermaid special-casing.
2026-04-24T06:15:37Z [coordinator] [plugin-support-final] [completed] Final validation and final review gates approved.
