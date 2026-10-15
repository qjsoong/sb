//! End-to-end tests for the `sb` binary.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

struct Sandbox {
    directory: PathBuf,
}

impl Sandbox {
    fn new() -> Self {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let directory = std::env::temp_dir()
            .join(format!("sb-cli-test-{}-{id}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("create sandbox");
        Self { directory }
    }

    fn write(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.directory.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(&path, contents).expect("write file");
        path
    }

    fn read(&self, name: &str) -> String {
        fs::read_to_string(self.directory.join(name)).expect("read file")
    }

    fn exists(&self, name: &str) -> bool {
        self.directory.join(name).exists()
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_sb"))
            .args(args)
            .current_dir(&self.directory)
            .output()
            .expect("run sb")
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn stderr(output: &Output) -> String {
    // Windows renders paths with backslashes; normalise so the assertions can
    // use the same `/`-separated paths on every platform.
    String::from_utf8_lossy(&output.stderr).replace('\\', "/")
}

fn post(title: &str, date: &str, body: &str) -> String {
    format!("---\ntitle: {title}\ndate: {date}\n---\n\n{body}\n")
}

fn copy_directory(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("create directory");
    for entry in fs::read_dir(from).expect("read directory") {
        let entry = entry.expect("read entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("file type").is_dir() {
            copy_directory(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("copy file");
        }
    }
}

#[test]
fn builds_a_site() {
    let sandbox = Sandbox::new();
    sandbox.write("site/sb.toml", "title = \"My <blog>\"\n");
    sandbox.write(
        "site/posts/first.md",
        &post("First & foremost", "2026-01-02", "Hello **world**."),
    );
    sandbox.write(
        "site/posts/second.md",
        &post("Second", "2026-03-04", "Another."),
    );
    sandbox.write("site/static/robots.txt", "User-agent: *\n");

    let output = sandbox.run(&["build", "site"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(
        stderr(&output).contains("built 2 posts"),
        "{}",
        stderr(&output)
    );
    assert!(stdout(&output).is_empty());

    let index = sandbox.read("site/public/index.html");
    assert!(index.contains("<title>My &lt;blog&gt;</title>"), "{index}");
    assert!(index.contains("First &amp; foremost"), "{index}");
    assert!(
        index.find("Second") < index.find("First"),
        "newest post first: {index}"
    );
    assert!(!index.contains("{{"), "{index}");

    let page = sandbox.read("site/public/posts/first.html");
    assert!(page.contains("<h1>First &amp; foremost</h1>"), "{page}");
    assert!(page.contains("<strong>world</strong>"), "{page}");
    assert!(
        page.contains("<title>First &amp; foremost — My &lt;blog&gt;</title>"),
        "{page}"
    );
    assert!(
        page.contains("<time datetime=\"2026-01-02\">2026-01-02</time>"),
        "{page}"
    );

    assert_eq!(sandbox.read("site/public/robots.txt"), "User-agent: *\n");
}

#[test]
fn builds_the_current_directory_by_default() {
    let sandbox = Sandbox::new();
    sandbox.write("sb.toml", "title = \"Blog\"\n");
    sandbox.write("posts/hello.md", &post("Hello", "2026-01-01", "Hi."));

    let output = sandbox.run(&["build"]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(sandbox.exists("public/posts/hello.html"));
}

#[test]
fn drafts_are_left_out_unless_requested() {
    let sandbox = Sandbox::new();
    sandbox.write("site/sb.toml", "title = \"Blog\"\n");
    sandbox.write(
        "site/posts/live.md",
        &post("Live", "2026-01-01", "Published."),
    );
    sandbox.write(
        "site/posts/wip.md",
        "---\ntitle: WIP\ndate: 2026-02-01\ndraft: true\n---\n\nNot yet.\n",
    );

    let output = sandbox.run(&["build", "site"]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(
        stderr(&output).contains("left out 1 draft"),
        "{}",
        stderr(&output)
    );
    let index = sandbox.read("site/public/index.html");
    assert!(index.contains("Live"), "{index}");
    assert!(!index.contains("WIP"), "{index}");
    assert!(!sandbox.exists("site/public/posts/wip.html"));

    let output = sandbox.run(&["build", "site", "--drafts"]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(sandbox.exists("site/public/posts/wip.html"));
    assert!(sandbox.read("site/public/index.html").contains("WIP"));
}

#[test]
fn reports_every_invalid_post_and_keeps_the_old_site() {
    let sandbox = Sandbox::new();
    sandbox.write("site/sb.toml", "title = \"Blog\"\n");
    sandbox.write("site/posts/ok.md", &post("OK", "2026-01-01", "Fine."));
    sandbox.write(
        "site/posts/no-title.md",
        "---\ndate: 2026-01-01\n---\n\nBody.\n",
    );
    sandbox.write("site/posts/bad-date.md", &post("Bad", "yesterday", "Body."));
    sandbox.write("site/public/keep.txt", "previous build\n");

    let output = sandbox.run(&["build", "site"]);
    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    let errors = stderr(&output);
    assert!(errors.contains("posts/no-title.md"), "{errors}");
    assert!(errors.contains("posts/bad-date.md"), "{errors}");
    assert_eq!(errors.matches("error:").count(), 2, "{errors}");
    assert_eq!(sandbox.read("site/public/keep.txt"), "previous build\n");
    assert!(!sandbox.exists("site/public/index.html"));
}

#[test]
fn configuration_errors_exit_with_two() {
    let sandbox = Sandbox::new();
    sandbox.write("site/posts/hello.md", &post("Hello", "2026-01-01", "Hi."));

    let missing = sandbox.run(&["build", "site"]);
    assert_eq!(missing.status.code(), Some(2), "{}", stderr(&missing));
    assert!(stderr(&missing).contains("sb.toml"), "{}", stderr(&missing));

    sandbox.write("site/sb.toml", "title = \"Blog\"\nunknown = true\n");
    let unknown = sandbox.run(&["build", "site"]);
    assert_eq!(unknown.status.code(), Some(2), "{}", stderr(&unknown));
    assert!(stderr(&unknown).contains("unknown"), "{}", stderr(&unknown));
}

#[test]
fn templates_can_be_overridden() {
    let sandbox = Sandbox::new();
    sandbox.write("site/sb.toml", "title = \"Blog\"\n");
    sandbox.write("site/posts/hello.md", &post("Hello", "2026-01-01", "Hi."));
    sandbox.write(
        "site/templates/post.html",
        "<!DOCTYPE html>\n<html><head><title>{{title}}</title></head>\n\
         <body><article data-page=\"custom\">{{content}}</article></body></html>\n",
    );

    let output = sandbox.run(&["build", "site"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let page = sandbox.read("site/public/posts/hello.html");
    assert!(page.contains("<article data-page=\"custom\">"), "{page}");
    // The index page keeps the built-in template.
    assert!(
        sandbox
            .read("site/public/index.html")
            .contains("<h1>Blog</h1>")
    );
}

#[test]
fn unknown_placeholders_are_errors() {
    let sandbox = Sandbox::new();
    sandbox.write("site/sb.toml", "title = \"Blog\"\n");
    sandbox.write("site/posts/hello.md", &post("Hello", "2026-01-01", "Hi."));
    sandbox.write(
        "site/templates/index.html",
        "<html><body>{{posts}}{{titel}}</body></html>\n",
    );

    let output = sandbox.run(&["build", "site"]);
    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
    let errors = stderr(&output);
    assert!(errors.contains("{{titel}}"), "{errors}");
    assert!(errors.contains("templates/index.html"), "{errors}");
}

#[test]
fn builds_the_example_site() {
    let sandbox = Sandbox::new();
    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("example");
    copy_directory(&example, &sandbox.directory.join("example"));

    let output = sandbox.run(&["build", "example"]);
    assert!(output.status.success(), "{}", stderr(&output));

    let index = sandbox.read("example/public/index.html");
    assert!(index.contains("Hello, world"), "{index}");
    assert!(index.contains("Markdown, rendered by mdk"), "{index}");
    assert!(sandbox.exists("example/public/robots.txt"));
}
