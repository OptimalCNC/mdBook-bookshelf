# Objective
Improve the project documentation so different users can quickly find the docs
they need and read them easily, while correcting drift against the current
codebase.

# Global Constraints
- Project is under active development; prefer the best current solution over
  compatibility.
- Keep the documented product direction mdBook-first: stock mdBook owns
  single-book loading and rendering, while `mdbook-bookshelf` adds multi-book
  routing, navigation, search, and bookshelf UI.
- Documentation changes should reflect current code and tests, not planned
  behavior.
- Keep docs concise, navigable, and reader-oriented for distinct audiences:
  users evaluating the tool, site authors, operators running the CLI, and
  contributors changing implementation.

# Integration Strategy
- Treat `README.md`, `docs/SUMMARY.md`, and `docs/index.md` as the first-entry
  navigation layer.
- Keep detailed reference content in focused docs under `docs/`.
- Validate behavioral claims against the Rust code and tests before editing.
- Use the repo's own `book build bookshelf.toml` flow as documentation build
  validation.

# Current State
- Existing docs cover overview, site behavior, linking, configuration,
  authoring, examples, and contributing.
- Initial scan found the docs broadly aligned with the current code, but the
  entry points can be improved for role-based navigation and easier reading.
- Drift audit found concrete gaps in sidebar placement, CLI/serve defaults,
  config validation, shared output assets, linking rules, authoring assumptions,
  and search caveats.

# Open Risks
- Documentation drift may exist in lower-level details such as config
  validation, CLI flags, generated routes, injected UI behavior, or search.
- The repo docs build may require local command plugins such as
  `mdbook-mermaid` depending on the current config.

# Active Chunk
```yaml
chunk_id: docs-config-validation-001
title: Document enforced bookshelf configuration validation
objective: Correct docs/configuration.md so it reflects the current parser and catalog validation rules for required bookshelf config, titles, src paths, and output root conflicts.
why_now: This is a compact, high-value drift fix isolated to one documentation file and backed by focused config parser tests.
depends_on: []
touchpoints:
  - docs/configuration.md
  - crates/mdbook-bookshelf/src/config.rs
  - tests/bookshelf_config_parse.rs
scope_in:
  - Document that [bookshelf] is required.
  - Document that root and child book titles are required and must be nonempty.
  - Document rejected src values: empty, absolute, containing .., resolving to ., or named SUMMARY.md.
  - Document duplicate and overlapping output root failures.
scope_out:
  - Do not document shared output asset staging or input-404 behavior in this chunk.
  - Do not change authoring guidance about index.md versus SUMMARY.md.
  - Do not change site-behavior search caveats.
  - Do not edit source code or tests.
target_files:
  - docs/configuration.md
implementation_tasks:
  - Read current validation tests and config parser code to confirm exact user-facing rules and wording.
  - Update the configuration reference with a concise validation subsection near the [bookshelf] and child book configuration material.
  - Keep examples aligned with accepted values and avoid implying index.md is validated during catalog build.
  - Cross-check terminology for output roots against existing docs so duplicate and overlap failures are understandable.
acceptance_criteria:
  - docs/configuration.md explicitly says [bookshelf] is required.
  - docs/configuration.md explicitly says root and child titles are required and nonempty.
  - docs/configuration.md lists all currently rejected src forms without adding unsupported behavior.
  - docs/configuration.md explains that duplicate or overlapping output roots fail validation.
  - No unrelated docs/source files are modified.
verification:
  - command: cargo test -p mdbook-bookshelf --test bookshelf_config_parse
    expect: Passes, confirming the documented validation behavior still matches the focused parser tests.
  - command: git diff -- docs/configuration.md
    expect: Diff is limited to configuration validation documentation and contains no shared asset, search, or authoring-index changes.
review_focus:
  - Check that the docs describe enforced behavior rather than recommended conventions.
  - Check that src validation wording matches config.rs and tests/bookshelf_config_parse.rs exactly.
  - Check that the chunk remains narrow enough for a single review loop.
```

# Chunk Ledger
- `docs-nav-001`: approved in commit `768be49`; added audience routing in
  `README.md` and `docs/index.md`, added `docs/operations.md`, linked it from
  `docs/SUMMARY.md`, and corrected the root `Bookshelf` sidebar wording to the
  implemented first unnumbered entry behavior.
- `docs-linking-001`: approved in commit `36097be`; corrected
  `docs/linking.md` to describe single-slash site-root rewrite behavior,
  preserved destination classes, query/fragment preservation, unresolved
  site-root Markdown page failures, and README/index target compatibility.

# Final Validation
- `cargo test --test cli_help --test bookshelf_config_parse`: passed for
  `docs-nav-001`.
- `cargo test --test build_cli build_cli_emits_bookshelf_ui_assets_without_fixture_residue`:
  passed for `docs-nav-001`.
- `cargo test --test serve_cli serve_cli_serves_built_site`: passed for
  `docs-nav-001`.
- `cargo run --bin book -- build bookshelf.toml --dest-dir .tmp/project-docs-site`:
  passed for `docs-nav-001`.
- `cargo test site_root_link_preprocessor`: passed for `docs-linking-001`.
- `cargo run --bin book -- build bookshelf.toml --dest-dir .tmp/docs-linking-001-site`:
  passed for `docs-linking-001`.

# Activity Log
2026-04-24T06:34:26Z [coordinator] [setup] [started] Reset coordination artifact for documentation navigation and drift cleanup.
2026-04-24T06:36:12Z [planner] [docs-nav-001] [planned] Proposed role-based entry navigation, operations page, and Bookshelf sidebar drift cleanup.
2026-04-24T06:38:14Z [researcher-subagent] [docs-drift-audit] [done] Found sidebar drift plus missing CLI, serve, config, linking, authoring, and search caveats.
2026-04-24T06:39:02Z [coordinator] [docs-nav-001] [accepted] Accepted first docs navigation chunk and recorded concrete scope.
2026-04-24T06:42:42Z [developer-subagent] [docs-nav-001] [completed] Added role routing, operations docs, SUMMARY entry, and first-sidebar Bookshelf wording; targeted validation passed.
2026-04-24T06:43:18Z [coordinator] [docs-nav-001] [checkpoint] Created review checkpoint 768be49 docs: improve navigation entry points.
2026-04-24T06:44:11Z [reviewer] [docs-nav-001] [approved] Role routing, operations docs, SUMMARY placement, and Bookshelf sidebar wording match current mdBook-first behavior.
2026-04-24T06:44:34Z [reviewer-subagent] [docs-nav-001] [approved] Role routing, operations claims, and Bookshelf sidebar wording match current implementation.
2026-04-24T06:44:57Z [coordinator] [docs-nav-001] [approved] Moved approved checkpoint 768be49 to chunk ledger after targeted validation.
2026-04-24T06:46:31Z [planner] [docs-linking-001] [planned] Proposed a focused linking reference drift cleanup backed by site-root link preprocessor tests.
2026-04-24T06:47:05Z [coordinator] [docs-linking-001] [accepted] Accepted one-page linking rewrite rules documentation chunk.
2026-04-24T06:48:26Z [developer-subagent] [docs-linking-001] [started] Started linking rewrite rules documentation correction against site_root_link_preprocessor tests.
2026-04-24T06:49:14Z [developer-subagent] [docs-linking-001] [completed] Documented site-root link rewrite, preservation, failure, and README/index target rules; required validation passed.
2026-04-24T06:50:11Z [coordinator] [docs-linking-001] [checkpoint] Created review checkpoint 36097be docs: clarify link rewrite rules.
2026-04-24T06:50:39Z [reviewer-subagent] [docs-linking-001] [approved] docs/linking.md matches tested site-root rewrite behavior and includes required caveats.
2026-04-24T06:51:02Z [reviewer] [docs-linking-001] [approved] Linking page stays narrow and matches current site-root rewrite behavior.
2026-04-24T06:51:25Z [coordinator] [docs-linking-001] [approved] Moved approved checkpoint 36097be to chunk ledger after validation and review.
2026-04-24T06:52:14Z [planner] [docs-config-validation-001] [planned] Proposed focused configuration validation documentation against parser tests.
2026-04-24T06:52:38Z [coordinator] [docs-config-validation-001] [accepted] Accepted one-file configuration validation drift cleanup chunk.
2026-04-24T06:53:11Z [developer-subagent] [docs-config-validation-001] [started] Started configuration validation documentation correction against config parser tests.
2026-04-24T06:53:58Z [developer-subagent] [docs-config-validation-001] [completed] Documented required bookshelf config, title, src, and output-root validation rules; required validation passed.
2026-04-24T06:55:07Z [coordinator] [docs-config-validation-001] [checkpoint] Created review checkpoint a44ec66 docs: document config validation rules.
2026-04-24T06:57:15Z [reviewers] [docs-config-validation-001] [changes-required] Requested enforced SUMMARY.md wording and focused parser test command correction.
2026-04-24T06:57:36Z [developer-subagent] [docs-config-validation-001] [completed] Reworked SUMMARY.md validation wording and focused verification command; parser test target passed.
