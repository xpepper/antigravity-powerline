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
    pub artifacts: &'static str,
    pub total_tokens: &'static str,
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
                artifacts: "\x1b[93m",      // Yellow
                total_tokens: "\x1b[94m",   // Blue
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
                artifacts: "\x1b[33m",      // Yellow
                total_tokens: "\x1b[34m",   // Blue
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
                artifacts: "\x1b[93m",      // Yellow
                total_tokens: "\x1b[95m",   // Magenta
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
                artifacts: "",
                total_tokens: "",
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
                artifacts: "\x1b[93m",      // Yellow
                total_tokens: "\x1b[94m",   // Blue
            },
        }
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
    }

    #[test]
    fn test_colorblind_palette() {
        let p = Palette::for_theme("colorblind");
        assert_eq!(p.reset, "\x1b[0m");
        assert_eq!(p.sep, "\x1b[90m  │  \x1b[0m");
        assert!(!p.tokens_alert.is_empty());
    }

    #[test]
    fn test_theme_fallback() {
        let p = Palette::for_theme("nonexistent-theme");
        assert_eq!(p.reset, "\x1b[0m");
    }
}
