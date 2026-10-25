#[derive(Debug, Clone)]
pub struct Palette {
    pub reset: &'static str,
    pub dim: &'static str,
    pub label: &'static str,
    pub sep: &'static str,
    pub model: &'static str,
    pub git: &'static str,
    pub tokens_normal: &'static str,
    pub tokens_warn: &'static str,
    pub tokens_alert: &'static str,
    pub quota: &'static str,
    pub quota_warn: &'static str,
    pub quota_alert: &'static str,
    pub cache: &'static str,
    pub trend_up: &'static str,
    pub trend_down: &'static str,
    pub artifacts: &'static str,
    pub total_tokens: &'static str,
    pub pr: &'static str,
    pub pr_approved: &'static str,
    pub pr_changes_requested: &'static str,
    pub pr_draft: &'static str,
}

impl Palette {
    pub fn for_theme(theme_name: &str) -> Self {
        match theme_name.to_lowercase().as_str() {
            "github" | "default" => Self {
                reset: "\x1b[0m",
                dim: "\x1b[90m",
                label: "\x1b[90m",
                sep: "\x1b[90m  │  \x1b[0m",
                model: "\x1b[1;34m",        // Blue
                git: "\x1b[35m",            // Magenta
                tokens_normal: "\x1b[97m",  // White
                tokens_warn: "\x1b[33m",    // Yellow
                tokens_alert: "\x1b[1;31m", // Red
                quota: "\x1b[92m",          // Green
                quota_warn: "\x1b[93m",     // Yellow
                quota_alert: "\x1b[91m",    // Red
                cache: "\x1b[36m",          // Cyan
                trend_up: "\x1b[92m",   // Bright green
                trend_down: "\x1b[91m", // Bright red
                artifacts: "\x1b[93m",      // Yellow
                total_tokens: "\x1b[94m",   // Blue
                pr: "\x1b[33m",             // Yellow
                pr_approved: "\x1b[92m",    // Bright green
                pr_changes_requested: "\x1b[1;31m", // Bold red
                pr_draft: "\x1b[90m",       // Grey
            },
            "nord" => Self {
                reset: "\x1b[0m",
                dim: "\x1b[90m",
                label: "\x1b[90m",
                sep: "\x1b[90m  │  \x1b[0m",
                model: "\x1b[1;36m",        // Frost Cyan
                git: "\x1b[95m",            // Purple
                tokens_normal: "\x1b[97m",  // Snow Storm White
                tokens_warn: "\x1b[33m",    // Yellow
                tokens_alert: "\x1b[1;31m", // Aurora Red
                quota: "\x1b[32m",          // Aurora Green
                quota_warn: "\x1b[33m",     // Yellow
                quota_alert: "\x1b[31m",    // Red
                cache: "\x1b[36m",          // Cyan
                trend_up: "\x1b[92m",   // Bright green
                trend_down: "\x1b[91m", // Bright red
                artifacts: "\x1b[33m",      // Yellow
                total_tokens: "\x1b[34m",   // Blue
                pr: "\x1b[33m",             // Yellow
                pr_approved: "\x1b[92m",    // Bright green
                pr_changes_requested: "\x1b[1;31m", // Bold red
                pr_draft: "\x1b[90m",       // Grey
            },
            "tokyo-night" => Self {
                reset: "\x1b[0m",
                dim: "\x1b[90m",
                label: "\x1b[90m",
                sep: "\x1b[90m  │  \x1b[0m",
                model: "\x1b[1;35m",        // Magenta
                git: "\x1b[94m",            // Cyan/Blue
                tokens_normal: "\x1b[97m",  // White
                tokens_warn: "\x1b[33m",    // Yellow
                tokens_alert: "\x1b[1;31m", // Red
                quota: "\x1b[92m",          // Green
                quota_warn: "\x1b[93m",     // Yellow
                quota_alert: "\x1b[91m",    // Red
                cache: "\x1b[96m",          // Bright Cyan
                trend_up: "\x1b[92m",   // Bright green
                trend_down: "\x1b[91m", // Bright red
                artifacts: "\x1b[93m",      // Yellow
                total_tokens: "\x1b[95m",   // Magenta
                pr: "\x1b[93m",             // Yellow
                pr_approved: "\x1b[92m",    // Bright green
                pr_changes_requested: "\x1b[1;31m", // Bold red
                pr_draft: "\x1b[90m",       // Grey
            },
            "plain" => Self {
                reset: "",
                dim: "",
                label: "",
                sep: "  |  ",
                model: "",
                git: "",
                tokens_normal: "",
                tokens_warn: "",
                tokens_alert: "",
                quota: "",
                quota_warn: "",
                quota_alert: "",
                cache: "",
                trend_up: "",
                trend_down: "",
                artifacts: "",
                total_tokens: "",
                pr: "",
                pr_approved: "",
                pr_changes_requested: "",
                pr_draft: "",
            },
            _ /* "colorblind" and fallback */ => Self {
                reset: "\x1b[0m",
                dim: "\x1b[90m",
                label: "\x1b[90m",
                sep: "\x1b[90m  │  \x1b[0m",
                model: "\x1b[1;36m",        // Cyan
                git: "\x1b[35m",            // Magenta
                tokens_normal: "\x1b[97m",  // Bright white
                tokens_warn: "\x1b[33m",    // Yellow
                tokens_alert: "\x1b[1;31m", // Red
                quota: "\x1b[94m",          // Blue
                quota_warn: "\x1b[33m",     // Yellow
                quota_alert: "\x1b[1;31m",  // Red
                cache: "\x1b[36m",          // Cyan
                trend_up: "\x1b[94m",          // Bright blue
                trend_down: "\x1b[38;5;208m",  // Orange
                artifacts: "\x1b[93m",      // Yellow
                total_tokens: "\x1b[94m",   // Blue
                pr: "\x1b[93m",             // Yellow
                // Blue/orange instead of green/red, distinguishable with
                // red-green color blindness.
                pr_approved: "\x1b[94m",            // Bright blue
                pr_changes_requested: "\x1b[38;5;208m", // Orange
                pr_draft: "\x1b[90m",               // Grey
            },
        }
    }

    /// True for a palette with no colours, which emits no escape codes.
    pub fn is_plain(&self) -> bool {
        self.reset.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_palette() {
        let p = Palette::for_theme("plain");
        assert_eq!(p.reset, "");
        assert_eq!(p.sep, "  |  ");
        assert_eq!(p.model, "");
        assert_eq!(p.pr, "");
    }

    #[test]
    fn test_only_the_plain_palette_is_plain() {
        assert!(Palette::for_theme("plain").is_plain());
        for theme in ["github", "nord", "tokyo-night", "colorblind"] {
            assert!(!Palette::for_theme(theme).is_plain(), "{theme}");
        }
    }

    #[test]
    fn test_colorblind_palette() {
        let p = Palette::for_theme("colorblind");
        assert_eq!(p.reset, "\x1b[0m");
        assert_eq!(p.sep, "\x1b[90m  │  \x1b[0m");
        assert!(!p.tokens_alert.is_empty());
        assert_eq!(p.pr, "\x1b[93m");
    }

    #[test]
    fn test_theme_fallback() {
        let p = Palette::for_theme("nonexistent-theme");
        assert_eq!(p.reset, "\x1b[0m");
    }

    #[test]
    fn test_review_state_colours_are_blue_and_orange_for_colorblind() {
        let cb = Palette::for_theme("colorblind");
        assert_eq!(cb.pr_approved, "\x1b[94m");
        assert_eq!(cb.pr_changes_requested, "\x1b[38;5;208m");

        let gh = Palette::for_theme("github");
        assert_eq!(gh.pr_approved, "\x1b[92m");
        assert_eq!(gh.pr_changes_requested, "\x1b[1;31m");
        assert_eq!(gh.pr_draft, "\x1b[90m");

        // Only the colorblind palette (the fallback) is blue/orange.
        for theme in ["nord", "tokyo-night"] {
            let palette = Palette::for_theme(theme);
            assert_eq!(palette.pr_approved, "\x1b[92m", "{theme}");
            assert_eq!(palette.pr_changes_requested, "\x1b[1;31m", "{theme}");
            assert_eq!(palette.pr_draft, "\x1b[90m", "{theme}");
        }

        let plain = Palette::for_theme("plain");
        assert_eq!(plain.pr_approved, "");
        assert_eq!(plain.pr_changes_requested, "");
        assert_eq!(plain.pr_draft, "");
    }
}
