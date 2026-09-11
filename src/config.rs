use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub use crate::icons::IconSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Style {
    #[default]
    Minimal,
    Powerline,
    Capsule,
    Plain,
}

impl std::str::FromStr for Style {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "minimal" => Ok(Style::Minimal),
            "powerline" => Ok(Style::Powerline),
            "capsule" => Ok(Style::Capsule),
            "plain" => Ok(Style::Plain),
            other => Err(format!("Unknown style: {other}")),
        }
    }
}

impl Serialize for IconSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            IconSet::Plain => serializer.serialize_str("plain"),
            IconSet::Nerd => serializer.serialize_str("nerd"),
            IconSet::Emoji => serializer.serialize_str("emoji"),
        }
    }
}

impl<'de> Deserialize<'de> for IconSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<IconSet>().map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub style: Style,
    #[serde(default)]
    pub icon_set: IconSet,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_segments")]
    pub segments: Vec<String>,
    #[serde(default)]
    pub model: ModelConfig,
    #[serde(default)]
    pub git: GitConfig,
    #[serde(default)]
    pub tokens: TokensConfig,
    #[serde(default)]
    pub quota: QuotaConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub total_tokens: TotalTokensConfig,
    #[serde(default)]
    pub artifacts: ArtifactsConfig,
    #[serde(default)]
    pub pr: PrConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            style: Style::default(),
            icon_set: IconSet::default(),
            theme: default_theme(),
            segments: default_segments(),
            model: ModelConfig::default(),
            git: GitConfig::default(),
            pr: PrConfig::default(),
            tokens: TokensConfig::default(),
            quota: QuotaConfig::default(),
            cache: CacheConfig::default(),
            total_tokens: TotalTokensConfig::default(),
            artifacts: ArtifactsConfig::default(),
        }
    }
}

fn default_theme() -> String {
    "colorblind".to_string()
}

fn default_segments() -> Vec<String> {
    vec![
        "model".to_string(),
        "git".to_string(),
        "pr".to_string(),
        "tokens".to_string(),
        "quota".to_string(),
        "cache".to_string(),
        "total_tokens".to_string(),
    ]
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub prefix: Option<String>,
    #[serde(default = "default_true")]
    pub brackets: bool,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prefix: None,
            brackets: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub prefix: Option<String>,
    #[serde(default = "default_true")]
    pub parentheses: bool,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prefix: None,
            parentheses: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokensConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub prefix: Option<String>,
    #[serde(default = "default_true")]
    pub show_percentage: bool,
    #[serde(default = "default_alert_threshold")]
    pub alert_threshold: u64,
    #[serde(default = "default_alert_icon")]
    pub alert_icon: String,
}

fn default_alert_threshold() -> u64 {
    100_000
}

fn default_alert_icon() -> String {
    "⚠️ ".to_string()
}

impl Default for TokensConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prefix: None,
            show_percentage: true,
            alert_threshold: default_alert_threshold(),
            alert_icon: default_alert_icon(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub prefix: Option<String>,
    #[serde(default = "default_preferred_quotas")]
    pub preferred_keys: Vec<String>,
}

fn default_preferred_quotas() -> Vec<String> {
    vec!["gemini-5h".to_string(), "gemini-weekly".to_string()]
}

impl Default for QuotaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prefix: None,
            preferred_keys: default_preferred_quotas(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub prefix: Option<String>,
    #[serde(default = "default_true")]
    pub hide_when_zero: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prefix: None,
            hide_when_zero: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotalTokensConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub prefix: Option<String>,
    #[serde(default = "default_true")]
    pub hide_when_zero: bool,
}

impl Default for TotalTokensConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prefix: None,
            hide_when_zero: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactsConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub prefix: Option<String>,
    #[serde(default = "default_true")]
    pub hide_when_zero: bool,
}

impl Default for ArtifactsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prefix: None,
            hide_when_zero: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub prefix: Option<String>,
    #[serde(default = "default_true")]
    pub hyperlinks: bool,
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_seconds: u64,
}

fn default_cache_ttl() -> u64 {
    60
}

impl Default for PrConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prefix: None,
            hyperlinks: true,
            cache_ttl_seconds: default_cache_ttl(),
        }
    }
}

impl Config {
    pub fn default_config_path() -> Option<PathBuf> {
        let gemini_dir = dirs::home_dir()?.join(".gemini");
        Some(gemini_dir.join("powerline.toml"))
    }

    pub fn load_from_file_or_default(custom_path: Option<&Path>) -> Self {
        if let Some(path) = custom_path
            && let Ok(content) = std::fs::read_to_string(path)
            && let Ok(cfg) = toml::from_str::<Config>(&content)
        {
            return cfg;
        }

        if let Some(home) = dirs::home_dir() {
            let antigravity_path = home.join(".gemini").join("antigravity-powerline.toml");
            if antigravity_path.exists()
                && let Ok(content) = std::fs::read_to_string(&antigravity_path)
                && let Ok(cfg) = toml::from_str::<Config>(&content)
            {
                return cfg;
            }

            let standard_path = home.join(".gemini").join("powerline.toml");
            if standard_path.exists()
                && let Ok(content) = std::fs::read_to_string(&standard_path)
                && let Ok(cfg) = toml::from_str::<Config>(&content)
            {
                return cfg;
            }
        }

        Self::default()
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = Config::default();
        assert_eq!(cfg.style, Style::Minimal);
        assert_eq!(cfg.icon_set, IconSet::Plain);
        assert_eq!(cfg.theme, "colorblind");
        assert_eq!(cfg.segments.len(), 7);
        assert!(cfg.model.enabled);
        assert!(cfg.git.enabled);
        assert!(cfg.pr.enabled);
        assert!(cfg.pr.hyperlinks);
        assert_eq!(cfg.pr.cache_ttl_seconds, 60);
    }

    #[test]
    fn test_toml_roundtrip() {
        let cfg = Config::default();
        let toml_str = cfg.to_toml_string().unwrap();
        assert!(toml_str.contains("style = \"minimal\""));
        assert!(toml_str.contains("theme = \"colorblind\""));

        let loaded: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(loaded.style, Style::Minimal);
        assert_eq!(loaded.theme, "colorblind");
    }

    #[test]
    fn test_parse_style() {
        assert_eq!("minimal".parse::<Style>().unwrap(), Style::Minimal);
        assert_eq!("powerline".parse::<Style>().unwrap(), Style::Powerline);
        assert_eq!("capsule".parse::<Style>().unwrap(), Style::Capsule);
        assert_eq!("plain".parse::<Style>().unwrap(), Style::Plain);
    }

    #[test]
    fn test_parse_icon_set() {
        assert_eq!("plain".parse::<IconSet>().unwrap(), IconSet::Plain);
        assert_eq!("nerd".parse::<IconSet>().unwrap(), IconSet::Nerd);
        assert_eq!("emoji".parse::<IconSet>().unwrap(), IconSet::Emoji);
    }
}
