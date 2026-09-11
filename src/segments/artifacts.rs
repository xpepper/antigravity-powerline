use crate::config::{ArtifactsConfig, IconSet};
use crate::icons::artifacts_icon;
use crate::theme::Palette;

pub fn render_artifacts_segment(
    artifact_count: Option<u64>,
    config: &ArtifactsConfig,
    icon_set: IconSet,
    palette: &Palette,
) -> Option<String> {
    if !config.enabled {
        return None;
    }

    let count = artifact_count.unwrap_or(0);
    if count == 0 && config.hide_when_zero {
        return None;
    }

    let icon = artifacts_icon(icon_set, config.prefix.as_deref());
    let icon_part = if icon.is_empty() {
        String::new()
    } else {
        format!("{}{}{} ", palette.label, icon, palette.reset)
    };

    Some(format!(
        "{}{}{}{}",
        icon_part, palette.artifacts, count, palette.reset
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_artifacts_zero_hidden() {
        let cfg = ArtifactsConfig::default();
        let p = Palette::for_theme("plain");
        assert!(render_artifacts_segment(Some(0), &cfg, IconSet::Plain, &p).is_none());
        assert!(render_artifacts_segment(None, &cfg, IconSet::Plain, &p).is_none());
    }

    #[test]
    fn test_render_artifacts_active() {
        let cfg = ArtifactsConfig::default();
        let p = Palette::for_theme("plain");
        let rendered = render_artifacts_segment(Some(3), &cfg, IconSet::Plain, &p).unwrap();
        assert_eq!(rendered, "Artifacts: 3");
    }

    #[test]
    fn test_render_artifacts_nerd() {
        let cfg = ArtifactsConfig::default();
        let p = Palette::for_theme("plain");
        let rendered = render_artifacts_segment(Some(2), &cfg, IconSet::Nerd, &p).unwrap();
        assert_eq!(rendered, "󰏗 2");
    }
}
