# mdBook-First Scope

This handoff is for implementing a bookshelf feature on top of the existing
`mdBook` tooling model, not for building an unrelated documentation generator
that merely happens to consume markdown files.

## What Stays True

The foundation remains `mdBook`:

- markdown books are still authored as books
- `SUMMARY.md` remains the canonical navigation definition for each book
- the end-user build and serve workflows should stay mdBook-like
- the rendered site should stay stylistically and structurally consistent with
  mdBook where practical

The bookshelf feature exists to extend mdBook's stock single-book behavior with
multi-book behavior.

## Problem Statement

Stock `mdBook` is built around one book at a time:

- one canonical `SUMMARY.md`
- one sidebar tree
- one book-scoped reading order
- one search scope presented as if the site were a single book

The bookshelf feature is needed to layer the following on top of that model:

1. easy switching between multiple books
2. unified search across all books
3. scoped navigation so the sidebar for one book is not polluted by the others

## Implementation Consequences

The implementation should therefore be mdBook-first:

- reuse mdBook libraries, concepts, and build/serve flow where practical
- integrate close to mdBook's parsing, rendering, and serving path
- preserve mdBook-like shell structure and reader conventions where practical
- avoid treating mdBook as a disposable preprocessing step under a new
  standalone site generator architecture

At the same time, the bookshelf feature should not depend on fragile HTML
rewriting as its core design.

## Explicit Non-Goals

Do not interpret this handoff as asking for:

- a clean-room documentation site generator that replaces mdBook outright
- a bespoke multi-book web app that only borrows markdown inputs
- a primary architecture based on post-processing stock mdBook HTML
- a design that merges every book into one giant sidebar tree
