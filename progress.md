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
- Status: STARTUP_IN_PROGRESS
- Current iteration: startup
- Current chunk: CHUNK-001
- Next action: execute CHUNK-001 to make the confirmed mdBook loading seam runnable and testable in this crate.
- Blockers: implementation crate paths exist in `Cargo.toml` but `src/` is absent; mdBook crate dependencies still need to be added and validated locally.

## Open Risks
- The repo has no active implementation code under `src/`, so the first chunk must establish the initial mdBook-first scaffold without drifting into a clean-room generator.
- Mapping each `bookshelf.toml` entry to a valid mdBook `Config`/book root boundary may expose path assumptions that need careful tests.
- Pulling in published mdBook crates may require network access or version adjustments before local verification works.
- The archived prototype may contain useful domain logic, but its architecture cannot be adopted wholesale if it depends on HTML post-processing or staging copies.

## Active Chunk
```yaml
chunk_id: CHUNK-001
title: Bootstrap mdBook seam with dependency wiring and single-book load smoke test
objective: Add the minimal crate scaffold and Cargo dependencies needed to prove the confirmed startup seam can load one authored book in-process without invoking stock HTML build/render.
why_now: The repo currently has no `src/` and no mdBook crate deps, so no implementation can start until the mdBook-first loading seam is executable in this crate.
depends_on: []
mdbook_touchpoints:
  - reuse: `mdbook_summary::parse_summary` + `mdbook_driver::MDBook::load_with_config_and_summary` for single-book load
  - avoid: `MDBook::build()` and `mdbook_html::HtmlHandlebars` rendering path
scope_in:
  - add mdBook crates as Cargo dependencies in `Cargo.toml`
  - create minimal `src/lib.rs` and `src/main.rs` that compile
  - add one focused test that parses `SUMMARY.md` and loads one book via the seam
scope_out:
  - no multi-book composition
  - no bookshelf page generation
  - no serve/watch workflow
  - no HTML output generation
target_files:
  - Cargo.toml
  - src/lib.rs
  - src/main.rs
  - tests/seam_single_book_load.rs
implementation_tasks:
  - declare published mdBook crate dependencies needed by the seam (`mdbook-driver`, `mdbook-summary`, and required error/path utilities)
  - implement a tiny library function that accepts book root/src paths and returns a loaded `MDBook` via parsed summary + `load_with_config_and_summary`
  - add a smoke test fixture usage against existing example docs in-repo to validate seam behavior end-to-end
  - ensure binary entrypoint exists but does not perform rendering logic yet
acceptance_criteria:
  - `cargo check` succeeds with new mdBook dependencies
  - a dedicated test proves `parse_summary` + `load_with_config_and_summary` loads one book successfully from disk
  - test assertions confirm loaded book contains at least one chapter item from canonical `SUMMARY.md`
  - no code path in this chunk calls `MDBook::build()` or stock HTML renderer APIs
verification:
  - command: `cargo check`
    expect: exits 0
  - command: `cargo test seam_single_book_load -- --exact`
    expect: exits 0 and runs the seam smoke test
review_focus:
  - dependency choices use published mdBook crates, not local `mdBook-repo` internals
  - seam usage is explicit and minimal, with no hidden render/build invocation
  - test is deterministic, small, and anchored to canonical `SUMMARY.md` semantics
```

## Chunk Ledger
- none yet

## Final Validation
- Pending

## Activity Log
- 2026-04-20T13:12:26Z [coordinator] [INIT] [STARTED] Created progress.md and recorded startup constraints, tentative integration strategy, and initial risks.
- 2026-04-20T13:14:46Z [researcher] [STARTUP] [DONE] Confirmed minimal mdBook seam: load single authored book via `MDBook::load_with_config_and_summary` from driver-owned config/summary and avoid stock HTML by not invoking `MDBook::build()`.
- 2026-04-20T13:16:34Z [planner] [STARTUP] [DONE] Selected next chunk: add mdBook Cargo dependencies and a seam-proof single-book load smoke test around `parse_summary` + `load_with_config_and_summary`.
- 2026-04-20T13:18:33Z [developer] [CHUNK-001] [STARTED] Began dependency wiring and seam smoke-test implementation for single-book `parse_summary` + `load_with_config_and_summary`.
