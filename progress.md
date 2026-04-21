# Bookshelf mdBook-First Implementation Progress

## Objective
- Implement the bookshelf feature on top of mdBook to satisfy the bookshelf handoff acceptance criteria and test plan.

## Global Constraints
- mdBook-first integration.
- No primary HTML post-processing architecture.
- No primary temp-workspace-copy architecture.
- No mdBook fork unless concretely blocked and explicitly approved.
- Use Cargo dependencies on mdBook crates rather than the local `mdBook-repo` checkout.
- `bookshelf.toml` is the single human-owned config.
- One canonical `SUMMARY.md` per book.
- `Bookshelf` page is generated in memory.

## Integration Strategy
- Preferred path: build a Rust `mdbook-bookshelf` driver that loads bookshelf config, reuses mdBook-compatible book loading/parsing seams per source book, builds one explicit bookshelf-owned site model, and renders the final multi-book HTML shell directly instead of post-processing stock HTML.
- Reused mdBook seams: `mdbook_summary::parse_summary`, `mdbook_driver::MDBook::load_with_config_and_summary` for single-book loading, optional `MDBook::preprocess_book` for mdBook-compatible preprocessing, and mdBook-like serve/watch flow patterns where practical. Avoid `MDBook::build()` and `mdbook_html::HtmlHandlebars` because they lock in single-book shell behavior.
- Cargo mdBook dependencies: add published mdBook crates to this crate and treat them as the implementation boundary rather than copying mdBook internals or editing `mdBook-repo`.
- Explicit non-goals: modifying `mdBook-repo`, reviving the archived HTML post-processing prototype as the primary architecture, merging all books into one sidebar, or copying full book trees into a staged workspace as the core design.

## Permissions
- Planner: read repo, write only `progress.md`.
- Researcher: read repo and mdBook source for reference only, write only `progress.md`, never edit `mdBook-repo`.
- Developer: read/write repo, read `mdBook-repo` for reference only, run build/test/format/validation, update `progress.md`, create one commit per developer iteration, no destructive git ops, never edit `mdBook-repo`.
- Reviewer-Subagent: read repo, run checks, write only `progress.md`.
- Reviewer-Claude: read-only; coordinator mirrors its `ProgressNote`.
- Coordinator: orchestrates and may commit on behalf of the developer when needed.

## Current State
- Status: READY_FOR_NEXT_CHUNK
- Current iteration: CHUNK-007 implementation
- Current chunk: CHUNK-007
- Next action: build a renderer-agnostic site-wide search document model with owning-book labels.
- Blockers: none recorded.

## Open Risks
- The repo has no active implementation code under `src/`, so the first chunk must establish the initial mdBook-first scaffold without drifting into a clean-room generator.
- Mapping each `bookshelf.toml` entry to a valid mdBook `Config`/book root boundary may expose path assumptions that need careful tests.
- Pulling in published mdBook crates may require network access or version adjustments before local verification works.
- The archived prototype may contain useful domain logic, but its architecture cannot be adopted wholesale if it depends on HTML post-processing or staging copies.

## Active Chunk
```yaml
chunk_id: CHUNK-007
title: Build renderer-agnostic site-wide search document model with owning-book labels
objective: Derive a deterministic search data model from the bookshelf site model so every searchable content page yields a labeled search document that includes owning-book identity for reader-visible results.
why_now: Search labeling is a direct acceptance requirement and the smallest remaining behavior layer that can be completed before sidebar/render/build/serve integration.
depends_on:
  - CHUNK-001
  - CHUNK-002
  - CHUNK-003
  - CHUNK-004
  - CHUNK-005
  - CHUNK-006
mdbook_touchpoints:
  - reuse: consume mdBook-loaded content already normalized into bookshelf-owned site/page metadata
  - avoid: no `MDBook::build()`, no `mdbook_html` searcher pipeline coupling, no HTML/template output
scope_in:
  - define typed search-document structs for site-wide indexing inputs
  - emit one deterministic search document per searchable content page across all books
  - include required owning-book label metadata on each document
  - exclude synthetic root `Bookshelf` page from searchable content documents
  - add focused tests for cross-book coverage and label correctness
scope_out:
  - no frontend search UI behavior
  - no JS index serialization format coupling
  - no sidebar data generation
  - no renderer/build/serve wiring
target_files:
  - src/search_model.rs
  - src/lib.rs
  - tests/search_model_build.rs
  - tests/fixtures/search-model/valid/bookshelf.toml
implementation_tasks:
  - add `build_search_documents(...)` API from existing site/navigation metadata
  - map each content page to stable search fields such as id/title/content text plus owning-book label
  - enforce deterministic document ordering for reproducible output/tests
  - skip synthetic/non-content pages explicitly
  - add regression tests for multi-book site-wide inclusion and per-document book-label presence
acceptance_criteria:
  - search model includes documents from all content books, site-wide
  - each search document includes owning-book label metadata suitable for reader-visible results
  - synthetic root `Bookshelf` page is not emitted as a searchable content document
  - output ordering and identifiers are deterministic across runs
  - chunk adds no HTML rendering or build/serve behavior
verification:
  - command: `cargo test search_model_build -- --exact`
    expect: exits 0 and validates site-wide inclusion, label presence, synthetic-page exclusion, and determinism
  - command: `cargo check --offline`
    expect: exits 0
review_focus:
  - owning-book label presence is enforced for every emitted search document
  - site-wide search scope is complete while honoring non-content exclusions
  - model stays renderer-agnostic to preserve separation before rendering integration
```

## Chunk Ledger
- CHUNK-001 approved via commits `007cca2` and `55866ca`: added published mdBook crate dependencies, a minimal `load_single_book_with_summary()` seam around `parse_summary` + `load_with_config_and_summary`, and a passing deterministic seam smoke test. Verified with `cargo check --offline` and `cargo test seam_single_book_load -- --exact`.
- CHUNK-002 approved via commits `4473c11` and `e0aafec`: added typed `bookshelf.toml` loading with TOML-native deserialization, structural validation for catalog/root/path invariants, and passing regression coverage for standard mdBook tables, inline comments, escaped quotes, duplicate IDs, missing root membership, and empty catalogs. Verified with `cargo test bookshelf_config_parse -- --exact` and `cargo check --offline`.
- CHUNK-003 approved via commits `55d1cb5`, `6181cd6`, and `5027c6c`: added a validated multi-book input catalog with canonical configured-summary ownership checks, deterministic root/src normalization including top-level `docs/` roots, and passing coverage for valid nested/top-level layouts plus missing or wrong-location summary cases. Verified with `cargo test input_catalog_build -- --exact` and `cargo check --offline`.
- CHUNK-004 approved via commits `963f4bc` and `5d71ebd`: added deterministic multi-book loading into parsed canonical `Summary` plus in-memory `MDBook`, threading one parsed summary through `MDBook::load_with_config_and_summary`, and passing coverage for successful multi-book load, malformed-summary parse failure, and mdBook-load failure with book-scoped context. Verified with `cargo test multi_book_load -- --exact` and `cargo check --offline`.
- CHUNK-005 approved via commits `66ca7d6` and `5dc7bb3`: added the first explicit bookshelf-owned site model with deterministic per-book ownership/order metadata and exactly one synthetic root `Bookshelf` page generated in memory, owned by the configured root book and excluded from content-book page order. Verified with `cargo test site_model_build -- --exact` and `cargo check --offline`.
- CHUNK-006 approved via commit `4b2a949`: added deterministic navigation metadata with strict intra-book prev/next boundaries, exact `Book / Page` breadcrumbs for content pages, and page-id-keyed active-book resolution for content plus synthetic Bookshelf pages, all without rendering logic. Verified with `cargo test navigation_metadata -- --exact` and `cargo check --offline`.

## Final Validation
- Pending

## Activity Log
- 2026-04-20T13:12:26Z [coordinator] [INIT] [STARTED] Created progress.md and recorded startup constraints, tentative integration strategy, and initial risks.
- 2026-04-20T13:14:46Z [researcher] [STARTUP] [DONE] Confirmed minimal mdBook seam: load single authored book via `MDBook::load_with_config_and_summary` from driver-owned config/summary and avoid stock HTML by not invoking `MDBook::build()`.
- 2026-04-20T13:16:34Z [planner] [STARTUP] [DONE] Selected next chunk: add mdBook Cargo dependencies and a seam-proof single-book load smoke test around `parse_summary` + `load_with_config_and_summary`.
- 2026-04-20T13:18:33Z [developer] [CHUNK-001] [STARTED] Began dependency wiring and seam smoke-test implementation for single-book `parse_summary` + `load_with_config_and_summary`.
- 2026-04-20T13:21:25Z [developer] [CHUNK-001] [DONE] Added mdBook seam scaffold, single-book load API, and deterministic seam smoke test with passing checks.
- 2026-04-20T13:24:21Z [coordinator] [CHUNK-001] [REVIEWING] Recorded developer commit `007cca2`; offline verification regenerated `Cargo.lock`, which is being treated as a review focus item.
- 2026-04-20T13:37:00Z [reviewer-subagent] [CHUNK-001] CHANGES_REQUIRED - Seam/test acceptance is met, but dependency changes were committed without the corresponding tracked `Cargo.lock` update.
- 2026-04-20T13:42:27Z [reviewer-claude] [CHUNK-001] APPROVED - Minimal mdBook-first seam via published crates with deterministic single-book load test; commit Cargo.lock as a small follow-up.
- 2026-04-20T15:12:14Z [developer] [CHUNK-001] [STARTED] Began review-fix iteration to include generated Cargo.lock and rerun offline seam verification.
- 2026-04-20T15:12:30Z [developer] [CHUNK-001] [DONE] Added Cargo.lock to chunk scope and revalidated offline cargo check plus exact seam smoke test.
- 2026-04-20T15:13:52Z [reviewer-subagent] [CHUNK-001] APPROVED - Prior blocker is fixed by committing Cargo.lock, and CHUNK-001 seam checks pass offline with exact test selection.
- 2026-04-20T15:13:56Z [reviewer-claude] [CHUNK-001] APPROVED - minimal published-crate seam with deterministic single-book load test and no build/HTML invocation.
- 2026-04-20T15:20:00Z [reviewer-subagent] [CHUNK-001] APPROVED - Prior blocker is fixed by committing Cargo.lock, and CHUNK-001 seam checks pass offline with exact test selection.
- 2026-04-20T15:14:40Z [planner] [CHUNK-002] [DONE] Selected minimal next chunk: implement `bookshelf.toml` typed parsing plus structural validation as the gate before multi-book catalog loading.
- 2026-04-20T15:16:43Z [developer] [CHUNK-002] [STARTED] Began typed `bookshelf.toml` parser/validator implementation and focused config topology tests.
- 2026-04-20T15:24:36Z [developer] [CHUNK-002] [DONE] Added typed config loading/validation with fixtures for valid config, duplicate IDs, missing root membership, and empty catalog plus passing offline checks.
- 2026-04-20T15:26:40Z [reviewer-subagent] [CHUNK-002] CHANGES_REQUIRED - Handwritten line parser is not TOML-compliant and can reject valid coexisting mdBook tables (non-quoted values, inline comments, escaping), so config fidelity is not met.
- 2026-04-20T15:27:17Z [reviewer-claude] [CHUNK-002] CHANGES_REQUIRED - handwritten line parser misreads valid TOML (inline comments, non-string values in `[book]`/`[output.html]`); switch to serde+`toml`.
- 2026-04-20T15:34:35Z [developer] [CHUNK-002] [DONE] Replaced line parsing with serde-plus-toml deserialization, added TOML-semantics regressions, and revalidated offline.
- 2026-04-20T15:36:49Z [reviewer-subagent] [CHUNK-002] APPROVED - Parser now uses serde+toml with TOML-semantics regression coverage while preserving required structural validation gates.
- 2026-04-20T15:37:04Z [reviewer-claude] [CHUNK-002] APPROVED - serde+toml parser satisfies config fidelity alongside standard mdBook tables and meets all acceptance criteria.
- 2026-04-20T15:33:18Z [developer] [CHUNK-003] [DONE] Added input catalog builder with canonical per-book summary ownership checks and passing fixture-based catalog tests.
- 2026-04-20T15:44:20Z [reviewer-subagent] [CHUNK-003] CHANGES_REQUIRED - Catalog builder hard-codes `<book-root>/src/SUMMARY.md`, which conflicts with accepted `bookshelf.toml` summary roots like `docs/SUMMARY.md` and `modules/*/docs/SUMMARY.md`.
- 2026-04-20T15:44:32Z [reviewer-claude] [CHUNK-003] CHANGES_REQUIRED - catalog builder hard-codes `<book-root>/src/SUMMARY.md`, conflicting with the spec's accepted `docs/`-style source roots and handoff example.
- 2026-04-20T15:54:18Z [developer] [CHUNK-003] [DONE] Removed fixed src-root assumptions, derived book root/src from configured summary paths, and revalidated catalog tests offline.
- 2026-04-20T15:58:04Z [reviewer-subagent] [CHUNK-003] CHANGES_REQUIRED - `src` hard-code is removed, but top-level `docs/SUMMARY.md` root-book layout is not positively covered and `book_root_rel` may remain an empty path instead of normalized `.`.
- 2026-04-20T15:58:16Z [reviewer-claude] [CHUNK-003] CHANGES_REQUIRED - top-level `docs/SUMMARY.md` root-book pattern accepted by code but never exercised as a success case; root-path normalization is also untested/ambiguous.
- 2026-04-20T16:39:06Z [developer] [CHUNK-003] [DONE] Added top-level docs-root success fixture/assertions and enforced deterministic `book_root_rel` normalization to `.`.
- 2026-04-21T01:22:41Z [reviewer-subagent] [CHUNK-003] APPROVED - Catalog now normalizes top-level roots to `.` and includes passing positive coverage for `docs/SUMMARY.md` root-book layouts.
- 2026-04-21T01:22:41Z [reviewer-claude] [CHUNK-003] APPROVED - canonical summary ownership and root/src normalization, including top-level `.`, are enforced and tested within scope.
- 2026-04-21T01:28:00Z [developer] [CHUNK-004] [DONE] Added deterministic multi-book loader results with parsed summaries, in-memory mdBook loads, and fixture-backed success plus malformed-summary coverage.
- 2026-04-21T01:30:33Z [reviewer-subagent] [CHUNK-004] APPROVED - Loader stays within seam-only scope, preserves catalog order, and provides deterministic book-scoped parse/load failures; duplicate summary parsing is acceptable in this chunk.
- 2026-04-21T01:30:33Z [reviewer-claude] [CHUNK-004] CHANGES_REQUIRED - loader re-parses each canonical SUMMARY.md instead of threading the parsed Summary into the approved seam, violating the single-source-of-truth invariant.
- 2026-04-21T01:32:44Z [developer] [CHUNK-004] [DONE] Removed summary double-parse by threading parsed Summary into seam-based loads and added mdBook-load failure propagation coverage.
- 2026-04-21T01:34:50Z [reviewer-subagent] [CHUNK-004] APPROVED - Loader now threads one parsed Summary into the seam and includes deterministic coverage for both summary-parse and mdBook-load failures per book.
- 2026-04-21T01:34:50Z [reviewer-claude] [CHUNK-004] APPROVED - parsed summary threaded through approved seam; deterministic per-book loading with book-id-scoped error context and no rendering work.
- 2026-04-21T01:40:00Z [developer] [CHUNK-005] [DONE] Added deterministic bookshelf-owned site model with one synthetic root Bookshelf page and per-book ownership/order metadata plus passing site-model checks.
- 2026-04-21T01:42:02Z [reviewer-subagent] [CHUNK-005] APPROVED - Site model is explicit and deterministic with exactly one root-owned synthetic Bookshelf page; current content route/id format is acceptable as internal placeholder metadata for this pre-navigation chunk.
- 2026-04-21T01:42:02Z [reviewer-claude] [CHUNK-005] CHANGES_REQUIRED - `SitePage.route` adds out-of-scope URL semantics and a latent `/` collision with the synthetic Bookshelf; defer routing to the navigation chunk.
- 2026-04-21T01:44:08Z [developer] [CHUNK-005] [DONE] Removed route metadata from the site model and revalidated synthetic-page plus ownership/order invariants.
- 2026-04-21T01:46:45Z [reviewer-subagent] [CHUNK-005] APPROVED - Route metadata was removed as requested; model remains deterministic and focused on ownership/order plus synthetic Bookshelf invariants.
- 2026-04-21T01:46:45Z [reviewer-claude] [CHUNK-005] APPROVED - model cleanly captures ownership plus deterministic order with synthetic root and no out-of-scope routing concerns.
- 2026-04-21T01:51:22Z [developer] [CHUNK-006] [DONE] Added deterministic navigation metadata with book-scoped prev/next, exact `Book / Page` breadcrumbs, and active-book resolution plus passing checks.
- 2026-04-21T01:54:32Z [reviewer-subagent] [CHUNK-006] APPROVED - Navigation metadata enforces book-scoped prev/next with exact `Book / Page` breadcrumbs; page-id keyed active-book resolution is sufficient at this pre-route stage.
- 2026-04-21T01:54:32Z [reviewer-claude] [CHUNK-006] APPROVED - book-scoped prev/next, `Book / Page` breadcrumbs, and page-id-keyed active-book resolution satisfy the pre-HTML, renderer-agnostic navigation contract.
- 2026-04-20T15:18:45Z [developer] [CHUNK-002] [DONE] Added strict typed config loader/validation and passing parse tests for valid, duplicate-id, missing-root, and empty-catalog cases.
- 2026-04-20T15:32:00Z [reviewer-subagent] [CHUNK-002] CHANGES_REQUIRED - Handwritten line parser is not TOML-compliant and can reject valid coexisting mdBook tables (non-quoted values, inline comments, escaping), so config fidelity is not met.
- 2026-04-20T15:24:29Z [developer] [CHUNK-002] [STARTED] Began review-fix to replace line parsing with TOML deserialization and add regression coverage for comments and quoting semantics.
- 2026-04-20T15:26:30Z [developer] [CHUNK-002] [DONE] Replaced config parsing with serde+toml, added TOML semantics regressions, and reran required offline checks.
- 2026-04-20T15:41:00Z [reviewer-subagent] [CHUNK-002] APPROVED - Parser now uses serde+toml with TOML-semantics regression coverage while preserving required structural validation gates.
- 2026-04-20T15:30:07Z [planner] [CHUNK-003] [DONE] Selected minimal next chunk: derive validated multi-book input catalog and enforce canonical per-book `SUMMARY.md` ownership before site-model work.
- 2026-04-20T15:31:47Z [developer] [CHUNK-003] [STARTED] Began input catalog builder implementation with canonical per-book SUMMARY ownership checks and fixture-backed tests.
- 2026-04-20T15:33:18Z [developer] [CHUNK-003] [DONE] Added input catalog builder with canonical per-book summary ownership checks and passing fixture-based catalog tests.
- 2026-04-20T15:50:00Z [reviewer-subagent] [CHUNK-003] CHANGES_REQUIRED - Catalog builder hard-codes `<book-root>/src/SUMMARY.md`, which conflicts with accepted `bookshelf.toml` summary roots like `docs/SUMMARY.md` and `modules/*/docs/SUMMARY.md`.
- 2026-04-20T15:36:29Z [developer] [CHUNK-003] [STARTED] Began review-fix to remove fixed src-root assumptions and derive book root/src from configured summary paths.
- 2026-04-20T15:38:31Z [developer] [CHUNK-003] [DONE] Updated catalog path derivation to honor configured source roots and revalidated canonical configured-summary ownership checks.
- 2026-04-21T10:05:00Z [reviewer-subagent] [CHUNK-003] CHANGES_REQUIRED - `src` hard-code is removed, but top-level `docs/SUMMARY.md` root-book layout is not positively covered and `book_root_rel` may remain an empty path instead of normalized `.`.
- 2026-04-20T16:37:59Z [developer] [CHUNK-003] [STARTED] Began review-fix to add top-level docs-root success coverage and enforce deterministic `book_root_rel = \".\"` normalization.
- 2026-04-20T16:39:06Z [developer] [CHUNK-003] [DONE] Added top-level docs-root success fixture/assertions and enforced deterministic `book_root_rel` normalization to `.`.
- 2026-04-21T10:15:00Z [reviewer-subagent] [CHUNK-003] APPROVED - Catalog now normalizes top-level roots to `.` and includes passing positive coverage for `docs/SUMMARY.md` root-book layouts.
- 2026-04-21T01:23:27Z [planner] [CHUNK-004] [DONE] Selected minimal next chunk: load each catalog entry into parsed canonical `Summary` plus in-memory `MDBook` and return deterministic per-book load results.
- 2026-04-21T01:24:46Z [developer] [CHUNK-004] [STARTED] Began multi-book loader implementation for canonical summary parsing plus in-memory per-book mdBook loads.
- 2026-04-21T01:27:40Z [developer] [CHUNK-004] [DONE] Added deterministic multi-book load pipeline with per-book summary parsing, seam-based in-memory MDBook loading, and book-scoped failure attribution.
- 2026-04-21T10:28:00Z [reviewer-subagent] [CHUNK-004] APPROVED - Loader stays within seam-only scope, preserves catalog order, and provides deterministic book-scoped parse/load failures; duplicate summary parsing is acceptable in this chunk.
- 2026-04-21T01:31:15Z [developer] [CHUNK-004] [STARTED] Began review-fix to remove summary double-parse and thread parsed Summary directly into the mdBook seam.
- 2026-04-21T01:32:44Z [developer] [CHUNK-004] [DONE] Removed summary double-parse by threading parsed Summary into seam-based loads and added mdBook-load failure propagation coverage.
- 2026-04-21T10:40:00Z [reviewer-subagent] [CHUNK-004] APPROVED - Loader now threads one parsed Summary into the seam and includes deterministic coverage for both summary-parse and mdBook-load failures per book.
- 2026-04-21T01:35:37Z [planner] [CHUNK-005] [DONE] Selected minimal next chunk: construct a deterministic bookshelf-owned site model (books + owned pages + synthetic root Bookshelf placeholder) from loaded books, without rendering logic.
- 2026-04-21T01:37:11Z [developer] [CHUNK-005] [STARTED] Began deterministic bookshelf-owned site-model implementation from loaded books with synthetic root Bookshelf page invariants.
- 2026-04-21T01:39:05Z [developer] [CHUNK-005] [DONE] Added deterministic site-model builder with synthetic root Bookshelf page and verified ownership/order invariants from loaded books.
- 2026-04-21T10:55:00Z [reviewer-subagent] [CHUNK-005] APPROVED - Site model is explicit and deterministic with exactly one root-owned synthetic Bookshelf page; current content route/id format is acceptable as internal placeholder metadata for this pre-navigation chunk.
- 2026-04-21T01:42:43Z [developer] [CHUNK-005] [STARTED] Began review-fix to remove premature route metadata and keep CHUNK-005 model strictly ownership/order focused.
- 2026-04-21T01:43:24Z [developer] [CHUNK-005] [DONE] Removed route metadata from site model/tests and revalidated CHUNK-005 ownership/order invariants.
- 2026-04-21T11:02:00Z [reviewer-subagent] [CHUNK-005] APPROVED - Route metadata was removed as requested; model remains deterministic and focused on ownership/order plus synthetic Bookshelf invariants.
- 2026-04-21T01:48:02Z [planner] [CHUNK-006] [DONE] Selected minimal next chunk: derive deterministic navigation metadata (book-scoped prev/next, `Book / Page` breadcrumbs, and active-book context resolution) from the site model without rendering.
- 2026-04-21T01:55:24Z [planner] [CHUNK-007] [DONE] Selected minimal next chunk: build renderer-agnostic site-wide search document model with required owning-book labels from the current site and navigation metadata.
- 2026-04-21T01:49:25Z [developer] [CHUNK-006] [STARTED] Began navigation-metadata implementation for book-scoped prev/next, `Book / Page` breadcrumbs, and active-book target resolution.
- 2026-04-21T01:52:14Z [developer] [CHUNK-006] [DONE] Added deterministic renderer-agnostic navigation metadata with book-scoped prev/next, `Book / Page` breadcrumbs, and active-book resolution.
- 2026-04-21T11:18:00Z [reviewer-subagent] [CHUNK-006] APPROVED - Navigation metadata enforces book-scoped prev/next with exact `Book / Page` breadcrumbs; page-id keyed active-book resolution is sufficient at this pre-route stage.
- 2026-04-21T01:56:46Z [developer] [CHUNK-007] [STARTED] Began renderer-agnostic site-wide search model implementation with per-document owning-book labels and synthetic-page exclusion.
- 2026-04-21T01:59:07Z [developer] [CHUNK-007] [DONE] Added deterministic site-wide search document model with owning-book labels and synthetic-page exclusion from emitted documents.
