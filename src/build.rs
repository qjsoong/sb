//! Site generation.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::config::Config;
use crate::post::{self, Post};
use crate::template::{Templates, escape_html};

/// Options for a build.
#[derive(Debug)]
pub struct Options {
    /// Include posts marked `draft: true`.
    pub drafts: bool,
}

/// What a successful build produced.
#[derive(Debug)]
pub struct Report {
    /// Number of post pages written.
    pub posts: usize,
    /// Number of drafts left out of the build.
    pub drafts: usize,
}

impl Report {
    /// A one-line summary for the terminal.
    #[must_use]
    pub fn summary(&self) -> String {
        let mut summary = format!(
            "built {} post{} into public/",
            self.posts,
            if self.posts == 1 { "" } else { "s" }
        );
        if self.drafts > 0 {
            summary.push_str(&format!(
                "; left out {} draft{} (--drafts includes {})",
                self.drafts,
                if self.drafts == 1 { "" } else { "s" },
                if self.drafts == 1 { "it" } else { "them" },
            ));
        }
        summary
    }
}

/// A failed build.
#[derive(Debug)]
pub enum Error {
    /// One or more posts have invalid front matter. Every message names its
    /// file.
    Posts(Vec<String>),
    /// Configuration, templates or I/O.
    Fatal(anyhow::Error),
}

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Self::Fatal(error)
    }
}

/// Builds the site rooted at `root` into `root/public`.
///
/// Every post is validated before anything is written, so a failed build
/// leaves the previous site untouched.
pub fn build(root: &Path, options: &Options) -> std::result::Result<Report, Error> {
    let root = root
        .canonicalize()
        .with_context(|| format!("cannot find site directory {}", root.display()))?;
    if !root.is_dir() {
        return Err(anyhow::anyhow!("{} is not a directory", root.display()).into());
    }
    let config = Config::load(&root)?;
    let templates = Templates::load(&root)?;

    let mut posts = Vec::new();
    let mut errors = Vec::new();
    for path in post::discover(&root)? {
        match Post::load(&root, &path) {
            Ok(post) => posts.push(post),
            Err(message) => errors.push(message),
        }
    }
    if !errors.is_empty() {
        return Err(Error::Posts(errors));
    }

    let mut drafts = 0;
    if !options.drafts {
        drafts = posts.iter().filter(|post| post.draft).count();
        posts.retain(|post| !post.draft);
    }
    post::sort(&mut posts);

    let output = root.join("public");
    reset(&output)?;
    copy_static(&root.join("static"), &output)?;
    for post in &posts {
        let page = render_post(&templates, &config, post)?;
        write_file(&output.join(post.output()), &page)?;
    }
    let index = render_index(&templates, &config, &posts)?;
    write_file(&output.join("index.html"), &index)?;

    Ok(Report {
        posts: posts.len(),
        drafts,
    })
}

fn render_post(templates: &Templates, config: &Config, post: &Post) -> Result<String> {
    let title = escape_html(&post.title);
    let site_title = escape_html(&config.title);
    let lang = escape_html(&config.lang);
    let description = escape_html(&post.description);
    let content = post.body_html();
    templates.post.render(&[
        ("lang", &lang),
        ("title", &title),
        ("site_title", &site_title),
        ("date", &post.date),
        ("description", &description),
        ("content", &content),
    ])
}

fn render_index(
    templates: &Templates,
    config: &Config,
    posts: &[Post],
) -> Result<String> {
    let site_title = escape_html(&config.title);
    let lang = escape_html(&config.lang);
    let description = escape_html(&config.description);
    let list = post_list(posts);
    templates.index.render(&[
        ("lang", &lang),
        ("title", &site_title),
        ("site_title", &site_title),
        ("description", &description),
        ("posts", &list),
    ])
}

/// The index page's list of posts, newest first.
fn post_list(posts: &[Post]) -> String {
    let mut list = String::from("<ul>\n");
    for post in posts {
        let url = escape_html(&post.url);
        let title = escape_html(&post.title);
        let date = escape_html(&post.date);
        writeln!(
            list,
            "  <li><a href=\"{url}\">{title}</a> \
             <time datetime=\"{date}\">{date}</time></li>"
        )
        .expect("writing into a String cannot fail");
    }
    list.push_str("</ul>");
    list
}

/// Removes the directory if it exists and recreates it empty.
fn reset(directory: &Path) -> Result<()> {
    if directory.exists() {
        fs::remove_dir_all(directory)
            .with_context(|| format!("cannot clear {}", directory.display()))?;
    }
    fs::create_dir_all(directory)
        .with_context(|| format!("cannot create {}", directory.display()))
}

/// Copies `static/` into the output directory.
fn copy_static(source: &Path, destination: &Path) -> Result<()> {
    if source.is_dir() {
        copy_directory(source, destination)?;
    }
    Ok(())
}

fn copy_directory(source: &Path, destination: &Path) -> Result<()> {
    for entry in fs::read_dir(source)
        .with_context(|| format!("cannot read {}", source.display()))?
    {
        let entry =
            entry.with_context(|| format!("cannot read {}", source.display()))?;
        let from = entry.path();
        let to = destination.join(entry.file_name());
        let file_type = entry
            .file_type()
            .with_context(|| format!("cannot inspect {}", from.display()))?;
        if file_type.is_dir() {
            fs::create_dir_all(&to)
                .with_context(|| format!("cannot create {}", to.display()))?;
            copy_directory(&from, &to)?;
        } else if file_type.is_file() {
            fs::copy(&from, &to).with_context(|| {
                format!("cannot copy {} to {}", from.display(), to.display())
            })?;
        }
    }
    Ok(())
}

fn write_file(path: &Path, contents: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("cannot create {}", parent.display()))?;
    }
    fs::write(path, contents)
        .with_context(|| format!("cannot write {}", path.display()))
}
