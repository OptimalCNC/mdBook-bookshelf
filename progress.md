# Bookshelf mdBook-First Implementation Progress

## Objective
- Implement `mdbook-bookshelf` as an mdBook-powered multi-book integration layer with `build` and `serve` as the primary user-facing targets, satisfying the bookshelf handoff acceptance criteria and test plan.

## Global Constraints
- mdBook-first integration.
- `mdbook-bookshelf` is an mdBook-powered multi-book integration layer, not a conventional mdBook plugin and not a standalone generator.
- No primary HTML post-processing architecture.
- No primary temp-workspace-copy architecture.
- No mdBook fork unless concretely blocked and explicitly approved.
- Use Cargo dependencies on mdBook crates rather than the local `mdBook-repo` checkout.
- `bookshelf.toml` is the single human-owned config.
- One canonical `SUMMARY.md` per book.
- `Bookshelf` page is generated in memory.
- `build` and `serve` are the primary user-facing targets.
- The bookshelf pipeline should stay as close as practical to stock mdBook's build and serve pipeline.
- mdBook should remain responsible for the single-book heavy lifting wherever practical.

## Integration Strategy
- Preferred path: build a Rust `mdbook-bookshelf` driver that keeps mdBook responsible for single-book loading, parsing, preprocessing, rendering, and stock-like output behavior wherever practical, while `mdbook-bookshelf` owns only the multi-book orchestration and composition layer.
- Reused mdBook seams retained in the current baseline: `mdbook_summary::parse_summary` and `mdbook_driver::MDBook::load_with_config_and_summary` for per-book summary parsing and in-memory book loading.
- Accepted correction seam for the current root-book mismatch: keep CHUNK-011's stock per-book `MDBook::build()` path, but for the configured root book inject one synthetic in-memory `Bookshelf` chapter into the loaded `Book` before stock preprocessing/rendering so the `Bookshelf` page is mdBook-rendered inside the root book instead of being a handwritten standalone chooser page.
- Corrected placement rule after direction review: the synthetic root-book `Bookshelf` page must be a trailing unnumbered affix page at a stable non-index path such as `bookshelf.html`; do not insert it at the front of the root book or otherwise depend on stock first-chapter/index overwrite behavior.
- User-confirmed extension policy for CHUNK-014 review-fix: `output.html.additional-js` and `output.html.additional-css` remain an acceptable mdBook-first seam. The remaining defect to fix is build residue under the caller-owned tree, not the use of mdBook's standard additional asset hooks.
- Pipeline similarity target: keep the bookshelf `build` and `serve` flow structurally close to stock mdBook and diverge only where the multi-book data model requires it.
- CLI similarity target: keep the binary structured as a `main.rs` dispatcher plus `cmd/*` subcommand modules on a `clap` seam, with `build` and `serve` implementations remaining library-owned rather than embedded in manual argument parsing.
- Cargo mdBook dependencies: use published mdBook crates as the implementation boundary rather than copying mdBook internals or editing `mdBook-repo`.
- Retained foundations after the direction reset: typed `bookshelf.toml` parsing, multi-book input catalog validation, per-book mdBook loading, explicit site ownership/order modeling, and navigation metadata.
- Root-book Bookshelf invariant: the `Bookshelf` page must be a page in the configured root book, not a separate site-root-only chooser page. Any top-level entry behavior must resolve into that root-book-owned page rather than replace it with a standalone shell.
- Root-book content-root invariant: the synthetic `Bookshelf` page must not replace the root book's canonical content-root page under `books/<root-book-id>/index.html`; the site root may use only a thin entry file to open the root-book-owned `Bookshelf` page at its own stable path.
- Removed due to direction drift: custom HTML emission, custom page shell rendering, synthetic public `pNNNN.html` routing, custom build CLI behavior, custom config projection for the bespoke renderer, and generator-specific tests/fixtures.
- Historical mismatch now corrected: CHUNK-012 introduced a standalone handwritten site-root chooser page. Approved CHUNK-013 replaced that rejected shape with a thin site-root redirect into the root-book-owned synthetic `Bookshelf` page.
- Explicit non-goals: modifying `mdBook-repo`, using mdBook merely as a parser/preprocessor feeding a separate site generator, reviving the archived HTML post-processing prototype as the primary architecture, merging all books into one sidebar, or copying full book trees into a staged workspace as the core design.

## Permissions
- Planner: read repo, write only `progress.md`.
- Researcher: read repo and mdBook source for reference only, write only `progress.md`, never edit `mdBook-repo`.
- Developer: read/write repo, read `mdBook-repo` for reference only, run build/test/format/validation, update `progress.md`, create one commit per developer iteration, no destructive git ops, never edit `mdBook-repo`.
- Reviewer: read repo, run narrow direction checks, write only `progress.md`.
- Reviewer-Subagent: read repo, run implementation checks, write only `progress.md`.
- Reviewer-Claude: read-only; coordinator mirrors its `ProgressNote`.
- Coordinator: orchestrates and may commit on behalf of the developer when needed.

## Current State
- Status: READY_FOR_IMPLEMENTATION
- Current iteration: CHUNK-017 planning
- Current chunk: CHUNK-017 stock TOC sidebar scope and deep-link activation proof
- Next action: implement CHUNK-017 to lock the already-present active-book sidebar scope, deep-link activation, and root-book `Bookshelf` affix behavior on the stock per-book TOC seam before taking on site-wide search.
- Blockers: none.

## Open Risks
- The root-book correction seam depends on post-load mutation of the public `MDBook.book` because `MDBook::load_with_config_and_summary()` cannot hydrate synthetic in-memory chapter content directly; future chunks must preserve that boundary unless a concrete blocker is recorded.
- The synthetic root-book page must remain an unnumbered affix entry and must not steal the root book's canonical content-root `index.html`, so the corrected CHUNK-013 path must place it as a trailing affix page with a stable separate path plus a thin site-root entry file.
- CHUNK-014's approved return-control seam depends on the stock `.right-buttons` header container and on build-time generated `additional-js` / `additional-css` inputs being cleaned up reliably so successful builds leave no residue under the caller-owned tree.
- mdBook mirrors `additional-js` / `additional-css` input paths into the emitted site output, so CHUNK-014 currently leaves build-specific `.mdbook-bookshelf/build-*` asset directories under each built book output tree; this is acceptable for the chunk but worth flattening in a later polish pass if those paths become reader-visible or operationally awkward.
- Active-book sidebar scope and deep-link activation currently appear to fall out of stock per-book `toc.html` / `toc.js` output, but that acceptance seam is not yet locked by executable bookshelf-specific coverage.
- The public mdBook crate APIs may still prove insufficient for the eventual multi-book shell, in which case the gap must be documented explicitly before any replacement behavior is implemented.
- Pulling in additional published mdBook crates or versions may require network access or version adjustments before local verification works.

## Active Chunk
```yaml
chunk_id: CHUNK-017
title: Stock TOC Sidebar Scope And Deep-Link Activation Proof
objective: Add executable acceptance coverage proving that stock per-book mdBook sidebar output already provides active-book-scoped navigation, deep-link activation, and the root-book `Bookshelf` affix behavior in the built multi-book site.
why_now: With CHUNK-016 closing the missing top-level `serve` workflow, the next smallest acceptance gap is to lock the already-present active-book context behavior down with bookshelf-specific runtime coverage before taking on site-wide search.
depends_on:
  - CHUNK-006
  - CHUNK-011
  - CHUNK-013
  - CHUNK-014
  - CHUNK-015
  - CHUNK-016
mdbook_touchpoints:
  - reuse: stock per-book `MDBook::build()` emission of `toc.html` and hashed `toc-*.js`, relying on mdBook's built-in sidebar tree and active-link selection instead of custom sidebar rendering
  - reuse: mdBook HTML theme behavior in `crates/mdbook-html/front-end/templates/toc.js.hbs`, where the active entry is derived from `document.location` plus `path_to_root`
  - avoid: custom sidebar HTML/JS, merged cross-book sidebar generation, or HTML post-processing to fake active-book context
scope_in:
  - extend the self-contained build smoke test with a small runtime harness for emitted `toc-*.js` so deep links into parser and UI pages must activate the matching sidebar entry
  - assert that parser and UI sidebar trees remain book-scoped and do not include foreign-book chapters
  - assert that the root-book `toc.html` contains `Bookshelf` as a trailing unnumbered affix entry after the numbered root-book chapters
scope_out:
  - no search implementation or search result labeling
  - no new `serve` behavior
  - no new sidebar rendering logic or theme overrides
target_files:
  - tests/build_cli.rs
implementation_tasks:
  - add a build-test harness that executes emitted `toc-*.js` against synthetic deep-link URLs and reports the active sidebar entry plus visible chapter labels
  - assert parser deep links activate `Grammar` within the parser sidebar tree and never expose UI chapters, and assert UI deep links activate `Navigation` within the UI sidebar tree and never expose parser chapters
  - assert the root-book sidebar HTML includes `Bookshelf` as an unnumbered affix entry after the numbered root-book chapters so the synthetic page remains root-book-owned without affecting numbering
  - preserve existing CHUNK-013 through CHUNK-016 smoke assertions while tightening coverage around the stock TOC seam instead of adding any new rendering code
acceptance_criteria:
  - `cargo test` passes with updated build CLI coverage
  - building the self-contained example proves, via the emitted parser `toc-*.js`, that a direct link to `books/parser/grammar.html` activates `Grammar` and exposes only parser-book sidebar entries
  - building the self-contained example proves, via the emitted UI `toc-*.js`, that a direct link to `books/ui/navigation.html` activates `Navigation` and exposes only UI-book sidebar entries
  - building the self-contained example proves `books/meta/toc.html` includes a trailing unnumbered `Bookshelf` entry after the numbered `Example Core` chapters
  - the existing site-root redirect, `Bookshelf` return control, breadcrumb assertions, and top-level `serve` workflow still pass unchanged
verification:
  - command: cargo test
    expect: build CLI coverage passes, including runtime assertions over emitted `toc-*.js` for parser and UI deep-link activation plus root-book affix-sidebar assertions
  - command: cargo run -- build bookshelf/handoffs/examples/self-contained/bookshelf.toml --dest-dir .tmp/chunk-017-smoke
    expect: `.tmp/chunk-017-smoke/books/parser/toc.html`, `.tmp/chunk-017-smoke/books/parser/toc-*.js`, `.tmp/chunk-017-smoke/books/ui/toc-*.js`, and `.tmp/chunk-017-smoke/books/meta/toc.html` prove book-scoped sidebar trees, direct-link active entry selection, and a trailing unnumbered root-book `Bookshelf` affix
review_focus:
  - verify the chunk reuses stock mdBook sidebar assets and active-link logic instead of adding bookshelf-owned sidebar rendering or route rewriting
  - verify the new assertions check runtime deep-link activation, not only static HTML presence
  - verify the root-book `Bookshelf` page remains a synthetic affix entry owned by the root book and does not alter chapter numbering
```

## Chunk Ledger
- CHUNK-001 approved via commits `007cca2` and `55866ca`: added published mdBook crate dependencies, a minimal `load_single_book_with_summary()` seam around `parse_summary` + `load_with_config_and_summary`, and a passing deterministic seam smoke test. Verified with `cargo check --offline` and `cargo test seam_single_book_load -- --exact`.
- CHUNK-002 approved via commits `4473c11` and `e0aafec`: added typed `bookshelf.toml` loading with TOML-native deserialization, structural validation for catalog/root/path invariants, and passing regression coverage for standard mdBook tables, inline comments, escaped quotes, duplicate IDs, missing root membership, and empty catalogs. Verified with `cargo test bookshelf_config_parse -- --exact` and `cargo check --offline`.
- CHUNK-003 approved via commits `55d1cb5`, `6181cd6`, and `5027c6c`: added a validated multi-book input catalog with canonical configured-summary ownership checks, deterministic root/src normalization including top-level `docs/` roots, and passing coverage for valid nested/top-level layouts plus missing or wrong-location summary cases. Verified with `cargo test input_catalog_build -- --exact` and `cargo check --offline`.
- CHUNK-004 approved via commits `963f4bc` and `5d71ebd`: added deterministic multi-book loading into parsed canonical `Summary` plus in-memory `MDBook`, threading one parsed summary through `MDBook::load_with_config_and_summary`, and passing coverage for successful multi-book load, malformed-summary parse failure, and mdBook-load failure with book-scoped context. Verified with `cargo test multi_book_load -- --exact` and `cargo check --offline`.
- CHUNK-005 approved via commits `66ca7d6` and `5dc7bb3`: added the first explicit bookshelf-owned site model with deterministic per-book ownership/order metadata and exactly one synthetic root `Bookshelf` page generated in memory, owned by the configured root book and excluded from content-book page order. Verified with `cargo test site_model_build -- --exact` and `cargo check --offline`.
- CHUNK-006 approved via commit `4b2a949`: added deterministic navigation metadata with strict intra-book prev/next boundaries, exact `Book / Page` breadcrumbs for content pages, and page-id-keyed active-book resolution for content plus synthetic Bookshelf pages, all without rendering logic. Verified with `cargo test navigation_metadata -- --exact` and `cargo check --offline`.
- CHUNK-011 approved via commits `3512051` and `209d682`: restored a real post-reset `build` path by projecting stock mdBook config from `bookshelf.toml`, rendering each configured book through `MDBook::load_with_config_and_summary` + `MDBook::build()` into isolated `books/<book-id>/` output roots, and adding smoke coverage plus a regression fixture proving relative mdBook config paths resolve from the `bookshelf.toml` directory. Verified with `cargo test` and `cargo run -- build bookshelf/handoffs/examples/self-contained/bookshelf.toml --dest-dir .tmp/chunk-011-smoke`.
- CHUNK-012 commit `babb913` added a synthetic site-root `index.html` chooser page that lists configured books with descriptions and links to each stock per-book mdBook root page under `books/<book-id>/`. On 2026-04-22, user review determined this is not an accepted baseline because `bookshelf/handoffs/01-target-site.md` requires `Bookshelf` to be a page inside the root book and the handwritten chooser shell visibly diverges from mdBook styling. Treat CHUNK-012 as a recorded mismatch to correct, not an approved seam to extend.
- CHUNK-013 review-fix iteration approved via corrective revert `ded1421` plus commit `10d70a9`: moved `Bookshelf` into the root book as a trailing synthetic mdBook-rendered affix page at `bookshelf.html`, preserved the authored root-book `index.html`, and replaced the standalone chooser page with a thin site-root redirect. Verified with `cargo test` and `cargo run -- build bookshelf/handoffs/examples/self-contained/bookshelf.toml --dest-dir .tmp/chunk-013-review-fix-smoke`.
- CHUNK-014 approved via commits `d916bd1` and `4ee6a2a`: added a visible `Bookshelf` return control through stock `output.html.additional-js` / `additional-css` injection while fixing transient asset cleanup so successful builds leave no caller-tree residue. Verified with `cargo test`.
- CHUNK-015 approved via commits `e89a5bd` and `900e271`: added exact `Book / Page` breadcrumbs on content pages through the approved per-book `additional-js` / `additional-css` seam, with runtime assertions proving exact text on root and non-root pages and no-op behavior on `bookshelf.html`, `print.html`, `toc.html`, and `404.html`. Verified with `cargo test` and `cargo run -- build bookshelf/handoffs/examples/self-contained/bookshelf.toml --dest-dir .tmp/chunk-015-smoke`.
- CHUNK-016 approved via commits `acf1832` and `7dc8ed2`: implemented the first real top-level `serve` workflow as a build-once wrapper around the existing multi-book `build` path, serving the resolved site root through a stock-like `axum`/`ServeDir` static server and fixing the loopback integration test to use child-selected ephemeral binding with read-back of the bound address. Verified with `cargo test` and `cargo test serve_cli_serves_built_site -- --exact`.
- CHUNK-007 through CHUNK-010 were removed from the active baseline on 2026-04-21 after direction review concluded the repo had drifted into a custom HTML generator path. The removed work included search placeholders, config projection for the bespoke renderer, custom HTML emission, synthetic public routing, custom build CLI behavior, and all generator-specific tests/fixtures.

## Final Validation
- Current HEAD remains on an approved mdBook-first path through CHUNK-016: stock per-book `MDBook::build()` output under `books/<book-id>/`, a thin site-root redirect into the root-book-owned synthetic `Bookshelf` page, a visible `Bookshelf` return control, exact runtime-verified `Book / Page` breadcrumbs on content pages, and a build-once top-level `serve` workflow over the same output tree. Final product acceptance still requires executable proof for active-book sidebar/deep-link context plus site-wide search, followed by the whole-system review against the acceptance criteria and test plan.

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
- 2026-04-21T02:50:09Z [developer] [CHUNK-008] [DONE] Rendered navigation chrome from metadata with Bookshelf return, active-book sidebar, exact breadcrumbs, and book-bounded prev/next in built HTML pages.
- 2026-04-21T03:21:11Z [reviewer-subagent] [CHUNK-008] APPROVED - Navigation chrome is metadata-driven and acceptance-critical invariants pass; current root-relative hrefs are acceptable interim output-path links for this pre-base-url stage.
- 2026-04-21T03:21:11Z [reviewer-claude] [CHUNK-008] CHANGES_REQUIRED - Root-absolute chrome hrefs commit to base-url semantics outside this build-fidelity chunk's scope; switch to page-relative links.
- 2026-04-21T03:23:44Z [developer] [CHUNK-008] [DONE] Switched navigation chrome hrefs to page-relative links and removed debug-only page detail lines from content output.
- 2026-04-21T03:26:30Z [reviewer-subagent] [CHUNK-008] APPROVED - Navigation chrome now uses page-relative hrefs and removes debug-only content lines while preserving metadata-driven sidebar, breadcrumb, and intra-book prev/next behavior.
- 2026-04-21T03:26:30Z [reviewer-claude] [CHUNK-008] APPROVED - chrome elements render per spec with page-relative hrefs, and scope stays within the stated bounds.
- 2026-04-21T03:35:00Z [developer] [CHUNK-009] [DONE] Added mdBook preprocessing into the HTML build pipeline with fixture-backed checks for include resolution and deterministic preprocess failure handling.
- 2026-04-21T05:22:42Z [reviewer-subagent] [CHUNK-009] CHANGES_REQUIRED - Preprocessing is wired, but using renderer context `markdown` and a manual unresolved-include heuristic is the wrong fidelity boundary for an HTML build pipeline.
- 2026-04-21T05:22:42Z [reviewer-claude] [CHUNK-009] APPROVED - preprocessing wired through `MDBook::preprocess_book` with projected config and book-scoped errors; test proves include resolution reaches HTML output.
- 2026-04-21T06:15:00Z [developer] [CHUNK-009] [DONE] Switched preprocessing to HTML-oriented renderer identity, removed manual include heuristic, and revalidated preprocessing effects plus failure reporting.
- 2026-04-21T06:23:28Z [reviewer-subagent] [CHUNK-009] APPROVED - Preprocessing now uses HTML renderer context with projected preprocessor config and deterministic book-scoped failures, and emitted content reflects mdBook-preprocessed chapter transforms.
- 2026-04-21T06:23:28Z [reviewer-claude] [CHUNK-009] APPROVED - mdBook preprocessing is now invoked via `preprocess_book` with HTML renderer identity and projected preprocessor config; escaped-markdown emission remains an acceptable later rendering follow-up.
- 2026-04-21T03:35:00Z [developer] [CHUNK-009] [DONE] Added mdBook preprocessing into the HTML build pipeline with fixture-backed checks for include resolution and deterministic preprocess failure handling.
- 2026-04-21T06:23:28Z [reviewer-subagent] [CHUNK-009] APPROVED - Preprocessing now uses HTML renderer context with projected preprocessor config and deterministic book-scoped failures, and emitted content reflects mdBook-preprocessed chapter transforms.
- 2026-04-21T06:23:28Z [reviewer-claude] [CHUNK-009] APPROVED - mdBook preprocessing is now invoked via `preprocess_book` with HTML renderer identity and projected preprocessor config; escaped-markdown emission remains an acceptable later rendering follow-up.
- 2026-04-21T03:35:00Z [developer] [CHUNK-009] [DONE] Added mdBook preprocessing into the HTML build pipeline with fixture-backed checks for include resolution and deterministic preprocess failure handling.
- 2026-04-21T05:22:42Z [reviewer-subagent] [CHUNK-009] CHANGES_REQUIRED - Preprocessing is wired, but using renderer context `markdown` and a manual unresolved-include heuristic is the wrong fidelity boundary for an HTML build pipeline.
- 2026-04-21T05:22:42Z [reviewer-claude] [CHUNK-009] APPROVED - preprocessing wired through `MDBook::preprocess_book` with projected config and book-scoped errors; test proves include resolution reaches HTML output.
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
- 2026-04-22T05:04:47Z [researcher] [ROOT-BOOK-SEAM] [DONE] `MDBook::load_with_config_and_summary()` cannot hydrate synthetic chapter content from summary alone; smallest reusable seam is to load the root book through that API, then mutate the returned `MDBook.book` so a first-position synthetic non-draft `Bookshelf` chapter with reserved `path` is present before stock preprocess/build.
- 2026-04-22T05:22:47Z [planner] [CHUNK-013] [DONE] Tightened CHUNK-013 after direction review: keep the root-book post-load mutation seam, but append the synthetic `Bookshelf` page as a trailing unnumbered affix at a stable non-index path so stock first-chapter/index behavior is never part of the solution.
- 2026-04-21T03:23:44Z [developer] [CHUNK-008] [DONE] Switched navigation chrome hrefs to page-relative links and removed debug-only page detail lines from content output.
- 2026-04-21T12:02:00Z [reviewer-subagent] [CHUNK-008] APPROVED - Navigation chrome is metadata-driven and acceptance-critical invariants pass; current root-relative hrefs are acceptable interim output-path links for this pre-base-url stage.
- 2026-04-21T12:16:00Z [reviewer-subagent] [CHUNK-008] APPROVED - Navigation chrome now uses page-relative hrefs and removes debug-only content lines while preserving metadata-driven sidebar, breadcrumb, and intra-book prev/next behavior.
- 2026-04-21T03:27:57Z [planner] [CHUNK-009] [DONE] Selected minimal next chunk: run mdBook preprocessing (`MDBook::preprocess_book`) per loaded book within `build_html_site()` so emitted HTML reflects mdBook-transformed content before rendering.
- 2026-04-21T06:26:14Z [planner] [CHUNK-010] [DONE] Selected minimal next chunk: add a narrow CLI build entrypoint so users can run one concrete command to generate the rough review site from a handoff `bookshelf.toml`.
- 2026-04-21T03:34:31Z [developer] [CHUNK-009] [STARTED] Began build-pipeline preprocessing integration using `MDBook::preprocess_book` with deterministic per-book error attribution.
- 2026-04-21T03:34:31Z [developer] [CHUNK-009] [DONE] Wired deterministic load->preprocess->render flow, emitted preprocessed chapter content, and added preprocess success/failure fixture coverage.
- 2026-04-21T05:23:39Z [developer] [CHUNK-009] [STARTED] Began review-fix to switch preprocess context to HTML renderer semantics and remove manual include-string failure heuristics.
- 2026-04-21T05:27:11Z [developer] [CHUNK-009] [DONE] Switched to HTML renderer preprocess context, removed manual include heuristic, and kept deterministic book-scoped preprocess failure attribution.
- 2026-04-21T12:30:00Z [reviewer-subagent] [CHUNK-009] CHANGES_REQUIRED - Preprocessing is wired, but using renderer context `markdown` (and a manual unresolved-include heuristic) is the wrong fidelity boundary for an HTML build pipeline; renderer context should be HTML-driven.
- 2026-04-21T13:40:00Z [reviewer-subagent] [CHUNK-009] APPROVED - Preprocessing now uses HTML renderer context with projected preprocessor config and deterministic book-scoped failures, and emitted content reflects mdBook-preprocessed chapter transforms.
- 2026-04-21T06:28:03Z [developer] [CHUNK-010] [STARTED] Began minimal build-only CLI implementation and handoff-example command smoke coverage.
- 2026-04-22T16:44:55Z [researcher] [SERVE-SEAM] [DONE] Recommended reusing stock `serve`'s build-once plus static-file Axum loop against the existing built site tree and explicitly deferring watch/live-reload because upstream watch roots assume one `MDBook`.
- 2026-04-22T16:43:53Z [planner] [CHUNK-016] [DONE] Selected the smallest next chunk: lock active-book sidebar scope, deep-link activation, and root-book `Bookshelf` affix behavior on the stock per-book TOC seam before search or `serve`.
- 2026-04-21T06:30:11Z [developer] [CHUNK-010] [DONE] Added build-only CLI command path, validated handoff-example build command, and confirmed emitted review-site artifacts.
- 2026-04-21T11:59:43Z [external-authority] [DIRECTION-RESET] [STARTED] Recorded user-requested direction reset, marked the custom HTML/generator path as invalid, and moved the repo to a halted-for-replan state.
- 2026-04-21T12:02:02Z [external-authority] [DIRECTION-RESET] [DONE] Removed custom HTML generation, bespoke renderer config plumbing, generator-specific CLI behavior, and associated tests/fixtures; retained only the mdBook-first foundations and revalidated them with `cargo test`.
- 2026-04-21T12:14:10Z [external-authority] [DIRECTION-RESET] [DONE] Prepared a commit checkpoint for the mdBook-powered reset, prompt tightening, and wrong-direction code removal.
- 2026-04-21T12:20:32Z [researcher] [POST-RESET-SEAM] [DONE] Recommended per-book stock build seam: reuse the existing summary-injection loader and end the first real build helper at `MDBook::build()`, leaving `serve` as a later thin wrapper over the same path.
- 2026-04-21T12:21:23Z [planner] [CHUNK-011] [DONE] Selected the first post-reset chunk: restore a real mdBook `build` path by projecting stock mdBook config from `bookshelf.toml` and invoking `MDBook::build()` per book into isolated output subdirectories.
- 2026-04-21T12:22:00Z [coordinator] [CHUNK-011] [STARTED] Accepted CHUNK-011, updated the active plan, and began the first post-reset implementation loop on the stock mdBook build seam.
- 2026-04-21T12:28:50Z [developer] [CHUNK-011] [DONE] Added stock per-book build orchestration, projected mdBook config parsing, a minimal `build` CLI path, and self-contained example smoke coverage; commit `3512051`.
- 2026-04-21T12:36:00Z [reviewer-subagent] [CHUNK-011] CHANGES_REQUIRED - Shared relative mdBook config paths were resolving against each book root instead of the `bookshelf.toml` directory; add a review fix plus regression coverage.
- 2026-04-21T12:42:02Z [developer] [CHUNK-011] [DONE] Rooted mdBook loads at the `bookshelf.toml` directory, preserved per-book `book.src`, and added shared-config-root regression coverage; commit `209d682`.
- 2026-04-21T12:59:12Z [reviewer] [CHUNK-011] [DONE] Approved commit `209d682` for preserving the mdBook-first build seam and avoiding architecture drift.
- 2026-04-21T13:01:06Z [reviewer-subagent] [CHUNK-011] APPROVED - Inspected commit `209d682d945b82a2fa06310905ee56f80554e216`, confirmed active chunk scope in `progress.md`, reviewed affected files, and passed targeted `build_cli` verification.
- 2026-04-21T13:01:56Z [reviewer-claude] [CHUNK-011] APPROVED iteration 2 fix anchors mdBook relative paths at bookshelf.toml dir; shared-asset regression test covers the fix; stock MDBook::build() direction preserved.
- 2026-04-21T13:01:56Z [coordinator] [CHUNK-011] [DONE] Re-ran `cargo test` and the self-contained `build` smoke command after review fixes, confirmed all reviewers approved, and moved CHUNK-011 to the ledger.
- 2026-04-21T16:44:22Z [planner] [CHUNK-012] [DONE] Selected the smallest acceptance-directed follow-up chunk: add a synthetic site-root Bookshelf chooser page on top of the approved stock per-book build seam.
- 2026-04-21T16:44:47Z [researcher] [ROOT-COMPOSITION-SEAM] [DONE] Recommended a bookshelf-owned site-root composer that emits only `index.html` above untouched stock per-book mdBook outputs.
- 2026-04-21T16:46:24Z [coordinator] [CHUNK-012] [STARTED] Accepted CHUNK-012 and began the next implementation loop for the synthetic site-root Bookshelf chooser page.
- 2026-04-21T16:49:54Z [developer] [CHUNK-012] [DONE] Added a post-build synthetic site-root chooser page and chooser-page assertions while preserving stock per-book mdBook outputs; commit `babb913`.
- 2026-04-21T16:52:31Z [reviewer] [CHUNK-012] APPROVED - Verified the commit adds only the synthetic root chooser page and does not reintroduce custom single-book rendering or HTML rewriting.
- 2026-04-21T16:52:31Z [reviewer-subagent] [CHUNK-012] APPROVED - Synthetic site-root chooser page is emitted after stock per-book builds, links target `books/<book-id>/index.html`, and narrow verification passed.
- 2026-04-22T00:00:00Z [reviewer-claude] [CHUNK-012] APPROVED synthetic site-root chooser added on top of unchanged stock per-book mdBook build; scope, direction, and tests all consistent with CHUNK-012.
- 2026-04-21T16:52:59Z [coordinator] [CHUNK-012] [DONE] Re-ran `cargo test` and the self-contained `build` smoke command, confirmed reviewer approval, and moved CHUNK-012 to the ledger.
- 2026-04-22T02:23:34Z [external-authority] [CHUNK-012] CHANGES_REQUIRED - User review rejected the standalone synthetic site-root chooser page because `Bookshelf` must be a page in the root book and should not diverge visibly from mdBook/root-book styling.
- 2026-04-22T02:23:34Z [coordinator] [DIRECTION-HARDENING] [DONE] Updated the coordinator prompt and current project status so future sessions treat the root-book `Bookshelf` invariant as a hard direction check and do not extend CHUNK-012's standalone chooser page.
- 2026-04-22T00:00:00Z [researcher] [CHUNK-013-SEAM] [DONE] Confirmed `load_with_config_and_summary()` cannot hydrate synthetic chapter content directly; accepted seam is post-load root-book `Book` mutation plus stock preprocess/build.
- 2026-04-22T05:36:19Z [planner] [CHUNK-014] [DONE] Selected the smallest next acceptance chunk: add the visible `Bookshelf` return control on content-book pages via stock mdBook `additional-js`/`additional-css` injection on top of the approved CHUNK-013 baseline.
- 2026-04-22T06:00:38Z [planner] [CHUNK-014] [DONE] Tightened CHUNK-014 after review: keep the visible return-control goal, but switch to a transient per-book theme overlay outside the caller-owned tree so build leaves no generated files behind in config/source paths.
- 2026-04-22T00:00:00Z [planner] [CHUNK-013] [DONE] Proposed the smallest correction chunk: generate a root-book-owned synthetic `Bookshelf` page and replace the standalone chooser with a thin site-root entry.
- 2026-04-22T00:00:00Z [coordinator] [CHUNK-013] [STARTED] Accepted CHUNK-013 with a tightened guardrail that the synthetic `Bookshelf` page must remain separate from the root book's canonical content-root `index.html`.
- 2026-04-22T00:00:00Z [reviewer] [CHUNK-013] CHANGES_REQUIRED - Front insertion of the synthetic page depends on stock first-chapter/index overwrite behavior, which violates the chunk's direction guardrail.
- 2026-04-22T00:00:00Z [reviewer-subagent] [CHUNK-013] APPROVED - Functional smoke checks passed for the thin site-root redirect, shelf links, and unchanged non-root outputs on the proposed implementation.
- 2026-04-22T00:00:00Z [reviewer-claude] [CHUNK-013] CHANGES_REQUIRED - mdBook-first direction is right, but the implementation still makes first-chapter/index semantics part of the design; tests failed to assert the root-book index/shelf separation directly.
- 2026-04-22T00:00:00Z [coordinator] [CHUNK-013] [REVERTED] Reverted developer commit `1a63f2d` with corrective revert `ded1421` after direction review rejected the front-inserted shelf-page seam.
- 2026-04-22T00:00:00Z [researcher] [CHUNK-013-REVIEW-FIX] [DONE] Recommended the smallest safe correction: keep the synthetic root-book page as a trailing suffix/affix page at `bookshelf.html`, upgraded post-load without first-chapter/index dependence.
- 2026-04-22T00:00:00Z [planner] [CHUNK-013-REVIEW-FIX] [DONE] Narrowed the next iteration to one review-fix chunk: trailing synthetic affix page plus explicit test coverage for distinct root-book `index.html` and `bookshelf.html`.
- 2026-04-22T00:00:00Z [developer] [CHUNK-013] [DONE] Implemented the review fix in commit `10d70a9` by appending the synthetic root-book `Bookshelf` page as a trailing top-level affix and strengthening root index/shelf separation tests.
- 2026-04-22T00:00:00Z [reviewer] [CHUNK-013] APPROVED - The corrected implementation keeps `Bookshelf` mdBook-rendered inside the root book, uses only a thin site-root redirect, and no longer depends on first-chapter/index overwrite behavior.
- 2026-04-22T00:00:00Z [reviewer-subagent] [CHUNK-013] APPROVED - The corrected smoke checks prove distinct root-book `index.html` and `bookshelf.html` roles, root shelf links do not loop, and non-root outputs stay on the CHUNK-011 seam.
- 2026-04-22T00:00:00Z [reviewer-claude] [CHUNK-013] APPROVED - Iteration 2 appends the Bookshelf chapter, preserves the root book's `index.html`, uses a thin site-root redirect, and tests cover all three invariants.
- 2026-04-22T00:00:00Z [coordinator] [CHUNK-013] [DONE] Re-ran `cargo test` and the self-contained build smoke command, confirmed all reviewers approved the corrected seam, and moved CHUNK-013 to the ledger.
- 2026-04-22T00:00:00Z [planner] [CHUNK-014] [DONE] Selected the next smallest acceptance-directed chunk: add a visible `Bookshelf` return control through stock mdBook `additional-js` and `additional-css` seams.
- 2026-04-22T00:00:00Z [researcher] [CHUNK-014-SEAM] [DONE] Confirmed the smallest seam is stock `output.html.additional-js` plus `additional-css`, using `path_to_root`, `rootBookId`, and the stock `.right-buttons` header container without template overrides.
- 2026-04-22T00:00:00Z [coordinator] [CHUNK-014] [STARTED] Accepted CHUNK-014 and began the next implementation loop on the stock mdBook asset-injection seam for the visible `Bookshelf` return control.
- 2026-04-22T00:00:00Z [developer] [CHUNK-014] [DONE] Implemented the first return-control iteration in commit `d916bd1` by generating build-time bookshelf UI assets and wiring them through stock `additional-js` / `additional-css`.
- 2026-04-22T00:00:00Z [reviewer] [CHUNK-014] CHANGES_REQUIRED - The return-control seam itself is mdBook-first, but generating `.mdbook-bookshelf/` assets under the caller-owned config/source root is a workflow regression to reject.
- 2026-04-22T00:00:00Z [reviewer-subagent] [CHUNK-014] CHANGES_REQUIRED - HTML wiring and target values are correct, but `build` dirties the fixture/config root by leaving generated `.mdbook-bookshelf/` files behind.
- 2026-04-22T00:00:00Z [reviewer-claude] [CHUNK-014] CHANGES_REQUIRED - Return-control is referenced through stock mdBook asset seams, but `build` materializes those inputs under the caller-owned source tree and tests do not assert the fixture stays clean.
- 2026-04-22T00:00:00Z [coordinator] [CHUNK-014] [REVERTED] Reverted developer commit `d916bd1` with corrective revert `e2d7b05` after all reviewers rejected the source-tree residue behavior, then cleaned the generated `.mdbook-bookshelf/` files from local fixtures.
- 2026-04-22T00:00:00Z [external-authority] [CHUNK-014] [DONE] User confirmed that `output.html.additional-js` / `additional-css` is an acceptable mdBook extension seam; the remaining issue is build residue, not the seam choice itself.
- 2026-04-22T05:05:42Z [planner] [CHUNK-013] [DONE] Selected the smallest correction chunk: inject a root-book-owned synthetic `Bookshelf` page via a stock mdBook preprocessor and replace the standalone chooser with a thin site-root redirect into that root-book page.
- 2026-04-22T05:23:29Z [researcher] [CHUNK-013-FOLLOWUP] [DONE] Smallest safe seam is a stable root-level synthetic affix page at `bookshelf.html` for the root book, positioned after authored content via summary-affix/order mutation, with a thin site-root entry file; do not splice it after chapter 1 or depend on first-chapter `index.html`.
- 2026-04-22T05:39:27Z [researcher] [CHUNK-014-ASSET-SEAM] [DONE] Smallest mdBook-first return-control seam is bookshelf-owned `output.html.additional-js` + `additional-css` injected into each per-book build, with JS targeting stock `#mdbook-menu-bar .right-buttons`, using `path_to_root` plus the root-book id to build the return href, and no-oping on `bookshelf.html`.
- 2026-04-22T06:00:35Z [researcher] [CHUNK-014-OUTPUT-ONLY-SEAM] [DONE] Output-tree-only salvage of `additional-js`/`additional-css` is possible but awkward because mdBook couples each path as both input locator and emitted filename; smallest clean seam is a generated per-book `output.html.theme` override under the output tree with `header.hbs` rendering a `Bookshelf` link from `path_to_root` and skipping `bookshelf.md`.
- 2026-04-22T10:55:01Z [reviewer] [CHUNK-014] APPROVED - Commit `4ee6a2a` keeps the visible return control on stock `additional-js` / `additional-css`, preserves the approved CHUNK-013 output shape, and successful builds leave no `.mdbook-bookshelf` residue in the caller-owned config/source tree.
- 2026-04-22T10:55:30Z [reviewer] [CHUNK-014] APPROVED - Root/non-root pages still reference the injected return assets with correct targets, the `bookshelf.html` self-skip guard is preserved, and verification confirmed no `.mdbook-bookshelf` residue remains under the checked fixture/config roots after build completion.
- 2026-04-22T00:00:00Z [developer] [CHUNK-014] [DONE] Implemented the residue-cleanup review fix in commit `4ee6a2a`, preserving the `additional-js` / `additional-css` seam while removing successful-build residue from the caller-owned tree.
- 2026-04-22T00:00:00Z [reviewer] [CHUNK-014] APPROVED - The corrected implementation still uses stock `additional-js` / `additional-css`, preserves the approved CHUNK-013 output shape, and leaves no generated helper residue after successful build.
- 2026-04-22T00:00:00Z [reviewer-subagent] [CHUNK-014] APPROVED - Root/non-root asset wiring and target values are correct, the `bookshelf.html` self-skip guard is intact, and fixture/config roots remain clean after build completion.
- 2026-04-22T00:00:00Z [reviewer-claude] [CHUNK-014] BLOCKED - Repeated Claude CLI review attempts failed with `ConnectionRefused`; chunk closure is waiting on reviewer infrastructure rather than on a known code defect.
- 2026-04-22T11:48:03Z [coordinator] [CHUNK-014] [RETRY-BLOCKED] Retried external Claude review on user request; the local Claude CLI/API still failed with `ConnectionRefused`, so chunk closure remains blocked on reviewer availability rather than on a known code defect.
- 2026-04-22T14:08:07Z [reviewer-claude] [CHUNK-014] APPROVED - transient config-root helper inputs are confined to the build, cleaned on success and Drop, verified residue-free by tests, and the return control is delivered purely through stock `additional-js` / `additional-css` without disturbing the CHUNK-013 output shape.
- 2026-04-22T14:10:22Z [coordinator] [CHUNK-014] [UNBLOCKED] Retried the documented Claude CLI review successfully and reran chunk verification (`cargo test` plus the self-contained build smoke), so CHUNK-014 is no longer waiting on reviewer infrastructure.
- 2026-04-22T14:15:00Z [planner] [CHUNK-015] [DONE] Selected the next smallest acceptance-directed chunk: exact `Book / Page` breadcrumbs on content pages via the approved stock `additional-js` / `additional-css` seam.
- 2026-04-22T14:15:00Z [coordinator] [CHUNK-015] [STARTED] Accepted CHUNK-015 and began the next implementation loop on the approved asset-injection seam for exact content-page breadcrumbs.
- 2026-04-23T00:00:00Z [developer] [CHUNK-015] [DONE] Implemented breadcrumb UI assets in commit `e89a5bd`, extending the approved transient `additional-js` / `additional-css` seam and preserving CHUNK-014 residue cleanup behavior.
- 2026-04-23T00:00:00Z [reviewer] [CHUNK-015] APPROVED - Breadcrumb rendering stays on stock per-book `MDBook::build()` output plus `additional-js` / `additional-css`, without theme overrides or HTML rewrites, and the synthetic root `Bookshelf` page remains separate.
- 2026-04-23T00:00:00Z [reviewer-subagent] [CHUNK-015] CHANGES_REQUIRED - Initial tests only proved asset injection and script contents, not exact runtime breadcrumb behavior or no-op behavior on special pages.
- 2026-04-23T00:00:00Z [researcher] [CHUNK-015-SEAM] [DONE] Confirmed CHUNK-015 stays on the smallest acceptable mdBook-first seam by using a content-page-only breadcrumb map on stock `additional-js` / `additional-css`, with unmapped special pages naturally no-oping at runtime.
- 2026-04-23T00:00:00Z [developer] [CHUNK-015] [DONE] Implemented the verification-only review fix in commit `900e271`, adding a Node-based runtime harness that proves exact breadcrumb insertion on content pages and no-op behavior on special pages.
- 2026-04-23T00:00:00Z [reviewer] [CHUNK-015] APPROVED - Stronger verification still stays on stock per-book `MDBook::build()` plus `additional-js` / `additional-css`, proves breadcrumbs only render on mapped content pages, and shows no caller-tree residue regression.
- 2026-04-23T00:00:00Z [reviewer-subagent] [CHUNK-015] APPROVED - Runtime tests now prove exact breadcrumb text on `architecture.html` and `grammar.html`, verify no-op on `bookshelf.html`/`print.html`/`404.html`/`toc.html`, and preserve CHUNK-014 behavior.
- 2026-04-23T00:00:00Z [reviewer-claude] [CHUNK-015] BLOCKED - The local Claude CLI/API failed again with `ConnectionRefused`, so chunk closure is waiting on reviewer infrastructure rather than on a known code defect.
- 2026-04-22T14:14:10Z [planner] [CHUNK-015] [DONE] Selected the smallest next acceptance chunk: render exact `Book / Page` breadcrumbs on content pages through stock `additional-js` / `additional-css`, deferring search and serve.
- 2026-04-22T16:21:13Z [researcher] [CHUNK-015-SEAM] [DONE] Confirmed exact `Book / Page` breadcrumbs can stay on the approved stock `additional-js` / `additional-css` seam by inserting one DOM node into `#mdbook-content main` from a content-page map; skipping `bookshelf.html`, `404.html`, `print.html`, and `toc.html` by omission keeps the chunk out of custom rendering and post-build rewriting.
- 2026-04-22T16:40:44Z [reviewer-claude] [CHUNK-015] APPROVED - Breadcrumbs delivered via the approved per-book additional-js/css seam with exact-text runtime assertions and CHUNK-014 behavior preserved.
- 2026-04-22T16:40:44Z [coordinator] [CHUNK-015] [DONE] Confirmed `cargo test` and the self-contained build smoke, cleared the external review block, and moved the project into CHUNK-016 planning.
- 2026-04-22T16:48:56Z [planner] [CHUNK-016] [DONE] Proposed a test-only stock-TOC proof chunk; coordinator rejected it after confirming a narrower real `serve` seam could materially advance the mdBook-like workflow.
- 2026-04-22T16:48:56Z [researcher] [CHUNK-016-SEAM] [DONE] Recommended a build-once top-level `serve` wrapper around `build_bookshelf(...)` plus the static `axum`/`ServeDir` half of stock `mdbook serve`, explicitly deferring watch and live-reload.
- 2026-04-22T16:49:40Z [coordinator] [CHUNK-016] [STARTED] Accepted a tighter CHUNK-016 around the missing top-level `serve` workflow and began the implementation loop on the stock static-file server seam.
- 2026-04-22T16:51:59Z [developer] [CHUNK-016] [STARTED] Began implementing build-once top-level `serve` by factoring the existing multi-book build path to return the resolved site root and wrapping it in a stock-like static file server.
- 2026-04-22T16:57:52Z [developer] [CHUNK-016] [BLOCKED] The delegated worker iteration was interrupted before code changes landed, so the coordinator completed the chunk locally instead of waiting on a stale worker state.
- 2026-04-22T16:57:52Z [coordinator] [CHUNK-016] [DONE] Implemented build-once top-level `serve` locally in commit `acf1832`, added loopback HTTP integration coverage, and revalidated with `cargo test` plus `cargo test serve_cli_serves_built_site -- --exact`.
- 2026-04-23T00:00:00Z [reviewer] [CHUNK-016] APPROVED - Serve stays a build-once wrapper around the existing per-book `MDBook::build()` output plus a stock-like `ServeDir` static server, while the site root still redirects into the root-book-owned `Bookshelf` page.
- 2026-04-22T17:00:39Z [reviewer-subagent] [CHUNK-016] CHANGES_REQUIRED - The new serve integration test still reserves a port in the parent and reuses it in the child, so the required deterministic ephemeral-port coverage remains flaky.
- 2026-04-22T17:03:02Z [coordinator] [CHUNK-016] [DONE] Fixed the serve integration test to start the child with `--port 0`, capture the bound loopback address from process output, and revalidated with `cargo test` plus `cargo test serve_cli_serves_built_site -- --exact`; commit `7dc8ed2`.
- 2026-04-22T17:03:02Z [reviewer-claude] [CHUNK-016] APPROVED - test-only race fix via `--port 0` plus read-back address; production serve path and deferred scope unchanged, tests green.
- 2026-04-22T17:03:02Z [reviewer] [CHUNK-016] APPROVED - Iteration-2 only makes the serve test deterministic; the chunk remains on the approved build-once `MDBook::build()` plus stock-like `ServeDir` top-level serve seam.
- 2026-04-22T17:07:05Z [reviewer-subagent] [CHUNK-016] APPROVED - `serve_cli` now uses child-selected ephemeral binding and probes the reported address, closing the parent-reserved-port finding.
- 2026-04-22T17:07:05Z [coordinator] [CHUNK-016] [DONE] All review gates approved the review-fix commit `7dc8ed2`, so CHUNK-016 is closed and the project is moving to CHUNK-017 planning.
- 2026-04-23T01:22:40Z [coordinator] [EXTERNAL-REORG] [STARTED] User requested a repo-layout cleanup before deeper implementation; checkpointed the post-CHUNK-016 baseline in commit `13323d3` and started moving library code under `crates/` plus CLI code under `cli/`.
- 2026-04-22T17:04:51Z [reviewer] [CHUNK-016] APPROVED - The iteration-2 fix only makes the serve test deterministic with child-selected ephemeral binding and reported-address capture; the chunk still stays on the approved build-once `MDBook::build()` plus stock-like `ServeDir` top-level serve seam with watch/live-reload deferred.
- 2026-04-23T01:22:40Z [coordinator] [EXTERNAL-REORG] [DONE] Reorganized the repo so library sources live under `crates/mdbook-bookshelf/src` and the CLI entrypoint lives under `cli/mdbook-bookshelf/src`, preserved package behavior through manifest path updates, and revalidated with `cargo test`.
- 2026-04-23T02:03:10Z [developer] [CLI-ARCH] [DONE] Refactored the binary to a `clap`-based `main.rs` plus `cmd/{build,serve,command_prelude}.rs` layout modeled on `mdBook/src`, keeping `build` and `serve` execution in library code and revalidating with `cargo test --bin mdbook-bookshelf --test cli_help --test build_cli --test serve_cli`.
- 2026-04-23T02:03:10Z [coordinator] [EXTERNAL-CLI-NAME] [STARTED] User requested that the compiled binary be renamed from `mdbook-bookshelf` to `mdbook`; updating the Cargo bin target, CLI help name, and integration-test binary lookups before revalidation.
- 2026-04-23T02:03:10Z [coordinator] [EXTERNAL-CLI-NAME] [DONE] Renamed the compiled binary target to `mdbook`, updated CLI help and integration-test binary lookups to match, and revalidated with `cargo test`.
