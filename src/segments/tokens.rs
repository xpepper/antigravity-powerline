use crate::config::{IconSet, TokensConfig};
use crate::icons::tokens_icon;
use crate::input::ContextWindowInfo;
use crate::theme::Palette;

pub fn format_tokens(n: Option<u64>) -> String {
    let count = match n {
        Some(v) if v > 0 => v,
        _ => return "0".to_string(),
    };

    if count >= 1_000_000 {
        format!("{:.1}M", count as f64 / 1_000_000.0)
    } else if count >= 10_000 {
        format!("{:.0}k", count as f64 / 1_000.0)
    } else if count >= 1_000 {
        format!("{:.1}k", count as f64 / 1_000.0)
    } else {
        count.to_string()
    }
}

pub fn render_tokens_segment(
    ctx: &ContextWindowInfo,
    config: &TokensConfig,
    icon_set: IconSet,
    palette: &Palette,
) -> Option<String> {
    if !config.enabled {
        return None;
    }

    let r = palette.reset;
    let d = palette.dim;
    let lbl = palette.label;

    let curr_tokens = ctx.active_tokens();
    let max_tokens = ctx.limit_tokens();
    let pct = ctx.effective_used_percentage().round() as u64;

    let tok_curr_str = format_tokens(Some(curr_tokens));
    let tok_max_str = format_tokens(Some(max_tokens));

    let is_alert = curr_tokens > config.alert_threshold || pct >= 50;
    let is_warn = pct >= 30 && !is_alert;

    let (alert_icon, curr_styled) = if is_alert {
        (
            config.alert_icon.as_str(),
            format!("{}{}{}", palette.tokens_alert, tok_curr_str, r),
        )
    } else if is_warn {
        ("", format!("{}{}{}", palette.tokens_warn, tok_curr_str, r))
    } else {
        (
            "",
            format!("{}{}{}", palette.tokens_normal, tok_curr_str, r),
        )
    };

    let max_styled = format!("{}{}{}", d, tok_max_str, r);
    let pct_styled = if config.show_percentage {
        format!(" ({}{}%{})", d, pct, r)
    } else {
        String::new()
    };

    let icon = if is_alert {
        ""
    } else {
        tokens_icon(icon_set, config.prefix.as_deref())
    };

    Some(
        format!(
            "{}{}{} {}{}/{}{}",
            lbl, icon, r, alert_icon, curr_styled, max_styled, pct_styled
        )
        .trim_start()
        .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_tokens() {
        assert_eq!(format_tokens(None), "0");
        assert_eq!(format_tokens(Some(0)), "0");
        assert_eq!(format_tokens(Some(450)), "450");
        assert_eq!(format_tokens(Some(1_200)), "1.2k");
        assert_eq!(format_tokens(Some(47_427)), "47k");
        assert_eq!(format_tokens(Some(1_048_576)), "1.0M");
    }

    #[test]
    fn test_render_tokens_normal() {
        let ctx = ContextWindowInfo {
            total_input_tokens: Some(47_427),
            context_window_size: Some(1_048_576),
            used_percentage: Some(4.52),
            ..Default::default()
        };
        let cfg = TokensConfig::default();
        let p = Palette::for_theme("plain");

        let rendered = render_tokens_segment(&ctx, &cfg, IconSet::Plain, &p).unwrap();
        assert_eq!(rendered, "Tokens: 47k/1.0M (5%)");
    }

    #[test]
    fn test_render_tokens_alert() {
        let ctx = ContextWindowInfo {
            total_input_tokens: Some(150_000),
            context_window_size: Some(200_000),
            used_percentage: Some(75.0),
            ..Default::default()
        };
        let cfg = TokensConfig::default();
        let p = Palette::for_theme("plain");

        let rendered = render_tokens_segment(&ctx, &cfg, IconSet::Plain, &p).unwrap();
        assert!(rendered.contains("⚠️"));
        assert!(rendered.contains("150k/200k (75%)"));
    }
}
