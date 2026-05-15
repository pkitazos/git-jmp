use anyhow::Result;
use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub general: General,
    pub appearance: Appearance,
}

#[derive(Debug, Clone, Default)]
pub struct General {
    pub vim_mode: bool,
    pub sources: Vec<RefSource>,
}

#[derive(Debug, Clone, Default)]
pub struct Appearance {
    pub quick_select_hint: QuickSelectHint,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PartialConfig {
    pub general: Option<PartialGeneral>,
    pub appearance: Option<PartialAppearance>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PartialGeneral {
    pub vim_mode: Option<bool>,
    pub sources: Option<Vec<RefSource>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RefSource {
    Local,
    Remote(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PartialAppearance {
    pub quick_select_hint: Option<QuickSelectHint>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QuickSelectHint {
    #[default]
    Full,
    Compact,
    Hidden,
}

pub fn parse_config(path: PathBuf) -> Result<PartialConfig> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) => {
            return match err.kind() {
                std::io::ErrorKind::NotFound => Ok(PartialConfig::default()),
                _ => Err(err.into()),
            };
        }
    };
    let config = toml::de::from_slice::<PartialConfig>(&bytes)?;
    Ok(config)
}

pub fn merge(global: PartialConfig, local: PartialConfig) -> Config {
    let zero = Config::default();

    let PartialGeneral {
        vim_mode: local_vim,
        sources: local_sources,
    } = local.general.unwrap_or_default();

    let PartialGeneral {
        vim_mode: global_vim,
        sources: global_sources,
    } = global.general.unwrap_or_default();

    let PartialAppearance {
        quick_select_hint: local_quick_hint,
    } = local.appearance.unwrap_or_default();

    let PartialAppearance {
        quick_select_hint: global_quick_hint,
    } = global.appearance.unwrap_or_default();

    Config {
        general: General {
            vim_mode: local_vim.or(global_vim).unwrap_or(zero.general.vim_mode),
            sources: local_sources
                .or(global_sources)
                .unwrap_or(zero.general.sources),
        },
        appearance: Appearance {
            quick_select_hint: local_quick_hint
                .or(global_quick_hint)
                .unwrap_or(zero.appearance.quick_select_hint),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sources_with_local_and_remotes() {
        let input = r#"
[general]
sources = ["local", { remote = "origin" }]
"#;

        let config: PartialConfig = toml::from_str(input).unwrap();
        let general = config.general.unwrap();
        let sources = general.sources.unwrap();

        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0], RefSource::Local);
        assert_eq!(sources[1], RefSource::Remote("origin".to_string()));
    }

    #[test]
    fn parses_empty_string_as_defaults() {
        let config: PartialConfig = toml::from_str("").unwrap();
        assert!(config.general.is_none());
        assert!(config.appearance.is_none());
    }

    #[test]
    fn parses_vim_mode() {
        let input = r#"
[general]
vim_mode = true
"#;
        let config: PartialConfig = toml::from_str(input).unwrap();
        assert_eq!(config.general.unwrap().vim_mode, Some(true));
    }

    #[test]
    fn parses_quick_select_hint_variants() {
        for (input, expected) in [
            ("full", QuickSelectHint::Full),
            ("compact", QuickSelectHint::Compact),
            ("hidden", QuickSelectHint::Hidden),
        ] {
            let toml = format!("[appearance]\nquick_select_hint = \"{}\"", input);
            let config: PartialConfig = toml::from_str(&toml).unwrap();
            assert_eq!(config.appearance.unwrap().quick_select_hint, Some(expected));
        }
    }

    #[test]
    fn parses_full_config() {
        let input = r#"
[general]
vim_mode = true
sources = ["local", { remote = "upstream" }]

[appearance]
quick_select_hint = "hidden"
"#;
        let config: PartialConfig = toml::from_str(input).unwrap();
        let general = config.general.unwrap();
        assert_eq!(general.vim_mode, Some(true));
        assert_eq!(
            general.sources.unwrap(),
            vec![RefSource::Local, RefSource::Remote("upstream".to_string())]
        );
        assert_eq!(
            config.appearance.unwrap().quick_select_hint,
            Some(QuickSelectHint::Hidden)
        );
    }

    #[test]
    fn merge_both_empty_gives_defaults() {
        let config = merge(PartialConfig::default(), PartialConfig::default());
        assert_eq!(config.general.vim_mode, false);
        assert!(config.general.sources.is_empty());
        assert_eq!(config.appearance.quick_select_hint, QuickSelectHint::Full);
    }

    #[test]
    fn merge_global_used_when_local_absent() {
        let global = PartialConfig {
            general: Some(PartialGeneral {
                vim_mode: Some(true),
                sources: Some(vec![RefSource::Local]),
            }),
            appearance: Some(PartialAppearance {
                quick_select_hint: Some(QuickSelectHint::Compact),
            }),
        };
        let config = merge(global, PartialConfig::default());
        assert_eq!(config.general.vim_mode, true);
        assert_eq!(config.general.sources, vec![RefSource::Local]);
        assert_eq!(
            config.appearance.quick_select_hint,
            QuickSelectHint::Compact
        );
    }

    #[test]
    fn merge_local_overrides_global() {
        let global = PartialConfig {
            general: Some(PartialGeneral {
                vim_mode: Some(true),
                sources: Some(vec![RefSource::Local]),
            }),
            appearance: Some(PartialAppearance {
                quick_select_hint: Some(QuickSelectHint::Compact),
            }),
        };
        let local = PartialConfig {
            general: Some(PartialGeneral {
                vim_mode: Some(false),
                sources: Some(vec![RefSource::Remote("origin".to_string())]),
            }),
            appearance: Some(PartialAppearance {
                quick_select_hint: Some(QuickSelectHint::Hidden),
            }),
        };
        let config = merge(global, local);
        assert_eq!(config.general.vim_mode, false);
        assert_eq!(
            config.general.sources,
            vec![RefSource::Remote("origin".to_string())]
        );
        assert_eq!(config.appearance.quick_select_hint, QuickSelectHint::Hidden);
    }

    #[test]
    fn merge_local_partial_override() {
        let global = PartialConfig {
            general: Some(PartialGeneral {
                vim_mode: Some(true),
                sources: Some(vec![RefSource::Local]),
            }),
            appearance: Some(PartialAppearance {
                quick_select_hint: Some(QuickSelectHint::Compact),
            }),
        };
        let local = PartialConfig {
            general: Some(PartialGeneral {
                vim_mode: Some(false),
                ..Default::default()
            }),
            ..Default::default()
        };
        let config = merge(global, local);
        assert_eq!(config.general.vim_mode, false);
        assert_eq!(config.general.sources, vec![RefSource::Local]);
        assert_eq!(
            config.appearance.quick_select_hint,
            QuickSelectHint::Compact
        );
    }
}
