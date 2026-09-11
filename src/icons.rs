#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IconSet {
    #[default]
    Plain,
    Nerd,
    Emoji,
}

impl std::str::FromStr for IconSet {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "plain" => Ok(IconSet::Plain),
            "nerd" => Ok(IconSet::Nerd),
            "emoji" => Ok(IconSet::Emoji),
            other => Err(format!("Unknown icon set: {other}")),
        }
    }
}

pub fn model_icon(icon_set: IconSet, custom: Option<&str>) -> &str {
    if let Some(c) = custom {
        return c;
    }
    match icon_set {
        IconSet::Plain => "",
        IconSet::Nerd => "󰚩",
        IconSet::Emoji => "🤖",
    }
}

pub fn git_icon(icon_set: IconSet, custom: Option<&str>) -> &str {
    if let Some(c) = custom {
        return c;
    }
    match icon_set {
        IconSet::Plain => "",
        IconSet::Nerd => "󰘬",
        IconSet::Emoji => "🌿",
    }
}

pub fn tokens_icon(icon_set: IconSet, custom: Option<&str>) -> &str {
    if let Some(c) = custom {
        return c;
    }
    match icon_set {
        IconSet::Plain => "Tokens:",
        IconSet::Nerd => "󰮚",
        IconSet::Emoji => "🪙",
    }
}

pub fn quota_icon(icon_set: IconSet, custom: Option<&str>) -> &str {
    if let Some(c) = custom {
        return c;
    }
    match icon_set {
        IconSet::Plain => "Quota:",
        IconSet::Nerd => "󰔛",
        IconSet::Emoji => "⏳",
    }
}

pub fn cache_icon(icon_set: IconSet, custom: Option<&str>) -> &str {
    if let Some(c) = custom {
        return c;
    }
    match icon_set {
        IconSet::Plain => "Cache:",
        IconSet::Nerd => "󰘸",
        IconSet::Emoji => "⚡",
    }
}

pub fn total_tokens_icon(icon_set: IconSet, custom: Option<&str>) -> &str {
    if let Some(c) = custom {
        return c;
    }
    match icon_set {
        IconSet::Plain => "Total:",
        IconSet::Nerd => "󰓅",
        IconSet::Emoji => "📊",
    }
}

pub fn artifacts_icon(icon_set: IconSet, custom: Option<&str>) -> &str {
    if let Some(c) = custom {
        return c;
    }
    match icon_set {
        IconSet::Plain => "Artifacts:",
        IconSet::Nerd => "󰏗",
        IconSet::Emoji => "📦",
    }
}

pub fn pr_icon(icon_set: IconSet, custom: Option<&str>) -> &str {
    if let Some(c) = custom {
        return c;
    }
    match icon_set {
        IconSet::Plain => "PR",
        IconSet::Nerd => "",
        IconSet::Emoji => "🔀",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_set_plain() {
        assert_eq!(model_icon(IconSet::Plain, None), "");
        assert_eq!(git_icon(IconSet::Plain, None), "");
        assert_eq!(tokens_icon(IconSet::Plain, None), "Tokens:");
        assert_eq!(quota_icon(IconSet::Plain, None), "Quota:");
        assert_eq!(cache_icon(IconSet::Plain, None), "Cache:");
        assert_eq!(total_tokens_icon(IconSet::Plain, None), "Total:");
        assert_eq!(artifacts_icon(IconSet::Plain, None), "Artifacts:");
        assert_eq!(pr_icon(IconSet::Plain, None), "PR");
    }

    #[test]
    fn test_icon_set_nerd() {
        assert_eq!(model_icon(IconSet::Nerd, None), "󰚩");
        assert_eq!(git_icon(IconSet::Nerd, None), "󰘬");
        assert_eq!(tokens_icon(IconSet::Nerd, None), "󰮚");
        assert_eq!(quota_icon(IconSet::Nerd, None), "󰔛");
        assert_eq!(pr_icon(IconSet::Nerd, None), "");
    }

    #[test]
    fn test_icon_set_emoji() {
        assert_eq!(model_icon(IconSet::Emoji, None), "🤖");
        assert_eq!(git_icon(IconSet::Emoji, None), "🌿");
        assert_eq!(cache_icon(IconSet::Emoji, None), "⚡");
        assert_eq!(pr_icon(IconSet::Emoji, None), "🔀");
    }

    #[test]
    fn test_custom_override() {
        assert_eq!(tokens_icon(IconSet::Plain, Some("Ctx:")), "Ctx:");
        assert_eq!(model_icon(IconSet::Nerd, Some("MDL")), "MDL");
        assert_eq!(pr_icon(IconSet::Plain, Some("Pull:")), "Pull:");
    }
}
