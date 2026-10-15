# sb — a simple blog

A minimal static blog generator built on [mdk](https://github.com/qjsoong/mdk).
Write Markdown posts; `sb` renders them into a directory of small, plain HTML
pages.

```console
$ sb build
sb: built 2 posts into public/
```

There are no themes, no plugins, no feeds and no client-side JavaScript. A site
is one configuration file, one posts directory and one optional HTML template,
and the output is plain HTML you can serve with anything.

## Install

`sb` has no binary releases yet, so install it from source:

```bash
cargo install --git https://github.com/qjsoong/sb --locked
```

`sb` builds on [mdk](https://github.com/qjsoong/mdk) and pins it to a revision.
`mdk` is currently private, so this command only works for accounts with access
to that repository.

## Quick start

```console
$ mkdir my-blog && cd my-blog
$ printf 'title = "My blog"\n' > sb.toml
$ mkdir posts
$ cat > posts/hello.md <<'EOF'
---
title: Hello, world
date: 2026-10-14
---

This is my first post.
EOF
$ sb build
sb: built 1 post into public/
$ python3 -m http.server -d public
```

## Site layout

| Path         | Purpose                                             |
| ------------ | --------------------------------------------------- |
| `sb.toml`    | Site configuration (required)                       |
| `posts/`     | Markdown posts; the directory tree becomes the URLs |
| `static/`    | Copied verbatim to the output root                  |
| `templates/` | Optional overrides for the two page templates       |
| `public/`    | Generated output, recreated on every build          |

Markdown files below `posts/` are picked up recursively; hidden files and
directories are skipped. `posts/notes/rust.md` becomes `/posts/notes/rust.html`.

Every post is validated before the first file is written, so a build that fails
leaves the previous site in place.

## Front matter

Every post starts with a front matter block. Both YAML (`---`) and TOML (`+++`)
delimiters are accepted.

| Key           | Required | Meaning                                               |
| ------------- | -------- | ----------------------------------------------------- |
| `title`       | yes      | Post title, used for the page heading and links       |
| `date`        | yes      | Publication date, `YYYY-MM-DD`; the index sorts by it |
| `description` | no       | Summary used in `<meta name="description">`           |
| `draft`       | no       | `true` keeps the post out of a normal build           |

Posts are sorted by date, newest first, then by file name. `sb build --drafts`
includes drafts, and points out how many were left out otherwise.

## Configuration

`sb.toml` lives in the site root.

| Key           | Required | Meaning                                               |
| ------------- | -------- | ----------------------------------------------------- |
| `title`       | yes      | Site title, shown in the header and used in `<title>` |
| `lang`        | no       | `lang` attribute for every page; defaults to `en`     |
| `description` | no       | Site description for the index page                   |

Unknown keys are rejected, so a typo is an error rather than a silent no-op.

## Templates

`sb` ships two templates, [templates/post.html](templates/post.html) and
[templates/index.html](templates/index.html). A file with the same name in the
site's `templates/` directory replaces the built-in one, so the files in this
repository are also the documentation for the built-in pages.

Pages are produced by replacing `{{placeholder}}` markers.

| Placeholder       | Post page                  | Index page       |
| ----------------- | -------------------------- | ---------------- |
| `{{lang}}`        | `lang` from `sb.toml`      | same             |
| `{{title}}`       | post title                 | site title       |
| `{{site_title}}`  | site title                 | site title       |
| `{{date}}`        | `YYYY-MM-DD`               | —                |
| `{{description}}` | post description           | site description |
| `{{content}}`     | rendered Markdown fragment | —                |
| `{{posts}}`       | —                          | list of posts    |

Values are HTML-escaped, and `{{content}}` and `{{posts}}` are inserted as HTML.
An unknown placeholder is an error, while a `{{...}}` that is not a name (for
example a CSS or JavaScript snippet) is left alone.

## What sb does not do

Tags, categories, feeds, sitemaps, pagination, related posts, syntax
highlighting, incremental builds, themes and plugins. A site that needs one of
them needs a different tool, or a small patch.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT
license](LICENSE-MIT) at your option.
