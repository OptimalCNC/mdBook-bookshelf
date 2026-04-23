Objective

Replace the custom bookshelf config model with one composed from mdBook's native
`Config` and `BookConfig` types so the top level behaves like a normal
`book.toml`, the top-level `book` section defines the root book, and
`[[bookshelf.book]]` defines the additional content books.

Global Constraints

- Project is under active development; do not preserve compatibility with the old
  bookshelf config model when a better mdBook-native model exists.
- Prefer mdBook's existing `Config` and `BookConfig` types over custom top-level
  config structs.
- Keep one canonical `SUMMARY.md` per book under that book's configured `src`.
- Stay within the existing mdBook-first architecture.

Integration Strategy

- Base the root-book config on mdBook's native top-level `Config`.
- Represent each additional content book with mdBook `BookConfig`-shaped data
  instead of a bespoke bookshelf-only book config.
- Keep bookshelf-only metadata limited to what mdBook does not model.
- Update fixtures, docs, and tests to reflect the new config shape.

Current State

- The top level now parses into native mdBook `Config`, stored in
  `BookshelfConfig.mdbook_config`.
- The root book now comes from top-level `[book]` plus `[bookshelf].root-id`.
- Non-root books now use `[[bookshelf.book]]` entries shaped as `id` + `root` +
  flattened mdBook `BookConfig`.
- The build and loader paths now reuse the stored native mdBook config instead
  of re-projecting the top-level config or defaulting it away.
- Shared config-root CSS/JS assets are staged per build under a non-hidden,
  hash-friendly source path, and relative shared `output.html.input-404` is
  explicitly rejected when child books use different roots.

Open Risks

- No known blocking risks.

Active Chunk

- Complete.

Chunk Ledger

- chunk_id: chunk-022-native-root-mdbook-config
  title: Promote the top-level mdBook config to the root book and reserve
    `[[bookshelf.book]]` for non-root books
  objective: Use mdBook's native top-level `Config` for the root book and shared
    build/output/preprocessor behavior, while keeping only bookshelf-specific
    identity and child-book root location as custom metadata.
  why_now: The current design parses the same file twice, duplicates root-book
    metadata, and still reimplements mdBook-owned config fields in custom
    structs.
  depends_on: []
  touchpoints:
    - `crates/mdbook-bookshelf/src/config.rs`
    - `crates/mdbook-bookshelf/src/catalog.rs`
    - `crates/mdbook-bookshelf/src/build.rs`
    - `crates/mdbook-bookshelf/src/lib.rs`
    - `README.md`
    - test files and fixture `bookshelf.toml` files
  scope_in:
    - top-level mdBook config parsed into native `Config`
    - root book synthesized from top-level `[book]`
    - `[bookshelf].root-id` as the root identity seam
    - child books modeled as `id` + `root` + flattened `BookConfig`
    - build path reuses stored native `Config` instead of re-projecting config
    - fixtures/docs/tests updated to remove duplicated root book entries
  scope_out:
    - per-child build/output/preprocessor overrides
    - navigation/output/search redesign
    - removing the need for stable book ids
  target_files:
    - `crates/mdbook-bookshelf/src/config.rs`
    - `crates/mdbook-bookshelf/src/catalog.rs`
    - `crates/mdbook-bookshelf/src/build.rs`
    - `README.md`
    - `tests/bookshelf_config_parse.rs`
    - `tests/input_catalog_build.rs`
    - `tests/multi_book_load.rs`
    - `tests/build_cli.rs`
    - `tests/serve_cli.rs`
    - `tests/fixtures/**/bookshelf.toml`
  implementation_tasks:
    - parse `[bookshelf]` separately from a top-level TOML table while
      deserializing the rest into native mdBook `Config`
    - define minimal bookshelf-only metadata for root identity and child roots
    - synthesize the root catalog/book entry from top-level `config.book`
    - remove duplicated root-book declarations from fixtures/docs
    - validate titles, ids, root-id collisions, and derived summary paths
  acceptance_criteria:
    - the top-level `book`, `build`, `output`, and `preprocessor` config is
      owned by mdBook `Config`, not a parallel bookshelf struct
    - the root book comes only from top-level `[book]`
    - child books use `root` plus mdBook-native `BookConfig` fields
    - the build path reuses stored native mdBook config directly
    - full tests pass after fixture/doc updates
  verification:
    - command: `cargo test`
      expect: full suite passes with the mdBook-native config model
  review_focus:
    - no duplicated root book inside `[[bookshelf.book]]`
    - child `src` remains mdBook-native relative to child `root`
    - no new custom top-level mdBook config class is invented

Final Validation

- `cargo test`

Activity Log

2026-04-23T07:50:00Z [coordinator] [init] [done] retargeted coordination to the mdBook-native config model refactor and adopted config.progress.md as the shared artifact
2026-04-23T08:18:23Z [planner] [chunk-022-native-root-mdbook-config] [planned] parse top level as native mdBook Config and move root-book metadata to top-level book section
2026-04-23T08:18:23Z [researcher] [mdbook-native-config-model] [done] use top-level Config plus tiny bookshelf metadata with child id and root instead of duplicating the root book
2026-04-23T08:40:00Z [developer] [chunk-022-native-root-mdbook-config] [done] refactored config parsing around native mdBook Config and BookConfig, updated build/catalog paths, and rewrote fixtures/docs
2026-04-23T08:44:00Z [direction-reviewer] [chunk-022-native-root-mdbook-config] [approved] native mdBook Config and BookConfig are now the owning config model with only minimal bookshelf metadata left
2026-04-23T08:47:00Z [coordinator] [chunk-022-native-root-mdbook-config] [rework] aligned loader with the stored native mdBook config path before final review
2026-04-23T08:52:00Z [implementation-reviewer] [chunk-022-native-root-mdbook-config] [changes_required] ids needed safe path-segment validation and shared output paths still had staging regressions
2026-04-23T08:58:00Z [coordinator] [chunk-022-native-root-mdbook-config] [rework] fixed id validation, staged shared assets under non-hidden hashed paths, and added input-404 regression coverage
2026-04-23T08:59:23Z [implementation-reviewer] [chunk-022-native-root-mdbook-config] [approved] prior id, staged-asset cleanup, and disabled input-404 regressions are resolved and verified
2026-04-23T08:59:23Z [coordinator] [chunk-022-native-root-mdbook-config] [done] final validation passed and both review tracks approved the completed chunk
