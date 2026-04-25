# Examples

This repository keeps two primary bookshelf examples.

## Self-Contained Example

See [`examples/self-contained/README.md`](../examples/self-contained/README.md)
for the fixture layout and authoring guidelines.

Config:

- `examples/self-contained/bookshelf.toml`

```bash
cargo run --bin book -- build examples/self-contained/bookshelf.toml --dest-dir .tmp/self-contained-site
```

Use it for:

- quick local build checks
- simple authoring examples
- verifying root redirect, shelf generation, scoped sidebars, and shared search

## Repo-Scale Fixture

Path:

- `examples/repo-scale.md`
- `tests/fixtures/build-cli/repo-scale/bookshelf.toml`

What it shows:

- a repository-wide root book
- multiple peer books with deeper chapter trees
- nested page routing such as `reference/modal-groups.md`
- direct-link activation and site-wide search behavior at larger scale

Use it for:

- full-system verification
- checking nested breadcrumb and `path_to_root` behavior
- checking cross-book search results and labels

## Other Fixtures

The `tests/fixtures/` tree contains smaller fixtures for config parsing,
catalog loading, navigation metadata, and other focused behaviors.

Those fixtures are useful when changing internals, but the self-contained and
repo-scale examples are the main human-readable references.
