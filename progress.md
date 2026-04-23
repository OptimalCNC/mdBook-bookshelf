Objective

Replace custom per-book `summary = ".../SUMMARY.md"` configuration with mdBook-style
per-book `src = "..."`, deriving each canonical summary as `src/SUMMARY.md` and
reusing stock mdBook source-directory semantics wherever possible.

Global Constraints

- Project is under active development; do not preserve compatibility with the old
  `summary` option.
- Prefer stock mdBook behavior and configuration seams over custom options.
- Keep one canonical `SUMMARY.md` per book under that book's configured `src`.
- Stay within the existing mdBook-first architecture.

Integration Strategy

- Treat each `[[bookshelf.book]]` entry as specifying a book source directory via
  `src`.
- Derive `summary_abs` internally as `<src>/SUMMARY.md` instead of configuring the
  summary path directly.
- Reuse `config.book.src` and `MDBook::load_with_config_and_summary()` exactly as
  mdBook expects.
- Update fixtures, docs, and tests to reflect the new configuration surface.

Current State

- Repo now parses `bookshelf.book.src` as the only supported per-book content
  root option.
- Canonical summaries are derived internally as `<src>/SUMMARY.md`.
- The parser rejects removed `summary` keys and file-like `src = ".../SUMMARY.md"`
  values, while accepting mdBook-valid `src = "."` and `src = "./docs"` forms.
- Checked-in configs, fixtures, and README examples now use `src = "..."`.

Open Risks

- No known blocking risks.

Active Chunk

- Complete.

Chunk Ledger

- chunk_id: chunk-021-bookshelf-src-option
  title: Replace per-book summary config with per-book src
  objective: Use `src` in each `[[bookshelf.book]]` entry and derive the
    canonical summary as `<src>/SUMMARY.md`.
  why_now: The repo TODO explicitly calls for `src`, and upstream mdBook already
    uses `[book].src` as the stock source-directory seam.
  depends_on: []
  touchpoints:
    - `crates/mdbook-bookshelf/src/config.rs`
    - `crates/mdbook-bookshelf/src/catalog.rs`
    - `crates/mdbook-bookshelf/src/build.rs`
    - `crates/mdbook-bookshelf/src/loader.rs`
    - `README.md`
    - test files and bookshelf fixture TOML files
  scope_in:
    - replace `bookshelf.book.summary` with `bookshelf.book.src`
    - validate `src` as a relative source directory path
    - derive summary paths internally as `src/SUMMARY.md`
    - update tests, fixtures, and user-facing docs to the new option
  scope_out:
    - cross-book references
    - mdBook plugin support
    - changing canonical `SUMMARY.md` ownership rules
  target_files:
    - `crates/mdbook-bookshelf/src/config.rs`
    - `crates/mdbook-bookshelf/src/catalog.rs`
    - `tests/bookshelf_config_parse.rs`
    - `tests/input_catalog_build.rs`
    - `tests/fixtures/**/bookshelf.toml`
    - `README.md`
  implementation_tasks:
    - parse `src` instead of `summary`
    - normalize and validate per-book `src`
    - derive `summary_abs` from `src`
    - update fixture TOML and test expectations
    - update README examples and remove the completed TODO item
  acceptance_criteria:
    - `[[bookshelf.book]]` entries use `src = "..."` and no code path requires a
      configured summary path
    - mdBook loading/building still uses each book's canonical `SUMMARY.md`
      inside that `src`
    - targeted tests pass with updated fixtures and expectations
  verification:
    - command: `cargo test`
      expect: all tests pass with `src`-based config fixtures
  review_focus:
    - implementation reuses stock mdBook `src` semantics instead of inventing a
      new summary-location option
    - no compatibility layer for the removed `summary` config

Final Validation

- `cargo test`
- `rg -n 'summary\s*=\s*"' README.md bookshelf tests crates cli`
  - only matched the intentional negative test in `tests/bookshelf_config_parse.rs`

Activity Log

2026-04-23T07:29:19Z [coordinator] [init] [done] created progress artifact and recorded upstream mdBook src seam
2026-04-23T07:30:00Z [coordinator] [chunk-021-bookshelf-src-option] [planned] locked implementation scope around stock mdBook src semantics
2026-04-23T07:33:29Z [planner] [chunk-021-bookshelf-src-option] [planned] replace per-book summary config with stock mdBook-style src and derive SUMMARY.md internally
2026-04-23T07:36:24Z [researcher] [content-root-seam] [done] upstream mdBook exposes book.src as the native content-root seam and offers no better stock summary-path option
2026-04-23T07:41:43Z [implementation-reviewer] [chunk-021-bookshelf-src-option] [changes_required] stale summary keys remain silently accepted and file-like src values are not rejected early
2026-04-23T07:41:43Z [direction-reviewer] [chunk-021-bookshelf-src-option] [changes_required] parser still rejects mdBook-valid src forms like . and ./docs instead of fully reusing stock src semantics
2026-04-23T07:41:43Z [coordinator] [chunk-021-bookshelf-src-option] [rework] sent both review findings back to developer for one follow-up iteration
2026-04-23T07:36:48Z [developer] [chunk-021-bookshelf-src-option] [done] switched per-book config to src, derived canonical SUMMARY.md internally, and updated tests/docs/fixtures
2026-04-23T07:43:02Z [developer] [chunk-021-bookshelf-src-option] [done] rejected stale summary keys, rejected file-like src values, and accepted mdBook-valid . / ./docs src forms
2026-04-23T07:45:49Z [direction-reviewer] [chunk-021-bookshelf-src-option] [approved] parser now matches the intended mdBook-style src seam
2026-04-23T07:45:49Z [implementation-reviewer] [chunk-021-bookshelf-src-option] [approved] stale summary is rejected, file-like src is rejected early, and mdBook-valid dot forms work
2026-04-23T07:45:49Z [coordinator] [chunk-021-bookshelf-src-option] [done] final validation passed and both review tracks approved the completed chunk
