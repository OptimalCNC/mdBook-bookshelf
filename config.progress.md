Objective

Keep the mdBook-native top-level config model, but simplify child bookshelf
config further by removing explicit child `id` and `root`, deriving child mount
paths and ownership keys from config-root-relative child `src` instead.

Global Constraints

- Project is under active development; do not preserve compatibility with old
  child `id` / `root` config when a source-derived mount model is better.
- Prefer stock mdBook config types where possible.
- Be prudent about any config that is not part of stock mdBook.
- Keep one canonical `SUMMARY.md` per book under that book's configured source
  directory.
- Stay within the existing mdBook-first architecture.

Integration Strategy

- Keep the top level rooted in mdBook's native `Config`.
- Interpret child `[[bookshelf.book]].src` as a config-root-relative source
  directory.
- Derive each child's public mount path from its source location.
- Remove explicit child `id` and `root` from human config.
- Keep `[bookshelf].root-id` only as a temporary root-book routing seam because
  root-book public routing still collides with the synthetic site-root
  `index.html` redirect.
- Update fixtures, docs, and tests to the new child routing model.

Current State

- The top level already parses into native mdBook `Config`.
- The root book still uses `[bookshelf].root-id` as a temporary public-routing
  seam.
- Child books no longer use human-authored `id` or `root`.
- Child `src` is now config-root-relative, child mounts are derived from source
  location, and child output routes follow source-tree paths like
  `modules/parser/...`.
- Search now writes a canonical site-wide payload plus localized
  `bookshelf-searchindex.js` wrappers per book root so cross-book search still
  works with non-sibling output mounts.

Open Risks

- Full `root-id` removal is still blocked by the root-book `index.html`
  collision with the site-root redirect and synthetic bookshelf landing flow.

Active Chunk

- Complete.

Chunk Ledger

- chunk_id: chunk-023-derived-child-mount-from-src
  title: Remove child `root` and child `id` by deriving child mounts from `src`
  objective: Collapse child book location and public child routing onto one
    config-root-relative `src` field.
  why_now: Child `root` and `id` are currently redundant seams; `id` acts as a
    route alias, and `root` mainly reconstructs the real source directory.
  depends_on: []
  touchpoints:
    - `crates/mdbook-bookshelf/src/config.rs`
    - `crates/mdbook-bookshelf/src/catalog.rs`
    - `crates/mdbook-bookshelf/src/build.rs`
    - `crates/mdbook-bookshelf/src/root_bookshelf_preprocessor.rs`
    - `crates/mdbook-bookshelf/src/bookshelf_ui.rs`
    - `crates/mdbook-bookshelf/src/search.rs`
    - `README.md`
    - `tests/bookshelf_config_parse.rs`
    - `tests/input_catalog_build.rs`
    - `tests/build_cli.rs`
    - `tests/fixtures/**/bookshelf.toml`
  scope_in:
    - remove child `id` and child `root` from human config
    - interpret child `src` as config-root-relative
    - derive child mount path and internal ownership key from `src`
    - route child output/search/links from the derived mount path
    - update fixtures/docs/tests for child routes like `/modules/parser/...`
  scope_out:
    - removing `[bookshelf].root-id`
    - redesigning root-book site-root landing semantics
  target_files:
    - `crates/mdbook-bookshelf/src/config.rs`
    - `crates/mdbook-bookshelf/src/catalog.rs`
    - `crates/mdbook-bookshelf/src/build.rs`
    - `crates/mdbook-bookshelf/src/root_bookshelf_preprocessor.rs`
    - `crates/mdbook-bookshelf/src/bookshelf_ui.rs`
    - `crates/mdbook-bookshelf/src/search.rs`
    - `README.md`
    - `tests/bookshelf_config_parse.rs`
    - `tests/input_catalog_build.rs`
    - `tests/build_cli.rs`
    - `tests/fixtures/**/bookshelf.toml`
  implementation_tasks:
    - parse child books from flattened `BookConfig` only
    - derive child mount path from config-root-relative `src`
    - project mdBook-native `{book_root, book.src}` internally from that source
    - replace child `books/<id>` routing with source-derived child routes
    - update runtime JS and search URL rewriting for multi-segment child mounts
  acceptance_criteria:
    - child books no longer accept human-authored `id` or `root`
    - child location and routing are driven from `src`
    - child output paths follow source-tree mounts like `modules/parser`
    - root-book public routing remains unchanged for this chunk
    - full tests pass after fixture/doc updates
  verification:
    - command: `cargo test`
      expect: full suite passes with source-derived child mount paths
  review_focus:
    - child `root` is actually gone
    - child routing is derived from `src` rather than a parallel slug
    - `[bookshelf].root-id` is the only temporary remaining custom route seam

Final Validation

- `cargo test`

Activity Log

2026-04-23T09:05:00Z [coordinator] [init] [done] retargeted coordination to source-derived child mount paths while retaining root-id temporarily
2026-04-23T09:05:30Z [researcher] [mount-path-identity] [done] current code uses child id as a route alias and child root mainly to reconstruct source location
2026-04-23T09:06:00Z [planner] [chunk-023-derived-child-mount-from-src] [planned] remove child root/id in favor of config-root-relative child src and derived child mounts
2026-04-23T09:30:00Z [developer] [chunk-023-derived-child-mount-from-src] [done] removed child id/root from config, derived child mounts from src, updated search routing, and rewrote fixtures/docs/tests
2026-04-23T09:41:00Z [implementation-reviewer] [chunk-023-derived-child-mount-from-src] [approved] derived child mounts, routing, and localized search behavior are coherent
2026-04-23T09:45:30Z [direction-reviewer] [chunk-023-derived-child-mount-from-src] [changes_required] child source-derived keys still needed to reserve the temporary root-id namespace
2026-04-23T09:46:57Z [coordinator] [chunk-023-derived-child-mount-from-src] [rework] reserved the root-id ownership namespace for child derived mounts and added regression coverage
2026-04-23T09:46:57Z [direction-reviewer] [chunk-023-derived-child-mount-from-src] [approved] child mounts now reserve the temporary root-id namespace and the regression is covered
2026-04-23T09:46:57Z [coordinator] [chunk-023-derived-child-mount-from-src] [done] final validation passed and both review tracks approved the completed chunk
