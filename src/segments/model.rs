use crate::config::{IconSet, ModelConfig};
use crate::icons::model_icon;
use crate::input::ModelInfo;
use crate::theme::Palette;

pub fn render_model_segment(
    model: Option<&ModelInfo>,
    config: &ModelConfig,
    icon_set: IconSet,
    palette: &Palette,
) -> Option<String> {
    if !config.enabled {
        return None;
    }

    let name = model
        .map(|m| m.resolved_name())
        .unwrap_or_else(|| "Gemini".to_string());
    let icon = model_icon(icon_set, config.prefix.as_deref());

    let formatted = if config.brackets {
        format!("[{name}]")
    } else {
        name
    };

    Some(format!(
        "{}{}{}{}{}",
        palette.label, icon, palette.model, formatted, palette.reset
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_model_default() {
        let m = ModelInfo {
            display_name: Some("Gemini 3.8 Flash (High)".to_string()),
            ..Default::default()
        };
        let cfg = ModelConfig::default();
        let p = Palette::for_theme("plain");

        let rendered = render_model_segment(Some(&m), &cfg, IconSet::Plain, &p).unwrap();
        assert_eq!(rendered, "[Gemini 3.8 Flash (High)]");
    }

    #[test]
    fn test_render_model_disabled() {
        let cfg = ModelConfig {
            enabled: false,
            ..Default::default()
        };
        let p = Palette::for_theme("plain");
        assert!(render_model_segment(None, &cfg, IconSet::Plain, &p).is_none());
    }

    #[test]
    fn test_render_model_nerd_no_brackets() {
        let m = ModelInfo {
            id: Some("gemini-pro".to_string()),
            ..Default::default()
        };
        let cfg = ModelConfig {
            enabled: true,
            brackets: false,
            prefix: None,
        };
        let p = Palette::for_theme("plain");
        let rendered = render_model_segment(Some(&m), &cfg, IconSet::Nerd, &p).unwrap();
        assert_eq!(rendered, "󰚩 gemini-pro");
    }
}
