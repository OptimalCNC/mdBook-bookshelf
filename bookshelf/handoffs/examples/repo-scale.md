# Repo-Scale Scenario

This note describes the larger target scenario that the final bookshelf feature should support.

It is intentionally local to the handoff package so implementation teams do not need to read files outside this folder to understand the target.

## Catalog

The repo-scale bookshelf should support at least these books:

- `MetaNC`
- `G-code Parser`
- `HMI`

## Reader Scenarios

The final implementation should handle these flows:

- open the site root and land on the `Bookshelf` page
- confirm the `Bookshelf` page belongs to the `MetaNC` root book
- enter `MetaNC`, `G-code Parser`, and `HMI` from the shelf
- use the `Bookshelf` button from each content book to return to the shelf
- deep link into a parser page and confirm parser context
- deep link into an HMI page and confirm HMI context
- confirm the root-book sidebar shows `Bookshelf` as an unnumbered affix entry
- confirm `Bookshelf` does not affect chapter numbering of root-book content
- confirm search stays site-wide and results identify the owning book

## Why This Scenario Matters

The small self-contained example verifies correctness on a compact tree.

This repo-scale scenario verifies that the same model still works when:

- one book is repository-wide
- multiple peer books have deeper chapter trees
- cross-book navigation and search context matter at larger scale
