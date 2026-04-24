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
- The existing build path clones shared mdBook config per book, stages config-root HTML assets for child book roots, injects bookshelf UI assets, loads each book through `MDBook::load_with_config_and_summary()`, registers the bookshelf link preprocessor, and calls `mdbook.build()`.
- No explicit regression coverage currently proves an external configured preprocessor runs in the bookshelf build.

# Open Risks
- Additional JS/CSS asset staging must remain generic and should not assume Mermaid asset names.
- Bookshelf-owned preprocessors must not suppress or reorder configured stock mdBook preprocessors in a way that breaks plugin output.
- The test should skip or report clearly if the external binary is unavailable on another machine, unless the project decides to vendor a test preprocessor fixture.

# Active Chunk
`plugin-regression`: Add one focused `tests/build_cli.rs` regression that builds a temp bookshelf fixture using stock mdBook config:
- `[preprocessor.mermaid] command = "mdbook-mermaid"`
- `[output.html] additional-js = ["mermaid.min.js", "mermaid-init.js"]`
- root and child Mermaid fences

Acceptance: generated root and child HTML contain Mermaid-preprocessed markup, configured JS assets are copied/referenced for root and staged child output, bookshelf UI assets still appear, and no production code hardcodes Mermaid.

# Chunk Ledger
None yet.

# Final Validation
Pending.

# Activity Log
2026-04-24T06:03:49Z [coordinator] [setup] [started] Reset coordination artifact for generic mdBook plugin support and captured current build seam.
2026-04-24T06:05:38Z [researcher-subagent] [plugin-support-seam] [done] Verified stock mdBook preprocessor and additional-asset seams are enough; add focused regression coverage.
2026-04-24T06:04:56Z [planner] [plugin-regression] [planned] Proposed a focused build CLI regression for stock configured preprocessors and additional JS assets.
2026-04-24T06:07:08Z [coordinator] [plugin-regression] [accepted] Accepted focused regression chunk; production changes only if a generic stock seam defect appears.
2026-04-24T06:09:25Z [developer-subagent] [plugin-regression] [started] Adding focused build CLI regression for stock mdBook Mermaid preprocessor and additional JS asset flow.
2026-04-24T06:10:30Z [developer-subagent] [plugin-regression] [completed] Added temp-fixture build CLI regression; targeted Mermaid and shared-asset tests pass without production changes.
