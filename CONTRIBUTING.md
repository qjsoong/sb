# Contributing to sb

## Getting started

```bash
git clone https://github.com/qjsoong/sb
cd sb
cargo build
cargo test --locked
```

`sb` depends on the private `mdk` repository, so cargo needs access to
`https://github.com/qjsoong/mdk` (the `gh` credential helper, for example).

CI fetches `mdk` with a read-only deploy key stored in the `MDK_DEPLOY_KEY`
secret. Pull requests from forks do not receive that secret; those runs check
formatting and then fail with a clear message, rather than reporting a green run
for code that was never compiled. CI runs on Linux only.

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
