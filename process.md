# Bookshelf Implementation Process

## Objective
- Implement the bookshelf feature to satisfy the bookshelf handoff acceptance criteria and test plan.

## Global Constraints
- First-class renderer-oriented implementation.
- No primary HTML post-processing architecture.
- No primary temp-workspace-copy architecture.
- `bookshelf.toml` is the single human-owned config.
- One canonical `SUMMARY.md` per book.
- `Bookshelf` page is generated in memory.

## Permissions
- Planner: read repo, write only `process.md`.
- Developer: read/write repo, run build/test/format/validation, update `process.md`, create one commit per developer iteration, no destructive git ops.
- Reviewer-Subagent: read repo, run checks, write only `process.md`.
- Reviewer-Claude: read-only; coordinator mirrors its `ProcessNote`.
- Coordinator: orchestrates and may commit on behalf of the developer when needed.

## Current State
- Status: IN_PROGRESS
- Current iteration: 3
- Current chunk: C02
- Next action: Implement the C02 review fix that rejects non-markdown SUMMARY targets and adds a regression test.
- Blockers:
  - Claude CLI review is currently unavailable in this environment because the CLI exits with `API Error: Unable to connect to API (ConnectionRefused)`.

## Active Chunk
```yaml
chunk_id: C02
title: Build the authored page catalog and per-book reading order
objective: Transform the validated input catalog into a site-owned authored page model with stable page ownership, summary hierarchy metadata, and per-book reading order for all authored markdown pages.
why_now: The renderer-oriented architecture now needs a concrete page model; without resolved page ownership and reading order, sidebar, breadcrumbs, previous and next, and direct rendering remain speculative.
depends_on:
  - C01
scope_in:
  - Add a page-model API that consumes the C01 input catalog.
  - Parse every canonical SUMMARY.md link into authored page records for each book.
  - Preserve summary nesting metadata needed for later sidebar generation.
  - Resolve every authored page target to an existing markdown source file.
  - Record stable page ownership by book id and reject duplicate ownership of the same source page across books.
  - Expose each book's authored page order starting at that book's content root.
scope_out:
  - Synthesizing the in-memory Bookshelf page.
  - HTML rendering, templates, assets, and theming.
  - Breadcrumb rendering, sidebar HTML, header controls, and previous/next link output.
  - Search index output and search result labels.
  - Serve and watch workflows.
target_files:
  - src/site_model.rs
  - src/lib.rs
  - tests/site_model_example.rs
  - tests/site_model_negative.rs
implementation_tasks:
  - Define site-model structs for books and authored pages, including source path, owning book id, title, order position, and summary depth.
  - Implement a builder that walks each canonical SUMMARY.md and collects all authored page entries in declared order.
  - Validate that each referenced markdown page exists on disk and that no authored page path is claimed by more than one book.
  - Expose a public API that builds the authored site model from the C01 input catalog.
  - Add example and negative tests that exercise page ownership and per-book order.
acceptance_criteria:
  - A new public API can build a site model from `bookshelf/handoffs/examples/self-contained/bookshelf.toml` and returns authored page orders `meta:[docs/index.md,docs/onboarding.md,docs/architecture.md]`, `parser:[modules/parser/docs/index.md,modules/parser/docs/grammar.md,modules/parser/docs/runtime.md]`, and `ui:[modules/ui/docs/index.md,modules/ui/docs/navigation.md,modules/ui/docs/diagnostics.md]` relative to the example config directory.
  - The site model records exactly nine authored pages for the self-contained example, and every authored page is owned by exactly one book.
  - Each authored page record includes the resolved markdown source path and the summary depth needed to reconstruct later sidebar trees.
  - Building the site model fails with explicit errors when a summary references a missing markdown file or when two books claim the same authored page path.
  - The chunk introduces no HTML post-processing step and no temp-workspace-copy step.
verification:
  - command: "cargo test --test site_model_example"
    expect: "The self-contained example builds into an authored site model with the expected page ownership and per-book order."
  - command: "cargo test --test site_model_negative"
    expect: "Negative fixtures fail with targeted errors for missing page targets and duplicate page ownership."
review_focus:
  - Summary traversal must preserve authored order exactly as declared in each canonical SUMMARY.md.
  - Page ownership must be normalized by resolved path so duplicate claims are caught reliably.
  - The model must represent authored pages only; the generated Bookshelf page stays out of this chunk.
```

## Chunk Ledger
- C01 `Bootstrap validated bookshelf input loading`:
  commits `621f5f2`, `d8088d2`; verification passed (`cargo test --test input_catalog_example`, `cargo test --test input_catalog_negative`, `cargo test`); reviewer-subagent approved after review fixes; Claude CLI review remains environment-blocked.

## Final Validation
- Pending

## Activity Log
- 2026-04-20T05:10:33Z [coordinator] [INIT] [STARTED] Created process.md after inspecting coordinator instructions and handoff docs.
- 2026-04-20T05:10:33Z [coordinator] [INIT] [BLOCKED] Repository contains handoff/spec material only; no implementation codebase exists for bookshelf work.
- 2026-04-20T05:13:54Z [coordinator] [INIT] [UNBLOCKED] User authorized greenfield implementation in this repository; proceeding with chunk planning.
- 2026-04-20T05:16:18Z [planner] [C01] [PLANNED] Chose validated bookshelf input loading from bookshelf.toml and canonical SUMMARY.md as the first implementation chunk.
- 2026-04-20T05:17:33Z [coordinator] [C01] [ACCEPTED] Accepted the planner artifact and activated C01 for implementation.
- 2026-04-20T05:19:07Z [developer] [C01] [STARTED] Bootstrapping the Rust crate and validated input-catalog loader from bookshelf.toml plus canonical summaries.
- 2026-04-20T05:31:11Z [coordinator] [C01] [VERIFIED] Re-ran `cargo test --test input_catalog_example`, `cargo test --test input_catalog_negative`, and `cargo test` successfully in the coordinator workspace.
- 2026-04-20T05:37:44Z [coordinator] [C01] [COMMITTED] Created checkpoint commit `621f5f2` with message `bookshelf: C01 iteration 1` after the developer worker hit a sandbox git-lock restriction.
- 2026-04-20T05:40:56Z [reviewer-subagent] [C01] [CHANGES_REQUIRED] Hand-rolled config parsing rejects valid TOML strings containing `#`, so the human-owned `bookshelf.toml` is not reliably loadable.
- 2026-04-20T05:46:09Z [coordinator] [C01] [CHANGES_REQUIRED] Synthesized a same-chunk fix request: replace the ad hoc config parser with TOML-compliant parsing and add a `#`-in-string regression test.
- 2026-04-20T05:46:09Z [coordinator] [C01] [REVIEW_BLOCKED] Claude CLI reviewer returned `API Error: Unable to connect to API (ConnectionRefused)` and did not emit a verdict.
- 2026-04-20T05:46:54Z [developer] [C01] [STARTED] Applying the C01 review fix for quoted `#` TOML values and adding regression coverage.
- 2026-04-20T05:47:47Z [developer] [C01] [FINISHED] Fixed quoted `#` handling in config parsing, added regression coverage, and re-ran the required C01 tests.
- 2026-04-20T05:49:04Z [reviewer-subagent] [C01] [APPROVED] Quote-aware inline-comment stripping now preserves valid TOML `#` inside strings, and regression coverage plus direct repro verification pass.
- 2026-04-20T05:49:30Z [coordinator] [C01] [CLOSED] Moved C01 to the ledger after verification passed and the strict subagent reviewer approved the review-fix iteration.
- 2026-04-20T05:51:00Z [planner] [C02] [PLANNED] Chose authored page ownership and per-book reading-order modeling as the next chunk after validated input loading.
- 2026-04-20T05:51:00Z [coordinator] [C02] [ACCEPTED] Accepted the planner artifact and activated C02 for implementation.
- 2026-04-20T05:53:23Z [developer] [C02] [STARTED] Building the authored page site model with summary-order traversal and normalized page ownership checks.
- 2026-04-20T05:55:13Z [developer] [C02] [FINISHED] Added the authored page model, verified page ownership and reading order, and re-ran the required C02 tests.
- 2026-04-20T05:57:00Z [reviewer-subagent] [C02] [CHANGES_REQUIRED] Site-model page resolution accepts any existing file path from SUMMARY links, so C02 does not yet enforce authored markdown-page ownership.
- 2026-04-20T05:57:46Z [coordinator] [C02] [CHANGES_REQUIRED] Synthesized a same-chunk fix request: reject non-markdown SUMMARY targets explicitly and add regression coverage for existing non-markdown files.
- 2026-04-20T05:58:21Z [developer] [C02] [STARTED] Applying the C02 review fix to reject non-markdown SUMMARY targets and add regression coverage.
