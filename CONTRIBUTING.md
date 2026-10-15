# Contributing to sb

## Getting started

```bash
git clone https://github.com/qjsoong/sb
cd sb
cargo build
cargo test --locked
```

`sb` depends on [mdk](https://github.com/qjsoong/mdk), which is public and
pinned to a released tag, so `cargo build` and `cargo test` work without extra
credentials.

CI runs the checks on Linux, macOS and Windows, plus a separate job for the
declared minimum Rust version (1.89).

## Before opening a pull request

Run the checks CI runs:

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
```

Markdown in this repository is formatted with `mdk` itself:

```bash
mdk fmt --check README.md CHANGELOG.md CONTRIBUTING.md
```

## Repository layout

| Path         | Purpose                                   |
| ------------ | ----------------------------------------- |
| `src/`       | The `sb` binary                           |
| `templates/` | The built-in page templates               |
| `example/`   | A small site used by the end-to-end tests |
| `tests/`     | End-to-end tests running the `sb` binary  |

Markdown and templates in `example/` are inputs, not documentation; keep them
minimal and deterministic, because the end-to-end tests build that site.

## Commit conventions

Use [Conventional Commits](https://www.conventionalcommits.org/): `feat`, `fix`,
`docs`, `refactor`, `test`, `build`, `ci`, `perf` or `chore`, optionally with a
scope such as `fix(build):`.
