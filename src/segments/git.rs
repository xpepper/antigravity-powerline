use crate::config::{GitConfig, IconSet};
use crate::icons::git_icon;
use crate::theme::Palette;

pub fn render_git_segment(
    branch: Option<&str>,
    config: &GitConfig,
    icon_set: IconSet,
    palette: &Palette,
) -> Option<String> {
    if !config.enabled {
        return None;
    }

    let b = branch?;
    if b.is_empty() {
        return None;
    }

    let icon = git_icon(icon_set, config.prefix.as_deref());
    let formatted = if config.parentheses {
        format!("({b})")
    } else {
        b.to_string()
    };

    Some(format!(
        "{}{}{}{}{}",
        palette.label, icon, palette.git, formatted, palette.reset
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_git_plain() {
        let cfg = GitConfig::default();
        let p = Palette::for_theme("plain");
        let rendered = render_git_segment(Some("main"), &cfg, IconSet::Plain, &p).unwrap();
        assert_eq!(rendered, "(main)");
    }

    #[test]
    fn test_render_git_nerd_no_parentheses() {
        let cfg = GitConfig {
            enabled: true,
            parentheses: false,
            prefix: None,
        };
        let p = Palette::for_theme("plain");
        let rendered = render_git_segment(Some("feature-x"), &cfg, IconSet::Nerd, &p).unwrap();
        assert_eq!(rendered, "󰘬 feature-x");
    }

    #[test]
    fn test_render_git_none() {
        let cfg = GitConfig::default();
        let p = Palette::for_theme("plain");
        assert!(render_git_segment(None, &cfg, IconSet::Plain, &p).is_none());
    }
}
