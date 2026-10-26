use super::is_listed;
use crate::cache_trend::{Signals, Step, Trend};
use crate::config::{CacheConfig, IconSet};
use crate::icons::cache_icon;
use crate::input::ContextWindowInfo;
use crate::segments::tokens::format_tokens;
use crate::theme::Palette;

/// The name that lists this segment in `segments`.
pub const NAME: &str = "cache";

/// Whether the segment needs the per-Session snapshot: it will render and
/// shows the trend arrow or the last miss.
pub fn needs_snapshot(segments: &[String], config: &CacheConfig) -> bool {
    config.enabled && (config.show_trend || config.show_last_miss) && is_listed(segments, NAME)
}

/// Renders the latest Step's cache hit rate. `signals.trend` compares it with
/// the Session average (see `cache_trend`); `signals.recent_miss` is the size
/// of a recent large Cache Miss.
pub fn render_cache_segment(
    ctx: &ContextWindowInfo,
    signals: Signals,
    config: &CacheConfig,
    icon_set: IconSet,
    palette: &Palette,
) -> Option<String> {
    if !config.enabled {
        return None;
    }

    let hit_pct = ctx
        .current_usage
        .as_ref()
        .and_then(|usage| Step::from(usage).hit_rate())
        .map_or(0, |pct| pct.round() as u64);

    // A recent miss is worth showing even on the fully cold Step it came from.
    let recent_miss = signals.recent_miss.filter(|_| config.show_last_miss);
    if hit_pct == 0 && config.hide_when_zero && recent_miss.is_none() {
        return None;
    }

    let icon = cache_icon(icon_set, config.prefix.as_deref());
    let icon_part = if icon.is_empty() {
        String::new()
    } else {
        format!("{}{}{} ", palette.label, icon, palette.reset)
    };

    let (color, arrow) = match signals.trend.filter(|_| config.show_trend) {
        Some(Trend::Up) => (palette.trend_up, " ↑"),
        Some(Trend::Down) => (palette.trend_down, " ↓"),
        None => (palette.cache, ""),
    };
    let miss = recent_miss
        .map(|tokens| {
            format!(
                " · {}miss {}{}",
                palette.trend_down,
                format_tokens(Some(tokens)),
                palette.reset
            )
        })
        .unwrap_or_default();

    Some(format!(
        "{}{}{}%{}{}{}",
        icon_part, color, hit_pct, arrow, palette.reset, miss
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache_trend::Trend;
    use crate::input::CurrentUsageInfo;

    #[test]
    fn test_render_cache_zero_hidden() {
        let ctx = ContextWindowInfo::default();
        let cfg = CacheConfig::default();
        let p = Palette::for_theme("plain");
        assert!(render_cache_segment(&ctx, Signals::default(), &cfg, IconSet::Plain, &p).is_none());
    }

    #[test]
    fn test_render_cache_active() {
        let ctx = ContextWindowInfo {
            current_usage: Some(CurrentUsageInfo {
                input_tokens: Some(5_457),
                output_tokens: Some(391),
                cache_creation_input_tokens: Some(0),
                cache_read_input_tokens: Some(36_647),
            }),
            ..Default::default()
        };
        let cfg = CacheConfig::default();
        let p = Palette::for_theme("plain");

        let rendered =
            render_cache_segment(&ctx, Signals::default(), &cfg, IconSet::Plain, &p).unwrap();
        assert_eq!(rendered, "Cache: 87%");
    }

    #[test]
    fn test_render_cache_rounds_an_exact_half_up() {
        // 23k of 40k prompt tokens from the cache is exactly 57.5%.
        let ctx = ContextWindowInfo {
            current_usage: Some(CurrentUsageInfo {
                input_tokens: Some(17_000),
                cache_read_input_tokens: Some(23_000),
                ..Default::default()
            }),
            ..Default::default()
        };
        let cfg = CacheConfig::default();
        let p = Palette::for_theme("plain");

        let rendered =
            render_cache_segment(&ctx, Signals::default(), &cfg, IconSet::Plain, &p).unwrap();
        assert_eq!(rendered, "Cache: 58%");
    }

    #[test]
    fn test_render_cache_nerd() {
        let ctx = ContextWindowInfo {
            current_usage: Some(CurrentUsageInfo {
                input_tokens: Some(50),
                output_tokens: Some(10),
                cache_creation_input_tokens: Some(0),
                cache_read_input_tokens: Some(950),
            }),
            ..Default::default()
        };
        let cfg = CacheConfig::default();
        let p = Palette::for_theme("plain");

        let rendered =
            render_cache_segment(&ctx, Signals::default(), &cfg, IconSet::Nerd, &p).unwrap();
        assert_eq!(rendered, "󰘸 95%");
    }

    /// The Step after a cold one: 4k sent, 170k read from the cache (98%).
    fn ctx_after_miss() -> ContextWindowInfo {
        ContextWindowInfo {
            current_usage: Some(CurrentUsageInfo {
                input_tokens: Some(4_000),
                cache_read_input_tokens: Some(170_000),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn render(signals: Signals, cfg: &CacheConfig, theme: &str) -> String {
        let p = Palette::for_theme(theme);
        render_cache_segment(&ctx_after_miss(), signals, cfg, IconSet::Plain, &p).unwrap()
    }

    fn signals(trend: Option<Trend>, recent_miss: Option<u64>) -> Signals {
        Signals { trend, recent_miss }
    }

    #[test]
    fn test_cache_trend_up() {
        let rendered = render(
            signals(Some(Trend::Up), None),
            &CacheConfig::default(),
            "plain",
        );
        assert_eq!(rendered, "Cache: 98% ↑");
    }

    #[test]
    fn test_cache_trend_down() {
        let rendered = render(
            signals(Some(Trend::Down), None),
            &CacheConfig::default(),
            "plain",
        );
        assert_eq!(rendered, "Cache: 98% ↓");
    }

    #[test]
    fn test_cache_trend_colours_the_rate() {
        let p = Palette::for_theme("github");
        let up = render(
            signals(Some(Trend::Up), None),
            &CacheConfig::default(),
            "github",
        );
        assert!(
            up.ends_with(&format!("{}98% ↑{}", p.trend_up, p.reset)),
            "{up:?}"
        );
        let down = render(
            signals(Some(Trend::Down), None),
            &CacheConfig::default(),
            "github",
        );
        assert!(
            down.ends_with(&format!("{}98% ↓{}", p.trend_down, p.reset)),
            "{down:?}"
        );
        let flat = render(Signals::default(), &CacheConfig::default(), "github");
        assert!(
            flat.ends_with(&format!("{}98%{}", p.cache, p.reset)),
            "{flat:?}"
        );
    }

    #[test]
    fn test_cache_trend_can_be_disabled() {
        let cfg = CacheConfig {
            show_trend: false,
            ..Default::default()
        };
        assert_eq!(
            render(signals(Some(Trend::Down), None), &cfg, "plain"),
            "Cache: 98%"
        );
    }

    #[test]
    fn test_recent_miss_follows_the_trend_arrow() {
        // The Step after a miss is well cached: the arrow says so, and the
        // miss stays visible next to it.
        let rendered = render(
            signals(Some(Trend::Up), Some(168_800)),
            &CacheConfig::default(),
            "plain",
        );
        assert_eq!(rendered, "Cache: 98% ↑ · miss 169k");
    }

    #[test]
    fn test_recent_miss_coloured_as_a_drop() {
        let p = Palette::for_theme("github");
        let rendered = render(
            signals(None, Some(168_800)),
            &CacheConfig::default(),
            "github",
        );
        assert!(
            rendered.ends_with(&format!(" · {}miss 169k{}", p.trend_down, p.reset)),
            "{rendered:?}"
        );
    }

    #[test]
    fn test_recent_miss_can_be_disabled() {
        let cfg = CacheConfig {
            show_last_miss: false,
            ..Default::default()
        };
        assert_eq!(
            render(signals(None, Some(168_800)), &cfg, "plain"),
            "Cache: 98%"
        );
    }

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn test_snapshot_needs_a_listed_enabled_segment_showing_a_signal() {
        let cfg = CacheConfig::default();
        assert!(needs_snapshot(&names(&["cache"]), &cfg));
        assert!(!needs_snapshot(&names(&["tokens"]), &cfg));
        let trend_only = CacheConfig {
            show_last_miss: false,
            ..Default::default()
        };
        assert!(needs_snapshot(&names(&["cache"]), &trend_only));
        let miss_only = CacheConfig {
            show_trend: false,
            ..Default::default()
        };
        assert!(needs_snapshot(&names(&["cache"]), &miss_only));
        let neither = CacheConfig {
            show_trend: false,
            show_last_miss: false,
            ..Default::default()
        };
        assert!(!needs_snapshot(&names(&["cache"]), &neither));
        let disabled = CacheConfig {
            enabled: false,
            ..Default::default()
        };
        assert!(!needs_snapshot(&names(&["cache"]), &disabled));
    }

    #[test]
    fn test_cold_step_still_shows_its_miss() {
        // `hide_when_zero` hides a segment with nothing to say, not the
        // Refresh right after a fully cold Step.
        let ctx = ContextWindowInfo {
            current_usage: Some(CurrentUsageInfo {
                input_tokens: Some(168_800),
                cache_read_input_tokens: Some(0),
                ..Default::default()
            }),
            ..Default::default()
        };
        let p = Palette::for_theme("plain");
        let cfg = CacheConfig::default();
        let rendered = render_cache_segment(
            &ctx,
            signals(Some(Trend::Down), Some(168_800)),
            &cfg,
            IconSet::Plain,
            &p,
        );
        assert_eq!(rendered.as_deref(), Some("Cache: 0% ↓ · miss 169k"));
        let hidden_miss = CacheConfig {
            show_last_miss: false,
            ..Default::default()
        };
        assert!(
            render_cache_segment(
                &ctx,
                signals(Some(Trend::Down), Some(168_800)),
                &hidden_miss,
                IconSet::Plain,
                &p
            )
            .is_none()
        );
    }
}
