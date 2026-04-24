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
chunk_id: docs-config-shared-assets-001
title: Document shared output assets and per-book HTML config behavior
objective: Correct docs/configuration.md drift around shared HTML output assets, command preprocessors, additional JS, and input-404 behavior across generated child book roots.
why_now: The current configuration docs do not explain how relative shared assets are resolved and staged during multi-book builds, which can mislead authors configuring CSS, JS, preprocessors, Mermaid, or custom 404 pages.
depends_on: []
touchpoints:
  - crates/mdbook-bookshelf/src/bookshelf_ui.rs
  - tests/build_cli.rs
scope_in:
  - Document that relative output.html.additional-css and output.html.additional-js paths are resolved from the config root.
  - Document that shared CSS and JS assets are staged under generated child book roots using bookshelf-config-assets.
  - Document that configured [preprocessor.*] command plugins and additional JS apply to each built book.
  - Document that relative output.html.input-404 is rejected when it would cross different book roots, and can be disabled with an empty string.
scope_out:
  - Do not edit docs/source.
  - Do not change implementation or tests.
  - Do not cover search behavior drift; leave that for a separate docs/site-behavior.md chunk.
target_files:
  - docs/configuration.md
implementation_tasks:
  - Locate the existing configuration sections for HTML output, preprocessors, shared assets, and 404 handling.
  - Add concise reader-facing guidance that matches the current code and test behavior.
  - Prefer examples or notes near the relevant configuration keys instead of creating a broad new section.
  - Cross-check wording against bookshelf_ui.rs and the shared config root, Mermaid/plugin, and input-404 tests in tests/build_cli.rs.
acceptance_criteria:
  - docs/configuration.md clearly states config-root resolution for relative additional-css and additional-js.
  - docs/configuration.md explains bookshelf-config-assets staging for child book roots.
  - docs/configuration.md states that configured command preprocessors and additional JS are applied to every built book.
  - docs/configuration.md states the current relative input-404 rejection behavior across book roots and the empty-string opt-out.
  - The change is limited to the configuration documentation and does not introduce source or implementation edits.
verification:
  - command: git diff -- docs/configuration.md
    expect: The diff only updates configuration documentation and covers all listed shared asset, preprocessor, JS, and input-404 behaviors.
review_focus:
  - Verify that path-resolution wording distinguishes config root, generated child book roots, and staged bookshelf-config-assets paths without implying unsupported compatibility behavior.
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
- `docs-config-validation-001`: approved across commits `a44ec66` and
  `a4e637a`; updated `docs/configuration.md` with enforced `[bookshelf]`,
  title, `src`, duplicate output root, and overlapping output root validation
  rules.
- `docs-authoring-index-001`: approved across commits `1aefd5f` and
  `e789107`; clarified that catalog build validates `SUMMARY.md`, while
  `index.md` remains expected for synthetic shelf links, book entry pages, and
  the default root-book site-root redirect.

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
- `cargo test -p mdbook-bookshelf --test bookshelf_config_parse`: passed for
  `docs-config-validation-001`.
- `rg -n "index.md|SUMMARY.md|catalog|redirect|shelf" docs/authoring.md crates/mdbook-bookshelf/src/catalog.rs crates/mdbook-bookshelf/src/root_bookshelf_preprocessor.rs crates/mdbook-bookshelf/src/build.rs tests/input_catalog_build.rs`:
  passed for `docs-authoring-index-001`.

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
2026-04-24T06:58:09Z [coordinator] [docs-config-validation-001] [checkpoint] Created rework checkpoint a4e637a docs: refine config validation wording.
2026-04-24T06:58:41Z [reviewer] [docs-config-validation-001] [approved] Rework fixes parser test verification command and config validation docs match current behavior.
2026-04-24T06:59:05Z [reviewer-subagent] [docs-config-validation-001] [approved] Validation wording now matches enforced config parser behavior.
2026-04-24T06:59:28Z [coordinator] [docs-config-validation-001] [approved] Moved approved config validation checkpoints a44ec66 and a4e637a to chunk ledger.
2026-04-24T07:00:13Z [planner] [docs-authoring-index-001] [planned] Proposed focused authoring clarification for index.md assumptions versus SUMMARY.md validation.
2026-04-24T07:00:35Z [coordinator] [docs-authoring-index-001] [accepted] Accepted one-file authoring index contract clarification chunk.
2026-04-24T07:01:09Z [developer-subagent] [docs-authoring-index-001] [started] Started authoring index contract clarification against catalog and navigation code.
2026-04-24T07:01:40Z [developer-subagent] [docs-authoring-index-001] [completed] Clarified SUMMARY.md catalog validation versus index.md runtime navigation assumptions; required verification passed.
2026-04-24T07:02:28Z [coordinator] [docs-authoring-index-001] [checkpoint] Created review checkpoint 1aefd5f docs: clarify authoring entry page assumptions.
2026-04-24T07:04:23Z [reviewer] [docs-authoring-index-001] [changes-required] Requested narrower site-root redirect wording for bookshelf entry-page mode.
2026-04-24T07:04:23Z [developer-subagent] [docs-authoring-index-001] [completed] Narrowed redirect wording to default root-book site-root redirect; required verification passed.
2026-04-24T07:05:02Z [coordinator] [docs-authoring-index-001] [checkpoint] Created rework checkpoint e789107 docs: refine authoring redirect wording.
2026-04-24T07:05:32Z [reviewer] [docs-authoring-index-001] [approved] Redirect wording now matches current configurable site-root behavior.
2026-04-24T07:05:55Z [reviewer-subagent] [docs-authoring-index-001] [approved] Rework fixes redirect overstatement while keeping authoring edit localized.
2026-04-24T07:06:18Z [coordinator] [docs-authoring-index-001] [approved] Moved approved authoring index checkpoints 1aefd5f and e789107 to chunk ledger.
2026-04-24T07:06:47Z [planner] [docs-config-shared-assets-001] [planned] Proposed focused shared output asset and per-book HTML config documentation chunk.
2026-04-24T07:07:10Z [coordinator] [docs-config-shared-assets-001] [accepted] Accepted one-file shared output assets configuration drift cleanup chunk.
2026-04-24T07:07:31Z [developer-subagent] [docs-config-shared-assets-001] [started] Started shared output asset and per-book HTML config documentation correction against bookshelf_ui and build_cli tests.
2026-04-24T07:08:46Z [developer-subagent] [docs-config-shared-assets-001] [completed] Documented shared CSS/JS config-root resolution, child root staging, per-book preprocessors/JS, and input-404 opt-out; required diff verification passed.
