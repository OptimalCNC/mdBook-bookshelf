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
- Current chunk: C04
- Next action: Implement book-scoped sidebar tree generation and root-book Bookshelf affix behavior.
- Blockers:
  - Claude CLI review is currently unavailable in this environment because the CLI exits with `API Error: Unable to connect to API (ConnectionRefused)`.

## Active Chunk
```yaml
chunk_id: C04
title: Build book-scoped sidebar trees
objective: Derive renderer-ready sidebar trees from the site model, including the root-book Bookshelf affix and strict active-book scoping.
why_now: Sidebar behavior is the next explicit reader-shell requirement in the handoff and blocks later page rendering, while still fitting into a single focused model chunk.
depends_on:
  - C03
scope_in:
  - Add sidebar model types for affix entries, authored chapter entries, and per-book sidebar trees.
  - Generate one sidebar tree per book from the existing site model, preserving canonical summary order and nesting depth.
  - Inject a synthetic Bookshelf affix entry into the root-book sidebar only, targeting the synthetic Bookshelf page from C03.
  - Keep non-root sidebars limited to their own authored chapters with no cross-book leakage.
  - Expose a public API that returns sidebar data by book id for later renderer use.
  - Add tests for the self-contained example and root-affix/sidebar-scoping invariants.
scope_out:
  - Breadcrumb computation.
  - Previous and next computation.
  - Header Bookshelf return control.
  - HTML rendering and output writing.
  - Search index and search result labels.
  - Serve and watch workflows.
target_files:
  - src/sidebar.rs
  - src/lib.rs
  - tests/sidebar_model_example.rs
  - tests/sidebar_model_invariants.rs
implementation_tasks:
  - Define sidebar structs for root affix entries, authored chapter nodes, and per-book sidebar trees.
  - Build sidebar trees from C03 site-model data without mutating authored_page_order or authored_pages.
  - Represent Bookshelf as a dedicated affix entry that references the synthetic Bookshelf page id from C03.
  - Preserve declared summary nesting and chapter order for authored nodes.
  - Add verification coverage for root-book affix behavior and non-root book isolation.
acceptance_criteria:
  - A new public API builds sidebar data from the self-contained example site model and returns exactly three sidebars: meta, parser, and ui.
  - The `meta` sidebar contains exactly one affix entry titled `Bookshelf` targeting the synthetic Bookshelf page from C03, plus authored chapter entries `Example Core`, `Onboarding`, and `Architecture` in canonical summary order.
  - The `parser` sidebar contains only `Example Parser`, `Grammar`, and `Runtime`; the `ui` sidebar contains only `Example UI`, `Navigation`, and `Diagnostics`; neither non-root sidebar includes a `Bookshelf` affix or any chapter from another book.
  - Sidebar chapter entries preserve the summary nesting depth from C02, so `Grammar` and `Runtime` remain children of `Example Parser` and the root-book chapters remain under `Example Core`.
  - The Bookshelf affix remains synthetic only: it is not added to authored_page_order, does not become an authored page, and stays distinct from authored chapter entries so root-book chapter ordering remains unchanged.
  - The chunk introduces no HTML post-processing step and no temp-workspace-copy step.
verification:
  - command: "cargo test --test sidebar_model_example"
    expect: "The self-contained example produces the expected meta, parser, and ui sidebar trees with correct titles, nesting, and no cross-book leakage."
  - command: "cargo test --test sidebar_model_invariants"
    expect: "Root-book Bookshelf affix behavior and non-root sidebar scoping invariants pass without altering authored page order."
review_focus:
  - Sidebar generation must consume only the existing site model and must not invent extra authored pages or mutate canonical order.
  - The root-book Bookshelf entry must be modeled as a dedicated affix, not as a numbered chapter node.
  - Non-root sidebars must remain isolated to their owning books even when multiple books have similar structures.
```

## Chunk Ledger
- C01 `Bootstrap validated bookshelf input loading`:
  commits `621f5f2`, `d8088d2`; verification passed (`cargo test --test input_catalog_example`, `cargo test --test input_catalog_negative`, `cargo test`); reviewer-subagent approved after review fixes; Claude CLI review remains environment-blocked.
- C02 `Build the authored page catalog and per-book reading order`:
  commits `7199aef`, `d0a0442`; verification passed (`cargo test --test site_model_example`, `cargo test --test site_model_negative`, `cargo test`); reviewer-subagent approved after review fixes; Claude CLI review remains environment-blocked.
- C03 `Synthesize the in-memory Bookshelf page`:
  commits `a23bfcc`, `9828143`; verification passed (`cargo test --test bookshelf_page_example`, `cargo test --test bookshelf_page_negative`, `cargo test`); reviewer-subagent approved after review fixes; Claude CLI review remains environment-blocked.

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
