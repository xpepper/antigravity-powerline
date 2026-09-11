use crate::config::{CacheConfig, IconSet};
use crate::icons::cache_icon;
use crate::input::ContextWindowInfo;
use crate::theme::Palette;

pub fn render_cache_segment(
    ctx: &ContextWindowInfo,
    config: &CacheConfig,
    icon_set: IconSet,
    palette: &Palette,
) -> Option<String> {
    if !config.enabled {
        return None;
    }

    let cached = ctx.cached_tokens();
    if cached == 0 && config.hide_when_zero {
        return None;
    }

    let usage = ctx.current_usage.as_ref();
    let inp = usage.and_then(|u| u.input_tokens).unwrap_or(0);
    let total_prompt = inp + cached;

    let hit_pct = if total_prompt > 0 {
        ((cached as f64 / total_prompt as f64) * 100.0).round() as u64
    } else {
        0
    };

    if hit_pct == 0 && config.hide_when_zero {
        return None;
    }

    let icon = cache_icon(icon_set, config.prefix.as_deref());
    let val_str = format!("{}%", hit_pct);

    Some(
        format!(
            "{}{}{} {}{}{}",
            palette.label, icon, palette.reset, palette.cache, val_str, palette.reset
        )
        .trim_start()
        .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::CurrentUsageInfo;

    #[test]
    fn test_render_cache_zero_hidden() {
        let ctx = ContextWindowInfo::default();
        let cfg = CacheConfig::default();
        let p = Palette::for_theme("plain");
        assert!(render_cache_segment(&ctx, &cfg, IconSet::Plain, &p).is_none());
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

        let rendered = render_cache_segment(&ctx, &cfg, IconSet::Plain, &p).unwrap();
        assert_eq!(rendered, "Cache: 87%");
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

        let rendered = render_cache_segment(&ctx, &cfg, IconSet::Nerd, &p).unwrap();
        assert_eq!(rendered, "󰘸  95%");
    }
}
