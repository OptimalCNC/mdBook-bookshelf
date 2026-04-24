# mdbook-bookshelf

`mdbook-bookshelf` builds multi-book documentation sites on top of stock
mdBook.

Install the `book` binary with either of these commands:

```bash
# from a local checkout
cargo install --locked --path .

# directly from GitHub
cargo install --locked --git https://github.com/OptimalCNC/mdBook-bookshelf mdbook-bookshelf
```

Run it against the repo's own docs:

```bash
book build bookshelf.toml
book serve bookshelf.toml --port 3000
```

Project documentation lives in [docs/index.md](./docs/index.md).
