# Test Plan

This note lists the tests that should be added beyond mdBook's stock tests.

The bookshelf feature should have its own tests for multi-book behavior while
still being implemented on top of mdBook's build and serve model.

## Unit Tests

Add unit tests for:

- parsing `bookshelf.toml`
- validating required `root_book` and `[[bookshelf.book]]`
- loading one canonical `SUMMARY.md` per book
- mapping authored markdown paths to canonical published `.html` paths
- building the in-memory site model
- mapping pages to owning books
- computing per-book reading order
- building breadcrumb data
- building sidebar trees
- search result labeling by owning book

## Integration Tests

Add integration tests that render the small self-contained example and assert:

- the site root opens the `Bookshelf` page
- the `Bookshelf` page lists all books with descriptions
- the root-book shelf item points to the root book's content root, not back to the shelf
- each content-book page shows the `Bookshelf` button
- the sidebar shows only the active book
- the root-book sidebar shows `Bookshelf` as an unnumbered affix item
- the `Bookshelf` affix does not affect chapter numbering
- previous and next never cross book boundaries
- breadcrumbs render as `Book / Page`
- authored site-root-relative links such as `/modules/parser/docs/index.md`
  resolve to `/modules/parser/docs/index.html`
- authored file-relative links such as `./sibling.md` resolve the same way mdBook
  already expects within one source tree
- direct links activate the correct book context
- search results include the owning book label
- the rendered site exposes a user-facing search flow that shows those labeled results

## Repo-Level Integration Tests

Add integration tests that cover the repo-scale scenario and assert the same behaviors at larger scale.

At minimum cover:

- root shelf page
- one root-book content page
- one parser content page
- one HMI content page

## Golden / Snapshot Tests

Add golden or snapshot tests for stable outputs where useful:

- generated site model serialization
- bookshelf page output
- root-book sidebar HTML
- non-root book sidebar HTML
- breadcrumb HTML
- selected rendered pages for the example site

Use snapshots sparingly and only where they protect intentional reader-facing structure.

## Negative Tests

Add failure tests for:

- missing `[bookshelf]`
- missing `root_book`
- duplicate book ids
- missing or empty `summary`
- authored `Bookshelf` page inside a canonical `SUMMARY.md`
- invalid page ownership or unresolved root page
- links that resolve outside the site root

## Watch / Serve Tests

If the final implementation supports live rebuilds, add tests or scripted checks for:

- markdown source change updates final HTML
- summary change updates final navigation
- config change updates bookshelf structure
- asset change updates final site

## Validation Command Expectations

The bookshelf implementation should ship with feature-specific validation
commands that cover bookshelf invariants, not just mdBook's stock build
success.

It should also exercise the mdBook-first top-level workflows that the feature
adds or extends, especially:

- one top-level build workflow for the multi-book site
- one top-level serve workflow for the multi-book site
