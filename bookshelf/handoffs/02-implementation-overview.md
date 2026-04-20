# Implementation Overview

This note records the recommended implementation direction.

## Recommendation

Implement the bookshelf feature as a first-class renderer-oriented system, not as HTML post-processing over stock mdBook output.

Concretely:

- Keep `bookshelf.toml` as the human-owned config.
- Keep one canonical `SUMMARY.md` per book.
- Build one explicit in-memory site model:
  - books
  - page ownership
  - per-book reading order
  - bookshelf page
  - sidebar trees
  - breadcrumbs
  - search labels
- Render the final HTML directly from that model, instead of rewriting mdBook's generated HTML afterward.

## Config Shape

The handoff examples use a logical config shape, not the dropped prototype shape.

Keep config fields that describe the bookshelf site itself, for example:

- `root_book`
- `[[bookshelf.book]]`

Avoid implementation-specific staging fields in the human-owned config, such as:

- temporary workspace directories
- temporary site output directories
- source-copy or source-staging controls

Those are implementation details and should not be part of the long-term authoring model by default.

## Why

The hard parts of the feature are reader-shell concerns:

- book-scoped sidebar behavior
- bookshelf entry behavior
- breadcrumbs
- per-book previous and next
- cross-book identity and search context

Those are rendering problems.

Trying to retrofit them after stock mdBook HTML generation is fragile and difficult to extend.

## Recommended Shape

The implementation should have these phases:

1. Load `bookshelf.toml`.
2. Load each canonical `SUMMARY.md`.
3. Build one in-memory site model that represents the final bookshelf site.
4. Render final HTML, assets, navigation, and search metadata from that model.

The final HTML should already be the intended multi-book reader shell.

## Recommended Boundaries

Keep these inputs human-owned:

- `bookshelf.toml`
- each book's `SUMMARY.md`
- each book's markdown pages
- shared static assets referenced by config

Keep these derived:

- unified navigation structures used for rendering
- bookshelf page content and route
- final HTML output
- search metadata derived from the site model

## Recommended Technical Direction

Preferred direction:

- implement the bookshelf feature in Rust, close to mdBook's rendering path
- render final site output from the bookshelf-owned model

Acceptable implementation styles:

- a custom backend / renderer
- a custom driver around mdBook libraries plus a bookshelf-specific renderer

Avoid using HTML post-processing as the long-term architecture.
