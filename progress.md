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
chunk_id: docs-nav-001
title: Role-based entry navigation and Bookshelf sidebar drift cleanup
objective: Make README.md, docs/index.md, and docs/SUMMARY.md route evaluators, site authors, CLI operators, and contributors to the right docs quickly, while correcting the current Bookshelf sidebar placement drift.
why_now: The first-entry docs are present but generic, and docs/site-behavior.md currently says the Bookshelf sidebar entry is trailing even though the implementation prepends it and tests assert it is first.
depends_on: []
touchpoints:
  - README.md
  - docs/index.md
  - docs/SUMMARY.md
  - docs/site-behavior.md
  - docs/operations.md
  - cli/mdbook-bookshelf/src/cmd/command_prelude.rs
  - cli/mdbook-bookshelf/src/cmd/serve.rs
  - crates/mdbook-bookshelf/src/root_bookshelf_preprocessor.rs
  - tests/build_cli.rs
scope_in:
  - Add a compact audience router to README.md and docs/index.md.
  - Add a focused CLI operators page under docs/ that documents only current build/serve behavior and flags.
  - Add the operators page to docs/SUMMARY.md.
  - Correct docs/site-behavior.md to say the synthetic Bookshelf sidebar entry is first/prepended and unnumbered.
scope_out:
  - No Rust behavior changes.
  - No broad rewrite of reference docs.
  - No planned features, compatibility notes, release packaging, or publishing instructions beyond current CLI behavior.
target_files:
  - README.md
  - docs/index.md
  - docs/SUMMARY.md
  - docs/site-behavior.md
  - docs/operations.md
implementation_tasks:
  - Rewrite the top README doc pointer into a short audience map for evaluators, authors, operators, and contributors.
  - Reshape docs/index.md so the audience map appears before detailed reference links and keeps the mdBook-first positioning.
  - Create docs/operations.md with current book build and book serve usage, default bookshelf.toml, --dest-dir, --hostname, --port, docs-build mdbook-mermaid requirement, and serve rebuild/live-reload behavior.
  - Insert the new operators page into docs/SUMMARY.md near the existing configuration/authoring material.
  - Replace the incorrect trailing sidebar entry wording in docs/site-behavior.md with the implemented first-entry behavior.
acceptance_criteria:
  - A new reader can choose an evaluator, site author, CLI operator, or contributor path from README.md and docs/index.md without reading the whole doc set.
  - CLI operator claims match the current clap definitions and serve tests.
  - Bookshelf sidebar wording matches insert(0, ...) and the build CLI assertion that Bookshelf is the first root-book sidebar link.
  - The docs remain mdBook-first and do not describe planned behavior.
verification:
  - command: cargo test --test cli_help --test bookshelf_config_parse
    expect: Passes; confirms documented CLI/config basics still match code.
  - command: cargo test --test build_cli build_cli_emits_bookshelf_ui_assets_without_fixture_residue
    expect: Passes; covers generated Bookshelf UI, sidebar order, scoped navigation, and search assets.
  - command: cargo test --test serve_cli serve_cli_serves_built_site
    expect: Passes; confirms serve builds and serves the current site behavior.
  - command: cargo run --bin book -- build bookshelf.toml --dest-dir .tmp/project-docs-site
    expect: Succeeds when mdbook-mermaid is installed; generated docs include the updated navigation pages.
review_focus:
  - Check that README.md and docs/index.md route audiences without duplicating full reference content.
  - Check docs/operations.md against cli/mdbook-bookshelf/src/cmd/command_prelude.rs and cli/mdbook-bookshelf/src/cmd/serve.rs.
  - Check docs/site-behavior.md against crates/mdbook-bookshelf/src/root_bookshelf_preprocessor.rs and tests/build_cli.rs.
  - Check that docs/SUMMARY.md stays easy to scan and does not over-nest the documentation set.
```

# Chunk Ledger
None yet.

# Final Validation
Pending.

# Activity Log
2026-04-24T06:34:26Z [coordinator] [setup] [started] Reset coordination artifact for documentation navigation and drift cleanup.
2026-04-24T06:36:12Z [planner] [docs-nav-001] [planned] Proposed role-based entry navigation, operations page, and Bookshelf sidebar drift cleanup.
2026-04-24T06:38:14Z [researcher-subagent] [docs-drift-audit] [done] Found sidebar drift plus missing CLI, serve, config, linking, authoring, and search caveats.
2026-04-24T06:39:02Z [coordinator] [docs-nav-001] [accepted] Accepted first docs navigation chunk and recorded concrete scope.
2026-04-24T06:42:42Z [developer-subagent] [docs-nav-001] [completed] Added role routing, operations docs, SUMMARY entry, and first-sidebar Bookshelf wording; targeted validation passed.
