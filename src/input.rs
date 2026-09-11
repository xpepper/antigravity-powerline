use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AntigravityInput {
    pub cwd: Option<String>,
    pub session_id: Option<String>,
    pub conversation_id: Option<String>,
    pub model: Option<ModelInfo>,
    pub workspace: Option<WorkspaceInfo>,
    pub version: Option<String>,
    #[serde(default)]
    pub context_window: ContextWindowInfo,
    #[serde(default)]
    pub quota: HashMap<String, QuotaInfo>,
    pub agent_state: Option<String>,
    pub artifact_count: Option<u64>,
    pub plan_tier: Option<String>,
    pub terminal_width: Option<usize>,
}

impl AntigravityInput {
    pub fn from_json(json_str: &str) -> Self {
        serde_json::from_str(json_str).unwrap_or_default()
    }

    pub fn resolved_cwd(&self) -> Option<&str> {
        self.cwd
            .as_deref()
            .or_else(|| self.workspace.as_ref().and_then(|w| w.current_dir.as_deref()))
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: Option<String>,
    pub display_name: Option<String>,
    pub name: Option<String>,
    pub effort: Option<String>,
}

impl ModelInfo {
    pub fn resolved_name(&self) -> String {
        self.display_name
            .as_deref()
            .or(self.name.as_deref())
            .or(self.id.as_deref())
            .unwrap_or("Gemini")
            .to_string()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub current_dir: Option<String>,
    pub project_dir: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CurrentUsageInfo {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cache_creation_input_tokens: Option<u64>,
    pub cache_read_input_tokens: Option<u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextWindowInfo {
    pub total_input_tokens: Option<u64>,
    pub total_output_tokens: Option<u64>,
    pub context_window_size: Option<u64>,
    pub used_percentage: Option<f64>,
    pub remaining_percentage: Option<f64>,
    pub current_tokens: Option<u64>,
    pub max_tokens: Option<u64>,
    pub current_usage: Option<CurrentUsageInfo>,
}

impl ContextWindowInfo {
    pub fn active_tokens(&self) -> u64 {
        if let Some(tokens) = self.current_tokens {
            tokens
        } else if let Some(tokens) = self.total_input_tokens {
            tokens
        } else {
            0
        }
    }

    pub fn limit_tokens(&self) -> u64 {
        if let Some(limit) = self.max_tokens {
            limit
        } else if let Some(limit) = self.context_window_size {
            limit
        } else {
            0
        }
    }

    pub fn effective_used_percentage(&self) -> f64 {
        if let Some(pct) = self.used_percentage {
            pct
        } else {
            let active = self.active_tokens() as f64;
            let limit = self.limit_tokens() as f64;
            if limit > 0.0 {
                (active / limit) * 100.0
            } else {
                0.0
            }
        }
    }

    pub fn cached_tokens(&self) -> u64 {
        self.current_usage
            .as_ref()
            .and_then(|u| u.cache_read_input_tokens)
            .unwrap_or(0)
    }

    pub fn total_turn_tokens(&self) -> u64 {
        let inp = self.total_input_tokens.unwrap_or(0);
        let out = self.total_output_tokens.unwrap_or(0);
        inp + out
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuotaInfo {
    pub remaining_fraction: Option<f64>,
    pub reset_time: Option<String>,
    pub reset_in_seconds: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_input() {
        let input = AntigravityInput::from_json("");
        assert!(input.model.is_none());
        assert_eq!(input.context_window.active_tokens(), 0);
        assert_eq!(input.context_window.effective_used_percentage(), 0.0);
    }

    #[test]
    fn test_parse_broken_json() {
        let input = AntigravityInput::from_json("{broken json...");
        assert!(input.session_id.is_none());
    }

    #[test]
    fn test_parse_full_antigravity_payload() {
        let payload = r#"{
            "cwd": "/repo",
            "session_id": "sess-123",
            "model": {
                "id": "gemini-3.8-flash",
                "display_name": "Gemini 3.8 Flash (High)",
                "effort": "high"
            },
            "workspace": {
                "current_dir": "/repo"
            },
            "context_window": {
                "total_input_tokens": 47427,
                "total_output_tokens": 4713,
                "context_window_size": 1048576,
                "used_percentage": 4.52,
                "current_usage": {
                    "input_tokens": 5457,
                    "output_tokens": 391,
                    "cache_read_input_tokens": 36647
                }
            },
            "quota": {
                "gemini-5h": {
                    "remaining_fraction": 0.961,
                    "reset_in_seconds": 17276
                },
                "gemini-weekly": {
                    "remaining_fraction": 0.881,
                    "reset_in_seconds": 563846
                }
            },
            "artifact_count": 2
        }"#;

        let input = AntigravityInput::from_json(payload);
        assert_eq!(input.resolved_cwd(), Some("/repo"));
        assert_eq!(input.session_id.as_deref(), Some("sess-123"));

        let model = input.model.unwrap();
        assert_eq!(model.resolved_name(), "Gemini 3.8 Flash (High)");
        assert_eq!(model.effort.as_deref(), Some("high"));

        assert_eq!(input.context_window.active_tokens(), 47427);
        assert_eq!(input.context_window.limit_tokens(), 1048576);
        assert_eq!(input.context_window.cached_tokens(), 36647);
        assert_eq!(input.context_window.total_turn_tokens(), 47427 + 4713);
        assert!((input.context_window.effective_used_percentage() - 4.52).abs() < 0.001);

        assert_eq!(input.artifact_count, Some(2));

        let q_5h = input.quota.get("gemini-5h").unwrap();
        assert!((q_5h.remaining_fraction.unwrap() - 0.961).abs() < 0.001);
        assert_eq!(q_5h.reset_in_seconds, Some(17276));
    }
}
