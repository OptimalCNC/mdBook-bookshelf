# Examples

This note lists the concrete examples and scenarios the implementation should be tested against.

## Example 1: Small Self-Contained Example

Use the bookshelf example at:

- [examples/self-contained/README.md](./examples/self-contained/README.md)
- [examples/self-contained/bookshelf.toml](./examples/self-contained/bookshelf.toml)
- [examples/self-contained/docs/SUMMARY.md](./examples/self-contained/docs/SUMMARY.md)
- [examples/self-contained/modules/parser/docs/SUMMARY.md](./examples/self-contained/modules/parser/docs/SUMMARY.md)
- [examples/self-contained/modules/ui/docs/SUMMARY.md](./examples/self-contained/modules/ui/docs/SUMMARY.md)

This example exists specifically to validate bookshelf behavior in a small tree.

It also illustrates the intended config direction:

- root-book content lives at `docs/index.md`
- the config does not include dropped prototype staging fields such as workspace or site directories

### Expected Books

- `Example Core`
- `Example Parser`
- `Example UI`

### Expected Reader Scenarios

- open the site root and land on the `Bookshelf` page
- choose `Example Core` from the shelf and enter its root page
- use the `Bookshelf` button to return to the shelf
- deep link directly to a parser page and confirm the parser sidebar is active
- deep link directly to a UI page and confirm the UI sidebar is active
- confirm search is site-wide and results identify the owning book

## Example 2: Repo-Scale Scenario

Use the repo-scale scenario note at:

- [examples/repo-scale.md](./examples/repo-scale.md)

### Expected Books

- `MetaNC`
- `G-code Parser`
- `HMI`

### Expected Reader Scenarios

- open the repo bookshelf root
- confirm the `Bookshelf` page belongs to the `MetaNC` root book
- enter `MetaNC`, `G-code Parser`, and `HMI` from the shelf
- deep link into a parser acceptance-reference page and verify parser context
- deep link into an HMI page and verify HMI context
- confirm the root-book sidebar shows `Bookshelf` as an affix entry and not as a numbered chapter

## Example Coverage Expectations

The implementation should be considered incomplete unless it works for:

- the small self-contained example
- the repo-scale scenario

Do not treat success on only one of them as sufficient.
