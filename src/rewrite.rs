use crate::config::Provider;

fn classify_model(model: &str) -> Option<&'static str> {
    let lower = model.to_lowercase();
    if lower.contains("haiku") {
        Some("haiku")
    } else if lower.contains("sonnet") {
        Some("sonnet")
    } else if lower.contains("opus") {
        Some("opus")
    } else {
        None
    }
}

pub fn rewrite_model(model: &str, provider: &Provider) -> String {
    let mapping = match &provider.models {
        Some(m) => m,
        None => return model.to_string(),
    };

    match classify_model(model) {
        Some("haiku") => mapping.haiku.clone(),
        Some("sonnet") => mapping.sonnet.clone(),
        Some("opus") => mapping.opus.clone(),
        _ => model.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ModelMapping, Provider};

    fn glm_provider() -> Provider {
        Provider {
            base_url: "https://open.z.ai/api/paas/v4".into(),
            api_key: Some("sk-test".into()),
            auth_token: None,
            models: Some(ModelMapping {
                haiku: "glm-4.5-air".into(),
                sonnet: "glm-4.7".into(),
                opus: "glm-4.7".into(),
            }),
        }
    }

    fn passthrough_provider() -> Provider {
        Provider {
            base_url: "https://api.anthropic.com".into(),
            api_key: None,
            auth_token: None,
            models: None,
        }
    }

    #[test]
    fn test_rewrite_sonnet() {
        assert_eq!(rewrite_model("claude-sonnet-4-20250514", &glm_provider()), "glm-4.7");
    }

    #[test]
    fn test_rewrite_haiku() {
        assert_eq!(rewrite_model("claude-haiku-3-20250101", &glm_provider()), "glm-4.5-air");
    }

    #[test]
    fn test_rewrite_opus() {
        assert_eq!(rewrite_model("claude-opus-4-20250514", &glm_provider()), "glm-4.7");
    }

    #[test]
    fn test_passthrough_no_mapping() {
        assert_eq!(rewrite_model("claude-sonnet-4-20250514", &passthrough_provider()), "claude-sonnet-4-20250514");
    }

    #[test]
    fn test_unknown_model_passthrough() {
        assert_eq!(rewrite_model("some-random-model", &glm_provider()), "some-random-model");
    }
}
