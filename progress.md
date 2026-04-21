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
- User-guided priority:
  - first priority is a working website HTML build path analogous to mdBook's core build flow
  - preserve the bookshelf project layout with multiple canonical `SUMMARY.md` files
  - treat `bookshelf.toml` as the human-owned source of truth and map the applicable pieces into mdBook `Config` rather than introducing a separate authoring model
  - defer search work until after website build works; when search resumes, prefer reusing mdBook library support over custom search behavior
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
- Current iteration: CHUNK-008 implementation
- Current chunk: CHUNK-008
- Next action: improve built HTML pages with acceptance-critical navigation chrome driven by existing metadata.
- Blockers: none recorded.

## Open Risks
- The repo has no active implementation code under `src/`, so the first chunk must establish the initial mdBook-first scaffold without drifting into a clean-room generator.
- Mapping each `bookshelf.toml` entry to a valid mdBook `Config`/book root boundary may expose path assumptions that need careful tests.
- Pulling in published mdBook crates may require network access or version adjustments before local verification works.
- The archived prototype may contain useful domain logic, but its architecture cannot be adopted wholesale if it depends on HTML post-processing or staging copies.
- The current HEAD includes CHUNK-007 search-model work (`4f090a4`) that is intentionally deferred from review while the build-to-HTML path is reprioritized.

## Active Chunk
```yaml
chunk_id: CHUNK-008
title: Render navigation chrome from existing metadata in built HTML pages
objective: Improve website build fidelity by wiring current site/navigation metadata into generated page chrome: Bookshelf return control, active-book-scoped sidebar, `Book / Page` breadcrumbs, and book-bounded prev/next links.
why_now: The first build path exists; this is the smallest high-impact step to make output behavior align with acceptance criteria without introducing search or full theme parity.
depends_on:
  - CHUNK-001
  - CHUNK-002
  - CHUNK-003
  - CHUNK-004
  - CHUNK-005
  - CHUNK-006
  - CHUNK-007B
mdbook_touchpoints:
  - reuse: existing bookshelf-owned `SiteModel` and `NavigationMetadata` as rendering inputs
  - reuse: `build_html_site()` pipeline and mdBook-config projection already in place
  - avoid: no top-level `MDBook::build()` and no direct `mdbook_html::HtmlHandlebars` final rendering
scope_in:
  - update HTML rendering layer to include a visible Bookshelf return control on every content page
  - render sidebar entries scoped to active book only, excluding other books' chapter trees
  - render breadcrumbs in exact `Book / Page` format on content pages
  - render prev/next links from existing book-scoped navigation metadata
  - add focused build-output assertions for these chrome elements
scope_out:
  - no search model, index, or result labeling
  - no serve/watch integration
  - no full mdBook theme or assets parity
target_files:
  - src/build_html.rs
  - src/render_html.rs
  - src/lib.rs
  - tests/build_html_navigation_chrome.rs
  - tests/fixtures/build-html-nav/valid/bookshelf.toml
implementation_tasks:
  - thread active-page navigation context into page rendering
  - emit Bookshelf return control markup on each content page
  - emit active-book-only sidebar tree markup for each content page
  - emit breadcrumb and prev/next markup using deterministic metadata from CHUNK-006
  - add integration-style fixture test that inspects generated HTML files for required chrome semantics
acceptance_criteria:
  - every generated content page contains a visible Bookshelf return control linking to root bookshelf page
  - content-page sidebar shows only the active book's pages, with no cross-book sidebar pollution
  - content pages render breadcrumbs exactly as `Book / Page`
  - prev/next links exist only within the same book and respect first/last boundaries
  - chunk introduces no search or index logic
verification:
  - command: `cargo test build_html_navigation_chrome -- --exact`
    expect: exits 0 and validates return control, scoped sidebar, breadcrumb format, and intra-book prev/next behavior in generated HTML
  - command: `cargo check --offline`
    expect: exits 0
review_focus:
  - renderer fidelity improvements are strictly driven by existing metadata, not ad hoc HTML heuristics
  - acceptance-critical navigation chrome now appears in built pages without cross-book leakage
  - scope remains website-build fidelity only, with search still deferred
```

## Chunk Ledger
- CHUNK-001 approved via commits `007cca2` and `55866ca`: added published mdBook crate dependencies, a minimal `load_single_book_with_summary()` seam around `parse_summary` + `load_with_config_and_summary`, and a passing deterministic seam smoke test. Verified with `cargo check --offline` and `cargo test seam_single_book_load -- --exact`.
- CHUNK-002 approved via commits `4473c11` and `e0aafec`: added typed `bookshelf.toml` loading with TOML-native deserialization, structural validation for catalog/root/path invariants, and passing regression coverage for standard mdBook tables, inline comments, escaped quotes, duplicate IDs, missing root membership, and empty catalogs. Verified with `cargo test bookshelf_config_parse -- --exact` and `cargo check --offline`.
- CHUNK-003 approved via commits `55d1cb5`, `6181cd6`, and `5027c6c`: added a validated multi-book input catalog with canonical configured-summary ownership checks, deterministic root/src normalization including top-level `docs/` roots, and passing coverage for valid nested/top-level layouts plus missing or wrong-location summary cases. Verified with `cargo test input_catalog_build -- --exact` and `cargo check --offline`.
- CHUNK-004 approved via commits `963f4bc` and `5d71ebd`: added deterministic multi-book loading into parsed canonical `Summary` plus in-memory `MDBook`, threading one parsed summary through `MDBook::load_with_config_and_summary`, and passing coverage for successful multi-book load, malformed-summary parse failure, and mdBook-load failure with book-scoped context. Verified with `cargo test multi_book_load -- --exact` and `cargo check --offline`.
- CHUNK-005 approved via commits `66ca7d6` and `5dc7bb3`: added the first explicit bookshelf-owned site model with deterministic per-book ownership/order metadata and exactly one synthetic root `Bookshelf` page generated in memory, owned by the configured root book and excluded from content-book page order. Verified with `cargo test site_model_build -- --exact` and `cargo check --offline`.
- CHUNK-006 approved via commit `4b2a949`: added deterministic navigation metadata with strict intra-book prev/next boundaries, exact `Book / Page` breadcrumbs for content pages, and page-id-keyed active-book resolution for content plus synthetic Bookshelf pages, all without rendering logic. Verified with `cargo test navigation_metadata -- --exact` and `cargo check --offline`.
- CHUNK-007B approved via commits `259f3b7` and `e95bbd6`: added the first working `build_html_site()` orchestration from `bookshelf.toml` through catalog/load/site-model/navigation into emitted HTML files, with minimal but material mdBook-config projection affecting output via per-page language and default-theme markers while preserving bookshelf-owned per-book titles. Verified with `cargo test build_html_site_smoke -- --exact` and `cargo check --offline`.

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
- 2026-04-21T02:22:42Z [coordinator] [REPRIORITIZE] [DONE] Recorded user-guided priority override: HTML build path first, preserve multiple canonical `SUMMARY.md`, map `bookshelf.toml` into mdBook `Config`, and defer search until after website build works.
- 2026-04-21T10:34:00Z [researcher] [HTML-SEAM] [DONE] Recommended bookshelf-owned HTML build pipeline: project `bookshelf.toml` into minimal per-book mdBook configs, reuse mdBook loading/preprocessing seams, and emulate only thin stock build/serve wrappers.
- 2026-04-21T02:22:42Z [planner] [CHUNK-007B] [DONE] Replanned next chunk to first working `build_html_site()` orchestration, explicitly deferring review of committed CHUNK-007 search-model work until after website build priority is satisfied.
- 2026-04-21T02:31:36Z [developer] [CHUNK-007B] [DONE] Added first working HTML build orchestration with config projection and fixture-backed smoke validation for root plus multi-book content pages.
- 2026-04-21T02:36:57Z [reviewer-subagent] [CHUNK-007B] CHANGES_REQUIRED - Projected per-book mdBook `Config` is currently decorative rather than driving load/preprocess/build behavior, so config-projection fidelity is not materially realized.
- 2026-04-21T02:36:57Z [reviewer-claude] [CHUNK-007B] CHANGES_REQUIRED - global `[book]` table is projected identically onto every book, breaking per-book titles on the Bookshelf landing page and masked by a too-lax smoke test.
- 2026-04-21T02:40:20Z [developer] [CHUNK-007B] [DONE] Made projected mdBook config materially affect emitted HTML, preserved distinct per-book bookshelf titles, and tightened build smoke coverage.
- 2026-04-21T02:43:46Z [reviewer-subagent] [CHUNK-007B] APPROVED - Projected mdBook config now materially affects emitted HTML (lang/theme) while preserving bookshelf-owned per-book titles; minimal projection scope is acceptable for this first build chunk.
- 2026-04-21T02:43:46Z [reviewer-claude] [CHUNK-007B] APPROVED - minimal end-to-end HTML build is real, deterministic, and mdBook-first via projected Config while honoring declared scope and avoid constraints.
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
- 2026-04-21T02:24:51Z [planner] [CHUNK-007B] [DONE] Re-prioritized next chunk to first working HTML build path: implement bookshelf-owned `build_html_site()` orchestration and minimal per-book mdBook `Config` projection, deferring search.
- 2026-04-21T02:44:51Z [planner] [CHUNK-008] [DONE] Selected minimal next chunk: upgrade HTML build fidelity by rendering navigation chrome from existing metadata (Bookshelf return control, scoped sidebar, breadcrumbs, and book-bounded prev/next).
- 2026-04-21T01:49:25Z [developer] [CHUNK-006] [STARTED] Began navigation-metadata implementation for book-scoped prev/next, `Book / Page` breadcrumbs, and active-book target resolution.
- 2026-04-21T01:52:14Z [developer] [CHUNK-006] [DONE] Added deterministic renderer-agnostic navigation metadata with book-scoped prev/next, `Book / Page` breadcrumbs, and active-book resolution.
- 2026-04-21T11:18:00Z [reviewer-subagent] [CHUNK-006] APPROVED - Navigation metadata enforces book-scoped prev/next with exact `Book / Page` breadcrumbs; page-id keyed active-book resolution is sufficient at this pre-route stage.
- 2026-04-21T01:56:46Z [developer] [CHUNK-007] [STARTED] Began renderer-agnostic site-wide search model implementation with per-document owning-book labels and synthetic-page exclusion.
- 2026-04-21T01:59:07Z [developer] [CHUNK-007] [DONE] Added deterministic site-wide search document model with owning-book labels and synthetic-page exclusion from emitted documents.
- 2026-04-21T02:31:36Z [developer] [CHUNK-007B] [STARTED] Began first working HTML build-path implementation with mdBook-config projection and bookshelf-owned file emission.
- 2026-04-21T02:31:36Z [developer] [CHUNK-007B] [DONE] Added first working HTML build orchestration with config projection and fixture-backed smoke validation for root plus multi-book content pages.
- 2026-04-21T10:34:00Z [researcher] [HTML-SEAM] [DONE] Recommended minimal mdBook-first seam: project `bookshelf.toml` into per-book `mdbook_core::Config`, keep multi-book composition in a custom renderer path, and emulate only thin stock build/serve wrappers next.
- 2026-04-21T11:35:00Z [reviewer-subagent] [CHUNK-007B] CHANGES_REQUIRED - Projected per-book mdBook `Config` is currently decorative (used mainly for root title text) rather than driving load/preprocess/build behavior, so config-projection fidelity is not materially realized.
- 2026-04-21T02:38:04Z [developer] [CHUNK-007B] [STARTED] Began review-fix to make projected mdBook config materially drive emitted HTML behavior and preserve distinct bookshelf per-book titles.
- 2026-04-21T02:40:11Z [developer] [CHUNK-007B] [DONE] Made projected config materially affect emitted HTML (lang/theme), preserved per-book bookshelf titles, and tightened smoke assertions.
- 2026-04-21T11:48:00Z [reviewer-subagent] [CHUNK-007B] APPROVED - Projected mdBook config now materially affects emitted HTML (lang/theme) while preserving bookshelf-owned per-book titles; minimal projection scope is acceptable for this first build chunk.
- 2026-04-21T02:46:19Z [developer] [CHUNK-008] [STARTED] Began HTML navigation chrome rendering upgrade using existing site/navigation metadata with active-book scoped output.
- 2026-04-21T02:50:09Z [developer] [CHUNK-008] [DONE] Rendered navigation chrome from metadata (Bookshelf return, active-book sidebar, breadcrumbs, and book-bounded prev/next) with passing build output checks.
