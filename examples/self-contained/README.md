# mdBook Shelf Example

This small project is a self-contained bookshelf fixture for documenting and
testing the current behavior of `mdbook-bookshelf`.

Layout:

- root book under `docs/`
- parser book under `modules/parser/docs/`
- UI book under `modules/ui/docs/`
- config in `bookshelf.toml` with root-book mdBook `[book]` metadata plus `[bookshelf]`
- child books use config-root-relative mdBook-native `src` values
- root-book content uses `docs/index.md`, like the other module books use their own `docs/index.md`
- the `Bookshelf` page is generated in memory, not authored in `docs/SUMMARY.md`

Prerequisite:

```bash
cargo install mdbook-variables
```
