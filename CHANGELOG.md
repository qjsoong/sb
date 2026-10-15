# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic
Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Changed

- Pin `mdk` to the released v0.3.0 tag instead of an unreleased revision.

## 0.1.0 - 2026-10-14

### Added

- `sb build`, rendering every Markdown file below `posts/` into a static site
  under `public/`: one page per post plus an index page.
- Front matter in YAML or TOML with `title`, `date`, `description` and `draft`,
  including draft filtering with `--drafts`.
- Verbatim copying of `static/` and optional page templates in `templates/` with
  `{{placeholder}}` substitution.
