# mdBook Documentation Portal Example

This small project is a self-contained documentation portal fixture for
documenting and testing the current behavior of `mdbook-bookshelf`.

Layout:

- core book under `docs/`
- parser book under `modules/parser/docs/`
- UI book under `modules/ui/docs/`
- config in `bookshelf.toml` with site-level mdBook `[book]` metadata plus
  explicit `[[bookshelf.book]]` entries
- `index-title = "Example Books"` configures the generated main page heading
  and return button label
- category sections group `core`, `parser`, and `ui` for the documentation
  index
- books use config-root-relative mdBook-native `src` values
- core content uses `docs/index.md`, like the other module books use their own
  `docs/index.md`
- the documentation index source is generated under
  `.mdbook/bookshelf/documentation-index/` and rendered to site-root
  `index.html`, not authored in `docs/SUMMARY.md`

Prerequisite:

```bash
cargo install mdbook-variables
```
