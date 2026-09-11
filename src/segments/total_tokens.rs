use crate::config::{IconSet, TotalTokensConfig};
use crate::icons::total_tokens_icon;
use crate::input::ContextWindowInfo;
use crate::segments::tokens::format_tokens;
use crate::theme::Palette;

pub fn render_total_tokens_segment(
    ctx: &ContextWindowInfo,
    config: &TotalTokensConfig,
    icon_set: IconSet,
    palette: &Palette,
) -> Option<String> {
    if !config.enabled {
        return None;
    }

    let total = ctx.total_turn_tokens();
    if total == 0 && config.hide_when_zero {
        return None;
    }

    let formatted = format_tokens(Some(total));
    let icon = total_tokens_icon(icon_set, config.prefix.as_deref());
    let icon_part = if icon.is_empty() {
        String::new()
    } else {
        format!("{}{}{} ", palette.label, icon, palette.reset)
    };

    Some(format!(
        "{}{}{}{}",
        icon_part, palette.total_tokens, formatted, palette.reset
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_total_tokens_zero_hidden() {
        let ctx = ContextWindowInfo::default();
        let cfg = TotalTokensConfig::default();
        let p = Palette::for_theme("plain");
        assert!(render_total_tokens_segment(&ctx, &cfg, IconSet::Plain, &p).is_none());
    }

    #[test]
    fn test_render_total_tokens_active() {
        let ctx = ContextWindowInfo {
            total_input_tokens: Some(47_427),
            total_output_tokens: Some(4_713),
            ..Default::default()
        };
        let cfg = TotalTokensConfig::default();
        let p = Palette::for_theme("plain");

        let rendered = render_total_tokens_segment(&ctx, &cfg, IconSet::Plain, &p).unwrap();
        assert_eq!(rendered, "Total: 52k");
    }
}
