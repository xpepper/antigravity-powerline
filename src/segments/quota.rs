use crate::config::{IconSet, QuotaConfig};
use crate::icons::quota_icon;
use crate::input::QuotaInfo;
use crate::theme::Palette;
use std::collections::HashMap;

pub fn render_quota_segment(
    quotas: &HashMap<String, QuotaInfo>,
    config: &QuotaConfig,
    icon_set: IconSet,
    palette: &Palette,
) -> Option<String> {
    if !config.enabled || quotas.is_empty() {
        return None;
    }

    let mut parts = Vec::new();

    for key in &config.preferred_keys {
        if let Some(info) = quotas.get(key)
            && let Some(frac) = info.remaining_fraction
        {
            let pct = (frac * 100.0).round() as u64;
            let label = match key.as_str() {
                "gemini-5h" => "5h",
                "gemini-weekly" => "wk",
                "3p-5h" => "3p-5h",
                "3p-weekly" => "3p-wk",
                other => other,
            };

            let color = if frac < 0.2 {
                palette.quota_alert
            } else if frac < 0.5 {
                palette.quota_warn
            } else {
                palette.quota
            };

            let reset = if config.show_reset && config.reset_keys.contains(key) {
                info.reset_in_seconds
                    .map(|secs| {
                        format!(
                            " {}({}){}",
                            palette.label,
                            format_reset(secs),
                            palette.reset
                        )
                    })
                    .unwrap_or_default()
            } else {
                String::new()
            };

            parts.push(format!(
                "{}{}: {}{}%{}{}",
                palette.label, label, color, pct, palette.reset, reset
            ));
        }
    }

    if parts.is_empty() {
        return None;
    }

    let icon = quota_icon(icon_set, config.prefix.as_deref());
    let joined_parts = parts.join("  ");

    if icon.is_empty() {
        Some(joined_parts)
    } else {
        Some(format!(
            "{}{}{} {}",
            palette.label, icon, palette.reset, joined_parts
        ))
    }
}

/// Formats seconds until quota reset as a compact human-readable duration.
fn format_reset(secs: u64) -> String {
    if secs < 60 {
        return "<1m".to_string();
    }

    let total_minutes = secs / 60;
    let days = total_minutes / (60 * 24);
    let hours = (total_minutes / 60) % 24;
    let minutes = total_minutes % 60;

    if days > 0 {
        format!("{}d {}h", days, hours)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_quota_segment() {
        let mut quotas = HashMap::new();
        quotas.insert(
            "gemini-5h".to_string(),
            QuotaInfo {
                remaining_fraction: Some(0.961),
                reset_time: None,
                reset_in_seconds: Some(17276),
            },
        );
        quotas.insert(
            "gemini-weekly".to_string(),
            QuotaInfo {
                remaining_fraction: Some(0.881),
                reset_time: None,
                reset_in_seconds: Some(563846),
            },
        );

        let cfg = QuotaConfig::default();
        let p = Palette::for_theme("plain");

        let rendered = render_quota_segment(&quotas, &cfg, IconSet::Plain, &p).unwrap();
        assert_eq!(rendered, "Quota: 5h: 96% (4h 47m)  wk: 88%");
    }

    #[test]
    fn test_render_quota_segment_without_reset() {
        let mut quotas = HashMap::new();
        quotas.insert(
            "gemini-5h".to_string(),
            QuotaInfo {
                remaining_fraction: Some(0.24),
                reset_time: None,
                reset_in_seconds: Some(9360),
            },
        );

        let cfg = QuotaConfig {
            show_reset: false,
            ..QuotaConfig::default()
        };
        let p = Palette::for_theme("plain");

        let rendered = render_quota_segment(&quotas, &cfg, IconSet::Plain, &p).unwrap();
        assert_eq!(rendered, "Quota: 5h: 24%");
    }

    #[test]
    fn test_format_reset() {
        assert_eq!(format_reset(30), "<1m");
        assert_eq!(format_reset(9360), "2h 36m");
        assert_eq!(format_reset(1800), "30m");
        assert_eq!(format_reset(563846), "6d 12h");
    }

    #[test]
    fn test_render_quota_empty() {
        let quotas = HashMap::new();
        let cfg = QuotaConfig::default();
        let p = Palette::for_theme("plain");
        assert!(render_quota_segment(&quotas, &cfg, IconSet::Plain, &p).is_none());
    }
}
