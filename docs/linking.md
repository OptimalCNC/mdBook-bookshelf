# Linking

This page explains how to write links in `mdbook-bookshelf`.

## Root Directory

The root directory is the directory that contains `bookshelf.toml`.

Example:

```text
my-repo/
  bookshelf.toml
  docs/
  modules/
```

Here, `my-repo/` is the root directory.

In `mdbook-bookshelf`:

- each book `src` is resolved from that directory
- documentation site-root link and image destinations start with a single `/`

So `/docs/index.md` means "the file `docs/index.md` under the
`bookshelf.toml` directory". A destination that starts with `//` is a
protocol-relative URL, not a documentation site-root destination.

## How To Write Links

- Use single-slash `/...` destinations for site-root-relative links, especially
  across books.
- Use `./...` and `../...` for nearby file-relative links inside one book.
- Link to markdown source files such as `index.md` and `grammar.md`.
- Link assets to their normal asset path.

Examples:

```md
See [Root Book](/docs/index.md).
See [Parser Book](/modules/parser/docs/index.md).
See [Grammar](./grammar.md).
See [Parent Page](../index.md).
![Logo](./images/logo.svg)
```

During build, `mdbook-bookshelf` rewrites single-slash site-root link and image
destinations relative to the current output page. Site-root markdown page links
are resolved through the loaded books and rewritten to their published `.html`
paths.

For example, when written in `docs/onboarding.md`:

```md
[Root](/)
[Core](/docs/index.md)
[Parser](/modules/parser/docs/index.md?mode=fast#grammar)
![Diagram](/assets/diagram.svg#icon)
```

the destinations become:

```md
[Root](../index.html)
[Core](index.html)
[Parser](../modules/parser/docs/index.html?mode=fast#grammar)
![Diagram](../assets/diagram.svg#icon)
```

Query strings and fragments are preserved after the destination is rewritten.
Non-markdown single-slash destinations, such as images or other assets, are
rewritten as paths under the site root.

These destinations are not documentation site-root destinations and are left
unchanged by this preprocessor:

```md
[Local](./grammar.md)
[Parent](../README.md)
[Fragment](#section)
[External](https://example.com/docs/index.md)
[Mail](mailto:team@example.com)
[Protocol](//cdn.example.com/app.js)
```

File-relative markdown links inside one book remain normal mdBook links.

## Missing Pages

A single-slash markdown page link must resolve to a loaded page. For example,
`/missing.md` fails the build if no loaded book contains that site-root source
path.

If a loaded page comes from `README.md`, it is also accepted as an `index.md`
site-root target. For example, a page loaded from `docs/README.md` can be
linked as either `/docs/README.md` or `/docs/index.md`, and both resolve to
`docs/index.html`. This is link-target compatibility; it does not require every
book to author an `index.md` source file.

## What Not To Write

Do not author links like these:

- `/modules/parser/docs/index.html`
- `/modules/parser/index.html`
- `../../outside-the-site.md`
- `/docs/index.html` when you can link to the source page as `/docs/index.md`

The authoring contract is always based on source paths under the
`bookshelf.toml` root.
