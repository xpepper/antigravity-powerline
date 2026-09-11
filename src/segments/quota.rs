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

            parts.push(format!(
                "{}{}: {}{}%{}",
                palette.label, label, color, pct, palette.reset
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
        assert_eq!(rendered, "Quota: 5h: 96%  wk: 88%");
    }

    #[test]
    fn test_render_quota_empty() {
        let quotas = HashMap::new();
        let cfg = QuotaConfig::default();
        let p = Palette::for_theme("plain");
        assert!(render_quota_segment(&quotas, &cfg, IconSet::Plain, &p).is_none());
    }
}
