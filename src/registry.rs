use crate::config::{ModelMapping, Provider};
use std::collections::HashMap;

pub fn builtin_providers() -> HashMap<String, Provider> {
    let mut m = HashMap::new();

    m.insert("claude".into(), Provider {
        base_url: "https://api.anthropic.com".into(),
        api_key: None,
        auth_token: None,
        models: None,
    });

    m.insert("glm".into(), Provider {
        base_url: "https://open.z.ai/api/paas/v4".into(),
        api_key: None,
        auth_token: None,
        models: Some(ModelMapping {
            haiku: "glm-4.5-air".into(),
            sonnet: "glm-4.7".into(),
            opus: "glm-4.7".into(),
        }),
    });

    m.insert("glm-flash".into(), Provider {
        base_url: "https://open.z.ai/api/paas/v4".into(),
        api_key: None,
        auth_token: None,
        models: Some(ModelMapping {
            haiku: "glm-4.7-flashx".into(),
            sonnet: "glm-4.7-flashx".into(),
            opus: "glm-4.7-flashx".into(),
        }),
    });

    m.insert("glm-5".into(), Provider {
        base_url: "https://open.z.ai/api/paas/v4".into(),
        api_key: None,
        auth_token: None,
        models: Some(ModelMapping {
            haiku: "glm-4.7-flashx".into(),
            sonnet: "glm-5-code".into(),
            opus: "glm-5".into(),
        }),
    });

    m.insert("minimax".into(), Provider {
        base_url: "https://api.minimax.io/anthropic/v1".into(),
        api_key: None,
        auth_token: None,
        models: Some(ModelMapping {
            haiku: "MiniMax-M2".into(),
            sonnet: "MiniMax-M2.5".into(),
            opus: "MiniMax-M2.5".into(),
        }),
    });

    m.insert("minimax-fast".into(), Provider {
        base_url: "https://api.minimax.io/anthropic/v1".into(),
        api_key: None,
        auth_token: None,
        models: Some(ModelMapping {
            haiku: "MiniMax-M2".into(),
            sonnet: "MiniMax-M2.5-Lightning".into(),
            opus: "MiniMax-M2.5".into(),
        }),
    });

    m
}
