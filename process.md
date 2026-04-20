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
- Current iteration: 1
- Current chunk: C01
- Next action: Implement validated bookshelf input loading and its tests.
- Blockers:
  - None currently; implementation will be bootstrapped from scratch in this repository.

## Active Chunk
```yaml
chunk_id: C01
title: Bootstrap validated bookshelf input loading
objective: Create the first Rust implementation slice that loads bookshelf.toml, resolves each book's canonical SUMMARY.md, and produces a validated in-memory input catalog for later renderer work.
why_now: Rendering cannot start safely until the implementation can load the human-owned config and canonical summaries without inventing HTML post-processing or temp-workspace staging.
depends_on: []
scope_in:
  - Initialize a Rust crate for the bookshelf implementation with a library entry point.
  - Parse bookshelf.toml, including root_book and bookshelf.book entries.
  - Resolve each summary path relative to the config file and load the canonical SUMMARY.md for that book.
  - Derive each book's content root from the first summary link and store it in a validated input catalog.
  - Return structured validation errors for missing bookshelf config, missing root_book, duplicate book ids, missing or empty summary, configured root_book not present in the book list, and authored Bookshelf entries inside canonical summaries.
  - Add unit and integration tests for the self-contained example and targeted negative cases.
scope_out:
  - HTML rendering, templates, and theming.
  - Generated Bookshelf page rendering.
  - Sidebar, breadcrumbs, previous and next, search labeling, and cross-book routing.
  - Serve workflow, watch mode, and live rebuild behavior.
target_files:
  - Cargo.toml
  - src/lib.rs
  - src/config.rs
  - src/input_catalog.rs
  - tests/input_catalog_example.rs
  - tests/input_catalog_negative.rs
implementation_tasks:
  - Create the crate skeleton and add dependencies needed for TOML parsing and summary loading.
  - Define typed config and input-catalog structures for the bookshelf site and per-book records.
  - Implement a load_input_catalog API that accepts a bookshelf.toml path and resolves all file paths relative to that config file.
  - Parse each canonical SUMMARY.md, extract the root page link, and reject authored Bookshelf entries.
  - Add tests that cover the self-contained example fixture and the required negative validation cases.
acceptance_criteria:
  - A new library API can load bookshelf/handoffs/examples/self-contained/bookshelf.toml and returns root_book = meta with exactly three books: meta, parser, and ui.
  - The loaded catalog resolves content-root pages to docs/index.md, modules/parser/docs/index.md, and modules/ui/docs/index.md relative to the example config.
  - Loading fails with explicit errors for missing bookshelf config, missing root_book, duplicate book ids, missing or empty summary, configured root_book not present in the configured books, and any canonical SUMMARY.md that authors a Bookshelf entry.
  - All new tests pass without requiring any HTML post-processing step or temp-workspace copy step.
verification:
  - command: "cargo test --test input_catalog_example"
    expect: "The self-contained example fixture loads successfully and asserts the expected book ids and root-page paths."
  - command: "cargo test --test input_catalog_negative"
    expect: "Negative cases fail with targeted validation errors for the required config and summary invariants."
review_focus:
  - Path resolution must be anchored to the bookshelf.toml directory, not the process working directory.
  - Summary loading must preserve one canonical SUMMARY.md per book and must not synthesize a merged or global summary.
  - Validation errors should identify the offending book id or summary path so failures are actionable.
```

## Chunk Ledger
- none yet

## Final Validation
- Pending

## Activity Log
- 2026-04-20T05:10:33Z [coordinator] [INIT] [STARTED] Created process.md after inspecting coordinator instructions and handoff docs.
- 2026-04-20T05:10:33Z [coordinator] [INIT] [BLOCKED] Repository contains handoff/spec material only; no implementation codebase exists for bookshelf work.
- 2026-04-20T05:13:54Z [coordinator] [INIT] [UNBLOCKED] User authorized greenfield implementation in this repository; proceeding with chunk planning.
- 2026-04-20T05:16:18Z [planner] [C01] [PLANNED] Chose validated bookshelf input loading from bookshelf.toml and canonical SUMMARY.md as the first implementation chunk.
- 2026-04-20T05:17:33Z [coordinator] [C01] [ACCEPTED] Accepted the planner artifact and activated C01 for implementation.
- 2026-04-20T05:19:07Z [developer] [C01] [STARTED] Bootstrapping the Rust crate and validated input-catalog loader from bookshelf.toml plus canonical summaries.
