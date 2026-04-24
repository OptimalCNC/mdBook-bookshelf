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
chunk_id: docs-linking-001
title: Document bookshelf site-root link rewrite rules
objective: Correct docs/linking.md so authors can predict exactly which links mdbook-bookshelf rewrites, preserves, or rejects.
why_now: This is a narrow, high-signal drift fix with direct unit-test evidence and only one reference page to review.
depends_on: []
touchpoints:
  - crates/mdbook-bookshelf/src/site_root_link_preprocessor.rs
  - docs/linking.md
scope_in:
  - Clarify that only single-slash /... link and image destinations are bookshelf site-root-relative.
  - Document that local, parent, fragment-only, external, mailto, and protocol-relative destinations are left unchanged.
  - Document that query and fragment suffixes are preserved when site-root links are rewritten.
  - Document that unresolved site-root .md page links fail the build.
  - Document that README.md is accepted as an index.md target for site-root page links.
scope_out:
  - Do not change link rewriting behavior.
  - Do not edit configuration, authoring, search, or operations docs.
  - Do not add broad navigation restructuring beyond this page.
target_files:
  - docs/linking.md
implementation_tasks:
  - Replace the current broad links that start with / wording with precise single-slash site-root behavior.
  - Add compact examples for rewritten cross-book page links, images, query/fragment preservation, and unchanged non-site-root destinations.
  - Add a short failure note for unresolved /... .md links.
  - Add a README.md/index.md alias note without implying index.md is globally required.
acceptance_criteria:
  - docs/linking.md matches the tested behavior in site_root_link_preprocessor.rs.
  - The page distinguishes rewritten destinations from preserved destinations.
  - The page avoids suggesting /... .html authoring for internal book pages.
  - The change is reviewable as a single-page documentation correction.
verification:
  - command: cargo test site_root_link_preprocessor
    expect: Link rewrite unit tests pass.
  - command: cargo run --bin book -- build bookshelf.toml --dest-dir .tmp/docs-linking-001-site
    expect: Project documentation builds successfully with the updated linking page.
review_focus:
  - Check for any wording that implies all slash-prefixed URLs are rewritten, including // protocol-relative URLs.
  - Check that README.md-as-index.md is described as link-target compatibility, not as an authoring requirement.
```

# Chunk Ledger
- `docs-nav-001`: approved in commit `768be49`; added audience routing in
  `README.md` and `docs/index.md`, added `docs/operations.md`, linked it from
  `docs/SUMMARY.md`, and corrected the root `Bookshelf` sidebar wording to the
  implemented first unnumbered entry behavior.

# Final Validation
- `cargo test --test cli_help --test bookshelf_config_parse`: passed for
  `docs-nav-001`.
- `cargo test --test build_cli build_cli_emits_bookshelf_ui_assets_without_fixture_residue`:
  passed for `docs-nav-001`.
- `cargo test --test serve_cli serve_cli_serves_built_site`: passed for
  `docs-nav-001`.
- `cargo run --bin book -- build bookshelf.toml --dest-dir .tmp/project-docs-site`:
  passed for `docs-nav-001`.

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
