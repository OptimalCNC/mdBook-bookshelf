# mdBook Shelf Example

This small project is a self-contained source fixture for implementing and
testing the bookshelf feature.

Layout:

- root book under `docs/`
- parser book under `modules/parser/docs/`
- UI book under `modules/ui/docs/`
- config in `bookshelf.toml` with standard mdBook tables plus `[bookshelf]`
- root-book content uses `docs/index.md`, like the other module books use their own `docs/index.md`
- the `Bookshelf` page is generated in memory, not authored in `docs/SUMMARY.md`
