//! Page templates and placeholder substitution.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};

/// The built-in post page template.
const POST: &str = include_str!("../templates/post.html");

/// The built-in index page template.
const INDEX: &str = include_str!("../templates/index.html");

/// The two page templates, each either a built-in default or an override in
/// the site's `templates` directory.
#[derive(Debug)]
pub struct Templates {
    /// Used for every post page.
    pub post: Template,
    /// Used for the index page.
    pub index: Template,
}

impl Templates {
    /// Loads `templates/post.html` and `templates/index.html` below the site
    /// root when they exist.
    pub fn load(root: &Path) -> Result<Self> {
        let directory = root.join("templates");
        Ok(Self {
            post: Template::load(
                &directory.join("post.html"),
                POST,
                "built-in post template",
            )?,
            index: Template::load(
                &directory.join("index.html"),
                INDEX,
                "built-in index template",
            )?,
        })
    }
}

/// A page template with `{{placeholder}}` markers.
#[derive(Debug)]
pub struct Template {
    name: String,
    text: String,
}

impl Template {
    fn load(path: &Path, default: &str, default_name: &str) -> Result<Self> {
        if path.is_file() {
            let text = fs::read_to_string(path)
                .with_context(|| format!("cannot read {}", path.display()))?;
            Ok(Self {
                name: path.display().to_string(),
                text,
            })
        } else {
            Ok(Self {
                name: default_name.to_string(),
                text: default.to_string(),
            })
        }
    }

    /// Replaces every `{{name}}` marker with the matching value.
    ///
    /// Values are inserted verbatim and never substituted again. A `{{...}}`
    /// that does not look like a name (for example a CSS or JavaScript
    /// snippet) is literal text, but an unknown word-shaped name is an error,
    /// so a typo cannot leave a literal `{{...}}` in a page.
    pub fn render(&self, values: &[(&str, &str)]) -> Result<String> {
        let mut output = String::with_capacity(self.text.len());
        let mut rest = self.text.as_str();
        let mut unknown: Vec<&str> = Vec::new();
        while let Some(start) = rest.find("{{") {
            output.push_str(&rest[..start]);
            let after = &rest[start + 2..];
            let Some(end) = after.find("}}") else {
                output.push_str(&rest[start..]);
                rest = "";
                break;
            };
            let name = after[..end].trim();
            if !is_name(name) {
                // Not a marker: keep the braces and keep scanning.
                output.push_str("{{");
                rest = after;
                continue;
            }
            if let Some((_, value)) = values.iter().find(|(key, _)| *key == name) {
                output.push_str(value);
            } else if !unknown.contains(&name) {
                unknown.push(name);
            }
            rest = &after[end + 2..];
        }
        output.push_str(rest);
        if !unknown.is_empty() {
            let available: Vec<&str> = values.iter().map(|(key, _)| *key).collect();
            bail!(
                "{}: unknown placeholder {}; available: {}",
                self.name,
                render_names(&unknown),
                render_names(&available),
            );
        }
        Ok(output)
    }
}

/// Whether a marker looks like a placeholder name.
fn is_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
}

/// Formats names as `{{name}}` and joins them for an error message.
fn render_names(names: &[&str]) -> String {
    names
        .iter()
        .map(|name| format!("{{{{{name}}}}}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Escapes text for HTML text and double-quoted attribute values.
#[must_use]
pub fn escape_html(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    fn template(text: &str) -> Template {
        Template {
            name: "test template".to_string(),
            text: text.to_string(),
        }
    }

    #[test]
    fn substitutes_placeholders() {
        let page = template("<h1>{{title}}</h1>{{content}}")
            .render(&[("title", "Hi"), ("content", "<p>x</p>")])
            .unwrap();
        assert_eq!(page, "<h1>Hi</h1><p>x</p>");
    }

    #[test]
    fn values_are_never_substituted_again() {
        let page = template("{{content}}")
            .render(&[("content", "{{title}}"), ("title", "Hi")])
            .unwrap();
        assert_eq!(page, "{{title}}");
    }

    #[test]
    fn reports_unknown_placeholders() {
        let error = template("{{titel}}")
            .render(&[("title", "Hi")])
            .unwrap_err();
        let message = format!("{error:#}");
        assert!(message.contains("{{titel}}"), "{message}");
        assert!(message.contains("{{title}}"), "{message}");
    }

    #[test]
    fn non_marker_braces_are_literal_text() {
        let page = template("a {{ b {{title}} c {{x + y}}")
            .render(&[("title", "T")])
            .unwrap();
        assert_eq!(page, "a {{ b T c {{x + y}}");
    }

    #[test]
    fn unclosed_markers_are_literal_text() {
        let page = template("a {{ title").render(&[("title", "x")]).unwrap();
        assert_eq!(page, "a {{ title");
    }

    #[test]
    fn escapes_html() {
        assert_eq!(
            escape_html("<a href=\"b\">'c' & d</a>"),
            "&lt;a href=&quot;b&quot;&gt;&#39;c&#39; &amp; d&lt;/a&gt;"
        );
    }
}
