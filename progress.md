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
- Status: CHUNK_READY
- Current iteration: CHUNK-002 implementation
- Current chunk: CHUNK-002
- Next action: implement strict `bookshelf.toml` parsing and structural validation ahead of multi-book catalog loading.
- Blockers: none recorded.

## Open Risks
- The repo has no active implementation code under `src/`, so the first chunk must establish the initial mdBook-first scaffold without drifting into a clean-room generator.
- Mapping each `bookshelf.toml` entry to a valid mdBook `Config`/book root boundary may expose path assumptions that need careful tests.
- Pulling in published mdBook crates may require network access or version adjustments before local verification works.
- The archived prototype may contain useful domain logic, but its architecture cannot be adopted wholesale if it depends on HTML post-processing or staging copies.

## Active Chunk
```yaml
chunk_id: CHUNK-002
title: Add `bookshelf.toml` typed parser with structural validation
objective: Introduce a strict in-crate `bookshelf.toml` loader that parses into typed Rust structs and validates minimal structural invariants required before multi-book catalog loading.
why_now: Multi-book loading cannot be implemented safely until the human-owned top-level config is reliably parsed and rejected on invalid topology.
depends_on:
  - CHUNK-001
mdbook_touchpoints:
  - avoid: no `MDBook` load calls in this chunk; config validation is completed before invoking mdBook seams
  - prepare_for_reuse: produce validated per-book roots/src inputs that the next chunk can pass to `parse_summary` + `load_with_config_and_summary`
scope_in:
  - define `bookshelf.toml` schema structs for site-level settings and declared books
  - implement parse-from-path API with actionable validation errors
  - validate required structural rules: non-empty catalog, unique book IDs, configured root book present in the declared books, and path normalization checks
  - add focused tests for valid config and representative failure modes
scope_out:
  - no per-book `SUMMARY.md` parsing yet
  - no `MDBook` loading yet
  - no rendering/site-model logic
target_files:
  - src/lib.rs
  - src/config.rs
  - tests/bookshelf_config_parse.rs
  - tests/fixtures/bookshelf-config/valid/bookshelf.toml
  - tests/fixtures/bookshelf-config/invalid-duplicate-id/bookshelf.toml
  - tests/fixtures/bookshelf-config/invalid-missing-root/bookshelf.toml
implementation_tasks:
  - add a public `load_bookshelf_config(path)` entrypoint returning typed config plus validation
  - encode validation rules with deterministic error messages for reviewable assertions
  - ensure relative paths are resolved against config directory without workspace copying
  - add table-driven tests for valid config and each invalid topology case
acceptance_criteria:
  - valid fixture config parses successfully into typed structs
  - duplicate book IDs fail validation with a specific deterministic error
  - missing root book membership fails validation with a specific deterministic error
  - empty book catalog fails validation
  - chunk introduces no `MDBook::build()` or stock HTML rendering usage
verification:
  - command: `cargo test bookshelf_config_parse -- --exact`
    expect: exits 0 and covers one valid plus multiple invalid config cases
  - command: `cargo check --offline`
    expect: exits 0
review_focus:
  - config schema fidelity to `bookshelf.toml` as the single human-owned source
  - validation strictness is sufficient to gate next chunk for catalog + summary ownership checks
  - error outputs are stable and assertion-friendly for future review loops
```

## Chunk Ledger
- CHUNK-001 approved via commits `007cca2` and `55866ca`: added published mdBook crate dependencies, a minimal `load_single_book_with_summary()` seam around `parse_summary` + `load_with_config_and_summary`, and a passing deterministic seam smoke test. Verified with `cargo check --offline` and `cargo test seam_single_book_load -- --exact`.

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
- 2026-04-20T15:18:45Z [developer] [CHUNK-002] [DONE] Added strict typed config loader/validation and passing parse tests for valid, duplicate-id, missing-root, and empty-catalog cases.
