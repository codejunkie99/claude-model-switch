use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMapping {
    pub haiku: String,
    pub sonnet: String,
    pub opus: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub base_url: String,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub auth_token: Option<String>,
    #[serde(default)]
    pub models: Option<ModelMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    pub active: String,
    pub providers: HashMap<String, Provider>,
}

impl ProfileConfig {
    pub fn config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not find home directory")?;
        Ok(home.join(".claude").join("model-profiles.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn active_provider(&self) -> Result<&Provider> {
        self.providers
            .get(&self.active)
            .with_context(|| format!("Active provider '{}' not found in profiles", self.active))
    }

    pub fn provider(&self, name: &str) -> Result<&Provider> {
        self.providers
            .get(name)
            .with_context(|| format!("Provider '{}' not found in profiles", name))
    }
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self {
            active: "claude".to_string(),
            providers: HashMap::from([(
                "claude".to_string(),
                Provider {
                    base_url: "https://api.anthropic.com".to_string(),
                    api_key: None,
                    auth_token: None,
                    models: None,
                },
            )]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ProfileConfig::default();
        assert_eq!(config.active, "claude");
        assert!(config.providers.contains_key("claude"));
    }

    #[test]
    fn test_serialization_roundtrip() {
        let config = ProfileConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: ProfileConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.active, config.active);
    }

    #[test]
    fn test_active_provider_found() {
        let config = ProfileConfig::default();
        assert!(config.active_provider().is_ok());
    }

    #[test]
    fn test_active_provider_missing() {
        let config = ProfileConfig {
            active: "nonexistent".to_string(),
            providers: HashMap::new(),
        };
        assert!(config.active_provider().is_err());
    }

    #[test]
    fn test_provider_lookup_found() {
        let config = ProfileConfig::default();
        assert!(config.provider("claude").is_ok());
    }

    #[test]
    fn test_provider_lookup_missing() {
        let config = ProfileConfig::default();
        assert!(config.provider("missing").is_err());
    }
}
