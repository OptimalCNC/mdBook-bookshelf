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
- Current iteration: 4
- Current chunk: C05
- Next action: Implement per-page reader context for breadcrumbs, within-book prev/next, and Bookshelf return metadata.
- Blockers:
  - Claude CLI review is currently unavailable in this environment because the CLI exits with `API Error: Unable to connect to API (ConnectionRefused)`.

## Active Chunk
```yaml
chunk_id: C05
title: Build per-page reader context
objective: Derive renderer-ready reader context for each page, including active book ownership, breadcrumbs, within-book previous and next links, and Bookshelf return metadata.
why_now: The site model and sidebar trees now exist, but the renderer still lacks the per-page navigation context needed to render content pages correctly without inventing behavior outside the bookshelf-owned model.
depends_on:
  - C03
  - C04
scope_in:
  - Add page-level reader-context types for authored pages and the synthetic Bookshelf page.
  - Derive active book context for each authored page from the existing site model.
  - Compute breadcrumb data in `Book / Page` form from the existing site and sidebar models.
  - Compute previous and next links strictly within each book's authored reading order.
  - Expose Bookshelf return metadata for authored content pages, targeting the synthetic Bookshelf page from C03.
  - Add lookup APIs for resolving reader context by authored source path and by the synthetic Bookshelf page id.
scope_out:
  - HTML rendering and output writing.
  - Route/URL emission for rendered pages.
  - Search index generation and book-labeled search results.
  - Serve and watch workflows.
  - Asset copying and page templating.
target_files:
  - src/reader_context.rs
  - src/lib.rs
  - tests/reader_context_example.rs
  - tests/reader_context_invariants.rs
implementation_tasks:
  - Define reader-context structs for breadcrumbs, adjacent-page links, Bookshelf return metadata, and per-page active-book context.
  - Build reader contexts from the existing site model and sidebar model without mutating authored pages, sidebars, or the synthetic Bookshelf page.
  - Derive per-book previous and next links from authored_page_order so navigation never crosses book boundaries.
  - Provide lookup helpers for authored pages by normalized source path and for the synthetic Bookshelf page by page id.
  - Add example and invariant tests covering breadcrumbs, prev/next boundaries, and deep-link book activation.
acceptance_criteria:
  - A new public API builds reader context from the self-contained example and returns contexts for all nine authored pages plus the synthetic Bookshelf page.
  - Resolving `modules/parser/docs/grammar.md` in the self-contained example yields active book `parser`, breadcrumbs `Example Parser / Grammar`, previous page `modules/parser/docs/index.md`, next page `modules/parser/docs/runtime.md`, and a Bookshelf return target pointing to the synthetic Bookshelf page from C03.
  - Resolving `docs/architecture.md` yields active book `meta`, breadcrumbs `Example Core / Architecture`, previous page `docs/onboarding.md`, and no next page.
  - Resolving `modules/ui/docs/index.md` yields active book `ui`, breadcrumbs `Example UI / Example UI`, no previous page, and next page `modules/ui/docs/navigation.md`.
  - No authored page in the example has a previous or next link that crosses into another book.
  - Resolving the synthetic Bookshelf page by its page id yields owner book `meta`, breadcrumbs `Example Core / Bookshelf`, and no previous or next page.
verification:
  - command: "cargo test --test reader_context_example"
    expect: "The self-contained example produces the expected reader context for representative meta, parser, ui, and synthetic Bookshelf pages."
  - command: "cargo test --test reader_context_invariants"
    expect: "Prev/next boundaries stay book-local and deep-link lookups activate the correct book context without mutating sidebar or authored-page order."
review_focus:
  - Reader context must be derived entirely from the existing bookshelf-owned model, not from ad hoc rendering assumptions.
  - Previous and next links must be computed from per-book authored order only and must never cross book boundaries.
  - Breadcrumbs and Bookshelf return metadata must identify the active book unambiguously for deep-linked content pages.
```

## Chunk Ledger
- C01 `Bootstrap validated bookshelf input loading`:
  commits `621f5f2`, `d8088d2`; verification passed (`cargo test --test input_catalog_example`, `cargo test --test input_catalog_negative`, `cargo test`); reviewer-subagent approved after review fixes; Claude CLI review remains environment-blocked.
- C02 `Build the authored page catalog and per-book reading order`:
  commits `7199aef`, `d0a0442`; verification passed (`cargo test --test site_model_example`, `cargo test --test site_model_negative`, `cargo test`); reviewer-subagent approved after review fixes; Claude CLI review remains environment-blocked.
- C03 `Synthesize the in-memory Bookshelf page`:
  commits `a23bfcc`, `9828143`; verification passed (`cargo test --test bookshelf_page_example`, `cargo test --test bookshelf_page_negative`, `cargo test`); reviewer-subagent approved after review fixes; Claude CLI review remains environment-blocked.
- C04 `Build book-scoped sidebar trees`:
  commit `b3f8726`; verification passed (`cargo test --test sidebar_model_example`, `cargo test --test sidebar_model_invariants`, `cargo test`); reviewer-subagent approved; Claude CLI review remains environment-blocked.

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
- 2026-04-20T05:59:09Z [developer] [C02] [FINISHED] Rejected non-markdown SUMMARY targets explicitly, added regression coverage, and re-ran the required C02 tests.
- 2026-04-20T06:00:40Z [reviewer-subagent] [C02] [APPROVED] Site-model resolution now rejects non-markdown SUMMARY targets explicitly, and regression coverage plus direct repro verification pass.
- 2026-04-20T06:01:21Z [coordinator] [C02] [CLOSED] Moved C02 to the ledger after verification passed and the strict subagent reviewer approved the review-fix iteration.
- 2026-04-20T06:02:19Z [planner] [C03] [PLANNED] Chose in-memory Bookshelf page synthesis and root entrypoint modeling as the next chunk after authored page modeling.
- 2026-04-20T06:02:19Z [coordinator] [C03] [ACCEPTED] Accepted the planner artifact and activated C03 for implementation.
- 2026-04-20T06:04:52Z [developer] [C03] [STARTED] Synthesizing the Bookshelf page and root entrypoint metadata on top of the authored site model.
- 2026-04-20T06:06:57Z [developer] [C03] [FINISHED] Added the synthetic Bookshelf page, root entry metadata, and the required C03 verification coverage.
- 2026-04-20T06:09:20Z [reviewer-subagent] [C03] [CHANGES_REQUIRED] The site model adds shelf items but still lacks first-class synthetic Bookshelf page identity such as route/title metadata, so later rendering would have to invent page data outside the model.
- 2026-04-20T06:10:09Z [coordinator] [C03] [CHANGES_REQUIRED] Synthesized a same-chunk fix request: add first-class Bookshelf page route/title identity to the site model and verify it directly in tests.
- 2026-04-20T06:10:49Z [developer] [C03] [STARTED] Applying the C03 review fix to add first-class Bookshelf page identity metadata and direct verification coverage.
- 2026-04-20T06:11:39Z [developer] [C03] [FINISHED] Added direct Bookshelf page identity metadata, verified it in C03 tests, and re-ran the required test suite.
- 2026-04-20T06:12:59Z [reviewer-subagent] [C03] [APPROVED] The site model now carries first-class synthetic Bookshelf page identity and the root entry points to that page id directly, with direct verification coverage passing.
- 2026-04-20T06:13:45Z [coordinator] [C03] [CLOSED] Moved C03 to the ledger after verification passed and the strict subagent reviewer approved the review-fix iteration.
- 2026-04-20T06:15:20Z [planner] [C04] [PLANNED] Chose book-scoped sidebar tree generation with root-book Bookshelf affix behavior as the next chunk after synthetic page modeling.
- 2026-04-20T06:16:48Z [coordinator] [C04] [ACCEPTED] Accepted the planner artifact and activated C04 for implementation.
- 2026-04-20T06:18:33Z [developer] [C04] [STARTED] Building derived per-book sidebar trees with a root-book Bookshelf affix and strict book scoping.
- 2026-04-20T06:20:47Z [developer] [C04] [FINISHED] Added the derived sidebar model, verified root-affix and scoping invariants, and re-ran the required C04 tests.
- 2026-04-20T06:24:20Z [reviewer-subagent] [C04] [APPROVED] Sidebar generation stays book-scoped, preserves canonical nesting/order, and models the root Bookshelf entry as a synthetic affix without altering authored pages.
- 2026-04-20T06:24:55Z [coordinator] [C04] [CLOSED] Moved C04 to the ledger after verification passed and the strict subagent reviewer approved the initial iteration.
- 2026-04-20T06:26:27Z [planner] [C05] [PLANNED] Chose per-page reader context derivation for breadcrumbs, within-book prev/next, and Bookshelf return metadata as the next chunk after sidebar modeling.
- 2026-04-20T06:26:27Z [coordinator] [C05] [ACCEPTED] Accepted the planner artifact and activated C05 for implementation.
- 2026-04-20T06:28:24Z [developer] [C05] [STARTED] Deriving per-page reader context for breadcrumbs, within-book navigation, and Bookshelf return metadata.
