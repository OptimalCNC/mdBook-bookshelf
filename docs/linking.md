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
- links that start with `/` are resolved from that directory

So `/docs/index.md` means "the file `docs/index.md` under the
`bookshelf.toml` directory".

## How To Write Links

- Use `/...` for site-root-relative links, especially across books.
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

During build, internal markdown page links are rewritten to the correct
published `.html` paths automatically.

## What Not To Write

Do not author links like these:

- `/modules/parser/docs/index.html`
- `/modules/parser/index.html`
- `../../outside-the-site.md`
- `/docs/bookshelf.html` when you really mean the target book's `index.md`

The authoring contract is always based on source paths under the
`bookshelf.toml` root.
