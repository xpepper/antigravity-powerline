use super::is_listed;
use crate::config::{CacheExpiryConfig, IconSet};
use crate::icons::cache_expiry_icon;
use crate::input::ContextWindowInfo;
use crate::segments::tokens::format_tokens;
use crate::theme::Palette;

/// The name that lists this segment in `segments`.
pub const NAME: &str = "cache_expiry";

/// Whether the segment will render, so callers can skip the idle snapshot
/// (a state file read and write) when nothing would show it.
pub fn is_visible(segments: &[String], config: &CacheExpiryConfig) -> bool {
    config.enabled && is_listed(segments, NAME)
}

/// Renders the long-break reminder: how long the Session has been idle and
/// that the Prompt Cache is likely cold. It only claims "likely": the idle
/// time is all that is measured, not the provider's cache. `idle_seconds` is
/// the time since the Session's latest Step (see `idle`), `None` when unknown.
pub fn render_cache_expiry_segment(
    ctx: &ContextWindowInfo,
    idle_seconds: Option<u64>,
    config: &CacheExpiryConfig,
    icon_set: IconSet,
    palette: &Palette,
) -> Option<String> {
    let idle = idle_seconds?;
    let context_tokens = ctx.active_tokens();
    if !config.enabled || idle < config.idle_seconds || context_tokens < config.min_tokens {
        return None;
    }

    let icon = cache_expiry_icon(icon_set, config.prefix.as_deref());
    let r = palette.reset;
    let context = if config.show_tokens {
        format!(" · {} context", format_tokens(Some(context_tokens)))
    } else {
        String::new()
    };

    Some(format!(
        "{}{}{} {}{}{}{} · cache likely cold{}{}",
        palette.label,
        icon,
        r,
        palette.tokens_alert,
        format_idle(idle),
        r,
        palette.dim,
        context,
        r
    ))
}

/// Idle time as `45s`, `54m`, `1h` or `1h 12m`.
fn format_idle(seconds: u64) -> String {
    let (hours, minutes) = (seconds / 3_600, seconds % 3_600 / 60);
    match (hours, minutes) {
        (0, 0) => format!("{seconds}s"),
        (0, m) => format!("{m}m"),
        (h, 0) => format!("{h}h"),
        (h, m) => format!("{h}h {m}m"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_with(tokens: u64) -> ContextWindowInfo {
        ContextWindowInfo {
            total_input_tokens: Some(tokens),
            ..Default::default()
        }
    }

    fn render(
        ctx: &ContextWindowInfo,
        idle: Option<u64>,
        cfg: &CacheExpiryConfig,
    ) -> Option<String> {
        render_cache_expiry_segment(ctx, idle, cfg, IconSet::Plain, &Palette::for_theme("plain"))
    }

    #[test]
    fn test_reminds_after_thirty_idle_minutes() {
        let cfg = CacheExpiryConfig::default();
        assert_eq!(
            render(&ctx_with(92_000), Some(1_800), &cfg).as_deref(),
            Some("Idle: 30m · cache likely cold")
        );
    }

    #[test]
    fn test_quiet_through_ordinary_pauses() {
        let cfg = CacheExpiryConfig::default();
        assert_eq!(render(&ctx_with(92_000), Some(300), &cfg), None);
        assert_eq!(render(&ctx_with(92_000), Some(1_799), &cfg), None);
    }

    #[test]
    fn test_quiet_below_the_token_floor() {
        let cfg = CacheExpiryConfig::default();
        assert_eq!(render(&ctx_with(49_999), Some(3_600), &cfg), None);
        assert!(render(&ctx_with(50_000), Some(3_600), &cfg).is_some());
    }

    #[test]
    fn test_floor_reads_the_current_context_size() {
        // `current_tokens` wins over `total_input_tokens`, as in `tokens`.
        let cfg = CacheExpiryConfig::default();
        let ctx = ContextWindowInfo {
            current_tokens: Some(10_000),
            ..ctx_with(92_000)
        };
        assert_eq!(render(&ctx, Some(3_600), &cfg), None);
    }

    #[test]
    fn test_quiet_when_disabled() {
        let cfg = CacheExpiryConfig {
            enabled: false,
            ..Default::default()
        };
        assert_eq!(render(&ctx_with(92_000), Some(3_600), &cfg), None);
    }

    #[test]
    fn test_quiet_when_idle_time_is_unknown() {
        let cfg = CacheExpiryConfig::default();
        assert_eq!(render(&ctx_with(92_000), None, &cfg), None);
    }

    #[test]
    fn test_idle_time_reads_in_minutes_then_hours() {
        assert_eq!(format_idle(45), "45s");
        assert_eq!(format_idle(1_800), "30m");
        assert_eq!(format_idle(3_299), "54m");
        assert_eq!(format_idle(3_600), "1h");
        assert_eq!(format_idle(4_320), "1h 12m");
        assert_eq!(format_idle(93_600), "26h");
    }

    #[test]
    fn test_context_size_is_opt_in() {
        let cfg = CacheExpiryConfig {
            show_tokens: true,
            ..Default::default()
        };
        assert_eq!(
            render(&ctx_with(92_000), Some(3_299), &cfg).as_deref(),
            Some("Idle: 54m · cache likely cold · 92k context")
        );
    }

    #[test]
    fn test_emoji_icon_and_theme_colors() {
        let p = Palette::for_theme("colorblind");
        let rendered = render_cache_expiry_segment(
            &ctx_with(92_000),
            Some(3_299),
            &CacheExpiryConfig::default(),
            IconSet::Emoji,
            &p,
        )
        .unwrap();
        assert!(rendered.contains("💤"));
        assert!(rendered.contains(&format!("{}54m{}", p.tokens_alert, p.reset)));
        assert!(rendered.contains(&format!("{} · cache likely cold{}", p.dim, p.reset)));
        assert!(!rendered.contains("context"));
    }

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn test_visible_when_listed_and_enabled() {
        let cfg = CacheExpiryConfig::default();
        assert!(is_visible(&names(&["tokens", "cache_expiry"]), &cfg));
    }

    #[test]
    fn test_hidden_when_not_listed() {
        let cfg = CacheExpiryConfig::default();
        assert!(!is_visible(&names(&["tokens", "cache"]), &cfg));
    }

    #[test]
    fn test_hidden_when_disabled() {
        let cfg = CacheExpiryConfig {
            enabled: false,
            ..Default::default()
        };
        assert!(!is_visible(&names(&["cache_expiry"]), &cfg));
    }
}
