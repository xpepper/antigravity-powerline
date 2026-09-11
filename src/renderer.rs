use crate::config::Style;
use crate::theme::Palette;

pub fn render_segments(segments: &[String], style: Style, palette: &Palette) -> String {
    if segments.is_empty() {
        return String::new();
    }

    match style {
        Style::Minimal => segments.join(palette.sep),
        Style::Plain => segments.join("  |  "),
        Style::Powerline => {
            let sep = format!("{}  {}", palette.dim, palette.reset);
            segments.join(&sep)
        }
        Style::Capsule => segments
            .iter()
            .map(|s| {
                format!(
                    "{}{}{} {}{}",
                    palette.dim, palette.reset, s, palette.dim, palette.reset
                )
            })
            .collect::<Vec<_>>()
            .join(" "),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_empty_segments() {
        let p = Palette::for_theme("plain");
        assert_eq!(render_segments(&[], Style::Minimal, &p), "");
    }

    #[test]
    fn test_render_minimal_style() {
        let p = Palette::for_theme("plain");
        let segments = vec!["A".to_string(), "B".to_string()];
        let out = render_segments(&segments, Style::Minimal, &p);
        assert_eq!(out, "A  |  B");
    }

    #[test]
    fn test_render_capsule_style() {
        let p = Palette::for_theme("plain");
        let segments = vec!["A".to_string()];
        let out = render_segments(&segments, Style::Capsule, &p);
        assert!(out.contains(''));
        assert!(out.contains(''));
    }

    #[test]
    fn test_render_powerline_style() {
        let p = Palette::for_theme("plain");
        let segments = vec!["A".to_string(), "B".to_string()];
        let out = render_segments(&segments, Style::Powerline, &p);
        assert!(out.contains(''));
    }
}
