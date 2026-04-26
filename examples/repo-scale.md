# Repo-Scale Fixture

This note describes the repo-scale bookshelf fixture used by the build and
full-system tests.

The fixture lives at:

- [../tests/fixtures/build-cli/repo-scale/bookshelf.toml](../tests/fixtures/build-cli/repo-scale/bookshelf.toml)

## Books

The fixture models these books:

- `MetaNC`
- `G-code Parser`
- `HMI`

## What It Verifies

The current test fixture verifies implemented behavior.

It checks:

- configured root `/` redirect to the synthetic `Bookshelf` page
- root-book `Bookshelf` page ownership and links
- root-book sidebar with a first unnumbered `Bookshelf` entry before authored
  chapters
- child-book sidebars that stay scoped to the active book
- direct links into nested parser pages such as `reference/modal-groups.md`
- site-wide search results that can open pages in other books

## Why It Matters

The self-contained example proves the model on a compact tree.

This repo-scale fixture proves the same model still holds when:

- one book is repository-wide
- peer books have deeper chapter trees
- nested routes need non-empty `path_to_root`
- cross-book search and deep linking matter at larger scale
