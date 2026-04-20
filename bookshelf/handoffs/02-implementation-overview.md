# Implementation Overview

This note records the recommended implementation direction.

## Recommendation

Implement the bookshelf feature as an mdBook-first, renderer-oriented system.

That means:

- stay close to mdBook's parsing, rendering, and serving path
- build the bookshelf behavior as an extension/integration on top of mdBook
- avoid HTML post-processing over stock mdBook output as the primary
  architecture
- avoid treating mdBook as a disposable preprocessing stage under a separate
  standalone site generator

Concretely:

- Keep mdBook as the underlying documentation tooling model.
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
- Render the final HTML from that model within an mdBook-centric workflow,
  instead of rewriting mdBook's generated HTML afterward.

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

Trying to retrofit them after stock mdBook HTML generation is fragile and
difficult to extend.

At the same time, rebuilding mdBook from scratch is the wrong target for this
handoff. The implementation should reuse mdBook concepts, libraries, and
workflows where practical, while solving the multi-book behavior that stock
mdBook does not solve.

## Recommended Shape

The implementation should have these phases:

1. Load `bookshelf.toml`.
2. Load each canonical `SUMMARY.md` through an mdBook-compatible processing path.
3. Build one in-memory site model that represents the final bookshelf site.
4. Render final HTML, navigation, and search outputs from that model inside one
   mdBook-like build and serve workflow.

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
- keep the implementation mdBook-first instead of building a clean-room
  replacement for mdBook
- render final site output from the bookshelf-owned model through an
  mdBook-centric integration path

Acceptable implementation styles:

- a custom backend / renderer integrated with mdBook
- a custom driver around mdBook libraries plus a bookshelf-specific renderer
- targeted mdBook extension work that preserves mdBook's overall build/serve
  model

Avoid:

- HTML post-processing as the long-term architecture
- a standalone multi-book generator that replaces mdBook's core role
