//! Site configuration, read from `sb.toml`.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

/// The site configuration.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Site title, shown in the header and used as the index page title.
    pub title: String,
    /// Language tag for the `lang` attribute.
    #[serde(default = "default_lang")]
    pub lang: String,
    /// Optional one-line description of the site.
    #[serde(default)]
    pub description: String,
}

fn default_lang() -> String {
    "en".to_string()
}

impl Config {
    /// Loads `sb.toml` from the site root.
    pub fn load(root: &Path) -> Result<Self> {
        let path = root.join("sb.toml");
        let text = fs::read_to_string(&path).with_context(|| {
            format!(
                "cannot read {}; a site needs an sb.toml with at least a title",
                path.display()
            )
        })?;
        let config: Self = toml::from_str(&text)
            .with_context(|| format!("invalid {}", path.display()))?;
        if config.title.trim().is_empty() {
            bail!("{}: title must not be empty", path.display());
        }
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_configuration_with_defaults() {
        let config: Config = toml::from_str("title = \"My blog\"").unwrap();
        assert_eq!(config.title, "My blog");
        assert_eq!(config.lang, "en");
        assert!(config.description.is_empty());

        let full: Config = toml::from_str(
            "title = \"博客\"\nlang = \"zh\"\ndescription = \"Notes\"\n",
        )
        .unwrap();
        assert_eq!(full.lang, "zh");
        assert_eq!(full.description, "Notes");
    }

    #[test]
    fn rejects_unknown_and_missing_keys() {
        assert!(toml::from_str::<Config>("titel = \"typo\"").is_err());
        assert!(toml::from_str::<Config>("lang = \"en\"").is_err());
    }
}
