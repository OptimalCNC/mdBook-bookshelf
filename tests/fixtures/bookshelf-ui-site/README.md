# mdBook Shelf Bookshelf UI Fixture

This small project is a purpose-built fixture for testing the current behavior
of `mdbook-bookshelf`.

Layout:

- core book under `docs/`
- parser book under `modules/parser/docs/`
- UI book under `modules/ui/docs/`
- config in `bookshelf.toml` with site-level mdBook `[book]` metadata plus explicit `[[bookshelf.book]]` entries
- books use config-root-relative mdBook-native `src` values
- core content uses `docs/index.md`, like the other module books use their own `docs/index.md`
- the documentation index source is generated under `.mdbook/bookshelf/documentation-index/`, not authored in `docs/SUMMARY.md`
