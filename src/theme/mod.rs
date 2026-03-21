use iced::Color;

// ── Placement ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, PartialEq)]
pub enum Placement {
    #[default]
    Center,
    TopRight,
    TopLeft,
    TopCenter,
    BottomRight,
    BottomLeft,
    BottomCenter,
}

// ── Theme ─────────────────────────────────────────────────────────────────────

/// Full configuration for aplec: colours and placement.
///
/// Lives in `~/.config/aplec/theme.conf` (`key = value` format).
/// Colours are `#RRGGBB` or `#RRGGBBAA`.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    pub placement: Placement,
    /// Gap in pixels between the popup and the nearest screen edge.
    pub margin: i32,
    pub background: Color,
    pub text: Color,
    pub text_dim: Color,
    pub border: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            placement: Placement::default(),
            margin: 10,
            background: hex("#1e1e2eee"),
            text: hex("#cdd6f4"),
            text_dim: hex("#6c7086"),
            border: hex("#45475a"),
        }
    }
}

impl Theme {
    /// Load theme from `~/.config/aplec/theme.conf`, falling back to defaults.
    pub fn load() -> Self {
        config_path("theme.conf")
            .and_then(|p| std::fs::read_to_string(p).ok())
            .map(|c| Self::parse_onto(Self::default(), &c))
            .unwrap_or_default()
    }

    fn parse_onto(mut base: Self, content: &str) -> Self {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, val)) = line.split_once('=') else {
                continue;
            };
            base.apply_key(key.trim(), val.trim().trim_matches('"'));
        }
        base
    }

    pub fn apply_key(&mut self, key: &str, value: &str) {
        match key {
            "placement" => {
                self.placement = match value {
                    "top-right" => Placement::TopRight,
                    "top-left" => Placement::TopLeft,
                    "top-center" => Placement::TopCenter,
                    "bottom-right" => Placement::BottomRight,
                    "bottom-left" => Placement::BottomLeft,
                    "bottom-center" => Placement::BottomCenter,
                    _ => Placement::Center,
                };
                return;
            }
            "margin" => {
                if let Ok(v) = value.parse() {
                    self.margin = v;
                }
                return;
            }
            _ => {}
        }
        let Some(color) = parse_color(value) else {
            return;
        };
        match key {
            "background" => self.background = color,
            "text" => self.text = color,
            "text_dim" => self.text_dim = color,
            "border" => self.border = color,
            _ => {}
        }
    }
}

fn config_path(rel: &str) -> Option<std::path::PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|h| std::path::PathBuf::from(h).join(".config/aplec").join(rel))
}

/// Parse `#RRGGBB` or `#RRGGBBAA` into an iced `Color`.
pub fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim().trim_start_matches('#');
    if s.len() < 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()? as f32 / 255.0;
    let g = u8::from_str_radix(&s[2..4], 16).ok()? as f32 / 255.0;
    let b = u8::from_str_radix(&s[4..6], 16).ok()? as f32 / 255.0;
    let a = if s.len() >= 8 {
        u8::from_str_radix(&s[6..8], 16).ok()? as f32 / 255.0
    } else {
        1.0
    };
    Some(Color { r, g, b, a })
}

fn hex(s: &str) -> Color {
    parse_color(s).expect("invalid hex in Theme::default()")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rrggbb() {
        let c = parse_color("#cba6f7").unwrap();
        assert!((c.r - 0.796).abs() < 0.01);
        assert!((c.g - 0.651).abs() < 0.01);
        assert!((c.b - 0.969).abs() < 0.01);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn parse_rrggbbaa() {
        let c = parse_color("#1e1e2eee").unwrap();
        assert!((c.r - 0.118).abs() < 0.01);
        assert!((c.a - 0.933).abs() < 0.01);
    }

    #[test]
    fn parse_without_hash() {
        assert!(parse_color("cdd6f4").is_some());
    }

    #[test]
    fn parse_too_short_returns_none() {
        assert!(parse_color("#fff").is_none());
    }

    #[test]
    fn apply_colour_key_updates_field() {
        let mut t = Theme::default();
        t.apply_key("border", "#ff0000");
        assert!((t.border.r - 1.0).abs() < 0.01);
        assert_eq!(t.border.g, 0.0);
    }

    #[test]
    fn apply_placement_key() {
        let mut t = Theme::default();
        t.apply_key("placement", "bottom-left");
        assert_eq!(t.placement, Placement::BottomLeft);
    }

    #[test]
    fn apply_margin_key() {
        let mut t = Theme::default();
        t.apply_key("margin", "36");
        assert_eq!(t.margin, 36);
    }

    #[test]
    fn apply_unknown_key_is_noop() {
        let base = Theme::default();
        let mut t = Theme::default();
        t.apply_key("nonexistent", "#ff0000");
        assert_eq!(t, base);
    }

    #[test]
    fn parse_conf_overrides_only_given_keys() {
        let t = Theme::parse_onto(
            Theme::default(),
            "border = #ff0000\n# comment\n\ntext = #aabbcc\n",
        );
        assert!((t.border.r - 1.0).abs() < 0.01);
        assert_eq!(t.background, Theme::default().background);
        assert_eq!(t.placement, Theme::default().placement);
    }
}
