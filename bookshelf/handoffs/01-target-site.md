# Target Site

This note defines the reader-facing target for the bookshelf feature.

## Goal

Build one documentation site that lets readers:

- choose a book before entering a deep chapter tree
- read inside one book at a time with navigation scoped to that book
- return to the cross-book chooser in one click
- keep all books available through one site and one top-level build and serve workflow

## Terms

### Root Book

The site has one root book.

The root book owns the `Bookshelf` page.

The repository chooses which book is the root book.

### Bookshelf Page

The `Bookshelf` page is a page inside the root book.

It is not a separate book.

### Content Book

A content book is any book shown as a selectable peer on the `Bookshelf` page.

The root book may also appear there as one of the selectable books.

## Required Reader Behavior

### Bookshelf Page

The `Bookshelf` page must:

- list all content books
- link each book to that book's root page
- show a short description for each book
- avoid presenting `Bookshelf` itself as another content book

Special case for the root-book shelf item:

- it links to the root book's content root page
- it does not link back to the `Bookshelf` page itself

### Bookshelf Entry Button

Every content-book page must show a visible `Bookshelf` return control in the page header.

That control must:

- return the reader to the `Bookshelf` page
- remain visible without opening the sidebar
- stay in a stable top-right header position
- use icon plus label when space allows

### Book-Scoped Navigation

When the reader is inside a content book:

- the sidebar shows only that book
- the active book owns its own reading order and expansion state
- previous and next stay inside that book
- unrelated chapters from other books never appear in the same sidebar tree

### Cross-Book Switching

The site must behave predictably when moving between books:

- entering a book from the `Bookshelf` page opens that book at its root page
- direct links into another book activate that target book's navigation context
- breadcrumbs render as `Book / Page`

### Search

Search may remain site-wide, but search results must include enough context for cross-book reading.

Results should:

- include the owning book label
- open the target page with the correct book context active

## Required Authoring Model

The authoring model must be:

- one human-owned `bookshelf.toml`
- one canonical `SUMMARY.md` per book
- one stable root page per content book
- one stable mapping from each page to its owning book

For summary ownership:

- the root book keeps its own canonical reading order
- the `Bookshelf` page is generated in memory, not authored as a canonical summary entry
- non-root books keep their own `SUMMARY.md` beside their docs roots

## Non-Goals

Do not implement:

- one global merged sidebar tree
- a separate chooser book
- separate sites per content book
- a solution that depends on rewriting stock mdBook HTML as the primary architecture
