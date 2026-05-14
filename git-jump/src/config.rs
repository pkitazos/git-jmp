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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PartialConfig {
    pub general: Option<PartialGeneral>,
    pub appearance: Option<PartialAppearance>,
}

impl Default for PartialConfig {
    fn default() -> Self {
        Self {
            general: Default::default(),
            appearance: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PartialGeneral {
    pub vim_mode: Option<bool>,
    pub sources: Option<Vec<RefSource>>,
}

impl Default for PartialGeneral {
    fn default() -> Self {
        Self {
            vim_mode: Default::default(),
            sources: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RefSource {
    Local,
    Remote(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PartialAppearance {
    pub quick_select_hint: Option<QuickSelectHint>,
}

impl Default for PartialAppearance {
    fn default() -> Self {
        Self {
            quick_select_hint: Default::default(),
        }
    }
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
}
