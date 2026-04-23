# Bookshelf Handoff

This folder is the handoff package for implementing the bookshelf feature on top
of the existing `mdBook` tooling model.

The goal is not to replace `mdBook` with a separate documentation engine. The
goal is to extend mdBook's stock single-book behavior so one mdBook-like build
and serve workflow can produce a multi-book site with:

- easy switching between books
- unified site-wide search
- book-scoped navigation instead of one merged sidebar

Start here, then read the files in order:

- [00-mdbook-first-scope.md](./00-mdbook-first-scope.md) — the mdBook-first scope and non-goals
- [01-target-site.md](./01-target-site.md) — what needs to be built and how the resulting site should appear
- [01a-routing-and-linking-contract.md](./01a-routing-and-linking-contract.md) — canonical URI placement and cross-book link authoring rules
- [02-implementation-overview.md](./02-implementation-overview.md) — recommended implementation direction
- [03-examples.md](./03-examples.md) — source examples and target scenarios to test against
- [04-test-plan.md](./04-test-plan.md) — tests to add beyond mdBook's stock test coverage
- [05-acceptance-criteria.md](./05-acceptance-criteria.md) — acceptance checklist for the finished feature

## Scope

This handoff is for implementing the bookshelf feature itself as an mdBook-first
extension/integration, not for extending the current HTML post-processing
prototype and not for building a standalone replacement for mdBook.

The intended destination is a maintainable implementation that adds a
bookshelf-owned multi-book model and reader behavior on top of mdBook's
generator and serve toolings.
