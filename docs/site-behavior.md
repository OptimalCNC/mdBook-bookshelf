# Site Behavior

This document describes the current reader-facing behavior of a built
`mdbook-bookshelf` site.

## Landing And Routes

- the directory containing `bookshelf.toml` is the site root
- `/` and `/index.html` redirect to `<root book src>/index.html` by default
- every authored markdown page publishes at the same site-root-relative path
  with `.md` changed to `.html`
- setting `[bookshelf].entry-page = "bookshelf"` redirects `/` and
  `/index.html` to `<root book src>/bookshelf.html` instead

Examples:

- `docs/index.md` -> `/docs/index.html`
- `docs/architecture.md` -> `/docs/architecture.html`
- `modules/gcode-parser/docs/index.md` ->
  `/modules/gcode-parser/docs/index.html`
- `modules/gcode-parser/docs/reference/modal-groups.md` ->
  `/modules/gcode-parser/docs/reference/modal-groups.html`

## The Bookshelf Page

The `Bookshelf` page is synthetic. It is generated into the root book at
`<root book src>/bookshelf.html`.

Current behavior:

- the page title is `Bookshelf`
- it lists the root book and every child book
- each shelf item links to that book's `index.html`
- descriptions come from `book.description` and
  `[[bookshelf.book]].description` when present
- the root-book shelf item links to the root book's `index.html`, not back to
  `bookshelf.html`

The synthetic page also affects the root book sidebar:

- the root book keeps its normal chapter numbering
- `Bookshelf` appears as the first root-book sidebar entry
- the `Bookshelf` sidebar entry is unnumbered, and authored root chapters keep
  their normal numbering and order after it
- child-book sidebars do not include `Bookshelf`

## Reading Inside A Book

When a reader opens a content page:

- the sidebar shows only that book's table of contents
- previous and next links stay inside that book
- the page header gets a `Bookshelf` return button
- a breadcrumb is injected above the page body in `Book / Page` form
- direct links into nested pages still activate the correct book context

Examples:

- `MetaNC / Architecture`
- `G-code Parser / Modal Groups`

The `Bookshelf` return control is not shown on the `Bookshelf` page itself.

## Search

The site keeps mdBook's stock search UI, but the loaded search data is widened
to the whole bookshelf site.

Current search behavior:

- each built book still emits its own stock mdBook search assets
- `mdbook-bookshelf` composes a site-wide shared index at `/searchindex.js`
- each book also gets a localized `bookshelf-searchindex.js`
- page runtime switches mdBook's search loader to the localized shared index
- results can open pages in other books
- result labels use breadcrumb-style text, for example
  `G-code Parser » Grammar » Modal Groups » Modal Groups`

In other words, the chrome still feels like mdBook, but search scope is the
entire bookshelf site.

Search caveats:

- each built book must contain exactly one stock mdBook `searchindex-*.js`
  file, because that emitted payload is the merge input
- those stock payloads must stay compatible with each other so
  `mdbook-bookshelf` can merge and localize the shared search data
- every book receives its own localized `bookshelf-searchindex.js`
- on a cold load with `?search=`, mdBook may first request the page-local
  per-book `searchindex-*.js`; the bookshelf runtime then switches future
  search loading to the localized shared index

## Constraints Readers Will Notice

- the root entry page is configurable, defaulting to the root book's
  `index.html`
- book switching happens through the shelf or through direct page links, not
  through one merged global sidebar
- the built site keeps source-derived URLs, so `docs/` and other authored path
  segments stay visible in the published routes
