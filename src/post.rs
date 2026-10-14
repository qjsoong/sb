//! Posts: discovery, front matter and metadata.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use mdk_core::{Dialect, Source};
use mdk_render::{HtmlOptions, render_html};

/// The Markdown dialect posts are written in.
const DIALECT: Dialect = Dialect::Gfm;

/// A Markdown post and the metadata from its front matter.
#[derive(Debug)]
pub struct Post {
    relative: PathBuf,
    output: PathBuf,
    /// Root-absolute URL of the generated page.
    pub url: String,
    /// Title from the front matter.
    pub title: String,
    /// Publication date from the front matter, `YYYY-MM-DD`.
    pub date: String,
    /// Optional one-line summary from the front matter.
    pub description: String,
    /// Whether the post is marked `draft: true`.
    pub draft: bool,
    source: Source,
}

impl Post {
    /// Loads and validates the post at `path`, which must be below `root`.
    ///
    /// Failures are strings because the caller reports every invalid post at
    /// once; each message starts with the post's path.
    pub fn load(root: &Path, path: &Path) -> std::result::Result<Self, String> {
        let relative = path.strip_prefix(root).unwrap_or(path).to_path_buf();
        let text = fs::read_to_string(path).map_err(|error| {
            format!("{}: cannot read file: {error}", relative.display())
        })?;
        Self::parse(relative, text)
    }

    /// Parses a post from its path relative to the site root and its text.
    fn parse(relative: PathBuf, text: String) -> std::result::Result<Self, String> {
        let display = relative.display();
        let source = Source::new(text);

        if source.front_matter().is_none() {
            let reason = if unclosed_front_matter(source.text()) {
                "unclosed front matter; the opening `---` has no closing `---` line"
            } else {
                "missing front matter; a post starts with `---`, then `title` and `date`"
            };
            return Err(format!("{display}: {reason}"));
        }

        let title = match source.front_matter_value("title") {
            Some("") => return Err(format!("{display}: front matter title is empty")),
            Some(title) => title.to_string(),
            None => {
                return Err(format!(
                    "{display}: front matter has no `title`; add `title: My post`"
                ));
            }
        };

        let date = match source.front_matter_value("date") {
            Some(date) if !date.is_empty() => {
                if !is_date(date) {
                    return Err(format!(
                        "{display}: `date` must be an ISO 8601 date like \
                         `2026-10-14`, got `{date}`"
                    ));
                }
                date.to_string()
            }
            _ => {
                return Err(format!(
                    "{display}: front matter has no `date`; add `date: 2026-10-14`"
                ));
            }
        };

        let draft = match source.front_matter_value("draft") {
            None | Some("") | Some("false") => false,
            Some("true") => true,
            Some(other) => {
                return Err(format!(
                    "{display}: `draft` must be `true` or `false`, got `{other}`"
                ));
            }
        };

        let description = source
            .front_matter_value("description")
            .unwrap_or("")
            .to_string();
        let output = relative.with_extension("html");
        let url = url_of(&output);

        Ok(Self {
            relative,
            output,
            url,
            title,
            date,
            description,
            draft,
            source,
        })
    }

    /// The post body as an HTML fragment, without the front matter.
    pub fn body_html(&self) -> String {
        render_html(self.source.text(), DIALECT, &HtmlOptions::default())
    }

    /// Output path relative to the `public` directory.
    pub fn output(&self) -> &Path {
        &self.output
    }
}

/// Finds every Markdown file below `<root>/posts`, sorted by path.
pub fn discover(root: &Path) -> Result<Vec<PathBuf>> {
    let directory = root.join("posts");
    if !directory.is_dir() {
        bail!(
            "no posts directory at {}; create it and add a Markdown file",
            directory.display()
        );
    }
    let mut files = Vec::new();
    collect(&directory, &mut files)?;
    files.sort();
    Ok(files)
}

/// Orders posts newest first, then by path so builds are deterministic.
pub fn sort(posts: &mut [Post]) {
    posts.sort_by(|a, b| {
        b.date
            .cmp(&a.date)
            .then_with(|| a.relative.cmp(&b.relative))
    });
}

fn collect(directory: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    let entries = fs::read_dir(directory)
        .with_context(|| format!("cannot read {}", directory.display()))?;
    for entry in entries {
        let entry =
            entry.with_context(|| format!("cannot read {}", directory.display()))?;
        let name = entry.file_name();
        if name.to_string_lossy().starts_with('.') {
            continue;
        }
        let path = entry.path();
        let file_type = entry
            .file_type()
            .with_context(|| format!("cannot inspect {}", path.display()))?;
        if file_type.is_dir() {
            collect(&path, files)?;
        } else if file_type.is_file() && path.extension().is_some_and(|ext| ext == "md")
        {
            files.push(path);
        }
    }
    Ok(())
}

/// The root-absolute URL of an output path.
fn url_of(path: &Path) -> String {
    let mut url = String::from("/");
    for (index, component) in path.components().enumerate() {
        if index > 0 {
            url.push('/');
        }
        url.push_str(&component.as_os_str().to_string_lossy());
    }
    url
}

/// Whether the text opens a front matter block that never closes.
fn unclosed_front_matter(text: &str) -> bool {
    let mut lines = text.lines();
    let Some(first) = lines.next() else {
        return false;
    };
    let opener = first.trim_end();
    if opener != "---" && opener != "+++" {
        return false;
    }
    !lines.any(|line| line.trim_end() == opener)
}

/// Whether `value` is a valid `YYYY-MM-DD` date.
fn is_date(value: &str) -> bool {
    if !value.is_ascii() || value.len() != 10 {
        return false;
    }
    let bytes = value.as_bytes();
    if bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    if !value
        .bytes()
        .enumerate()
        .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
    {
        return false;
    }
    let (Ok(year), Ok(month), Ok(day)) = (
        value[0..4].parse::<u32>(),
        value[5..7].parse::<u32>(),
        value[8..10].parse::<u32>(),
    ) else {
        return false;
    };
    (1..=12).contains(&month) && (1..=days_in_month(year, month)).contains(&day)
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: u32) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(title: &str, date: &str) -> String {
        format!("---\ntitle: {title}\ndate: {date}\n---\n\nBody.\n")
    }

    fn parse(name: &str, contents: &str) -> std::result::Result<Post, String> {
        Post::parse(PathBuf::from(format!("posts/{name}")), contents.to_string())
    }

    #[test]
    fn parses_front_matter_metadata() {
        let post = parse(
            "hello.md",
            "---\ntitle: 'Hello, world'\ndate: 2026-10-14\ndescription: A \"quoted\" summary\ndraft: true\n---\n\nHi.\n",
        )
        .unwrap();
        assert_eq!(post.title, "Hello, world");
        assert_eq!(post.date, "2026-10-14");
        assert_eq!(post.description, "A \"quoted\" summary");
        assert!(post.draft);
        assert_eq!(post.output(), Path::new("posts/hello.html"));
        assert_eq!(post.url, "/posts/hello.html");
        assert!(post.body_html().contains("<p>Hi.</p>"));
        assert!(!post.body_html().contains("title:"));
    }

    #[test]
    fn parses_toml_front_matter() {
        let post = parse(
            "note.md",
            "+++\ntitle = \"Note\"\ndate = \"2026-01-02\"\n+++\n\nBody.\n",
        )
        .unwrap();
        assert_eq!(post.title, "Note");
        assert_eq!(post.date, "2026-01-02");
        assert!(!post.draft);
        assert!(post.description.is_empty());
    }

    #[test]
    fn nested_paths_keep_their_structure() {
        let post = parse("notes/rust.md", &text("Rust", "2026-01-01")).unwrap();
        assert_eq!(post.output(), Path::new("posts/notes/rust.html"));
        assert_eq!(post.url, "/posts/notes/rust.html");
        assert_eq!(post.relative, Path::new("posts/notes/rust.md"));
    }

    #[test]
    fn reports_problems_with_the_post_path() {
        let cases = [
            ("no-front-matter.md", "Just text.\n", "missing front matter"),
            ("unclosed.md", "---\ntitle: x\n", "unclosed front matter"),
            (
                "no-title.md",
                "---\ndate: 2026-01-01\n---\n\nBody.\n",
                "no `title`",
            ),
            (
                "empty-title.md",
                "---\ntitle:\ndate: 2026-01-01\n---\n\nBody.\n",
                "title is empty",
            ),
            ("no-date.md", "---\ntitle: x\n---\n\nBody.\n", "no `date`"),
            (
                "bad-date.md",
                "---\ntitle: x\ndate: yesterday\n---\n\nBody.\n",
                "ISO 8601",
            ),
            (
                "bad-draft.md",
                "---\ntitle: x\ndate: 2026-01-01\ndraft: maybe\n---\n\nBody.\n",
                "`draft` must be",
            ),
        ];
        for (name, contents, expected) in cases {
            let error = parse(name, contents).unwrap_err();
            assert!(error.starts_with(&format!("posts/{name}:")), "{error}");
            assert!(error.contains(expected), "{error}");
        }
    }

    #[test]
    fn dates_are_validated() {
        for valid in ["2026-10-14", "2024-02-29", "2000-02-29", "1999-12-31"] {
            assert!(is_date(valid), "{valid}");
        }
        for invalid in [
            "2026-02-29",
            "2026-13-01",
            "2026-00-10",
            "2026-04-31",
            "2026-4-1",
            "26-10-14",
            "2026-10-14T09:30:00Z",
            "٢٠٢٦-١٠-١٤",
        ] {
            assert!(!is_date(invalid), "{invalid}");
        }
    }

    #[test]
    fn sorts_newest_first_then_by_path() {
        let mut posts = vec![
            parse("b.md", &text("B", "2026-01-02")).unwrap(),
            parse("c.md", &text("C", "2026-03-01")).unwrap(),
            parse("a.md", &text("A", "2026-01-02")).unwrap(),
        ];
        sort(&mut posts);
        let titles: Vec<&str> = posts.iter().map(|post| post.title.as_str()).collect();
        assert_eq!(titles, ["C", "A", "B"]);
    }
}
