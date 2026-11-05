//! Thèmes Nexus Studio v2.0 — palettes VS Code-like
//!
//! 4 thèmes réels, tous testés : NexusDark (défaut), Nord, Gruvbox, Light.

use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemeName {
    NexusDark,
    Nord,
    Gruvbox,
    Light,
}

impl ThemeName {
    pub fn all() -> &'static [(&'static str, ThemeName)] {
        &[
            ("nexus-dark", ThemeName::NexusDark),
            ("nord", ThemeName::Nord),
            ("gruvbox", ThemeName::Gruvbox),
            ("light", ThemeName::Light),
        ]
    }

    pub fn from_str(s: &str) -> Option<ThemeName> {
        let s = s.to_lowercase();
        Self::all()
            .iter()
            .find(|(name, _)| *name == s)
            .map(|(_, t)| *t)
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ThemeName::NexusDark => "Nexus Dark",
            ThemeName::Nord => "Nord",
            ThemeName::Gruvbox => "Gruvbox",
            ThemeName::Light => "Light",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub name: ThemeName,
    // Fonds
    pub bg: Color,
    pub bg_sidebar: Color,
    pub bg_panel: Color,
    pub bg_status: Color,
    pub bg_status_insert: Color,
    pub bg_tab_active: Color,
    pub bg_tab_inactive: Color,
    pub bg_selection: Color,
    pub bg_line_current: Color,
    // Premier plan
    pub fg: Color,
    pub fg_dim: Color,
    pub fg_line_numbers: Color,
    pub fg_line_current_number: Color,
    // Accents
    pub accent: Color,
    pub border: Color,
    pub border_focus: Color,
    // Syntaxe
    pub syn_keyword: Color,
    pub syn_string: Color,
    pub syn_comment: Color,
    pub syn_number: Color,
    pub syn_function: Color,
    pub syn_type: Color,
    pub syn_operator: Color,
    pub syn_punctuation: Color,
    // Verdicts
    pub authorized: Color,
    pub zeropath: Color,
}

const fn c(hex: u32) -> Color {
    Color::Rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

pub const NEXUS_DARK: Theme = Theme {
    name: ThemeName::NexusDark,
    bg: c(0x1e1e1e),
    bg_sidebar: c(0x252526),
    bg_panel: c(0x1b1b1b),
    bg_status: c(0x007acc),
    bg_status_insert: c(0x2ea043),
    bg_tab_active: c(0x1e1e1e),
    bg_tab_inactive: c(0x2d2d2d),
    bg_selection: c(0x264f78),
    bg_line_current: c(0x2a2a2a),
    fg: c(0xd4d4d4),
    fg_dim: c(0x858585),
    fg_line_numbers: c(0x5a5a5a),
    fg_line_current_number: c(0xc6c6c6),
    accent: c(0x0098ff),
    border: c(0x3c3c3c),
    border_focus: c(0x0098ff),
    syn_keyword: c(0x569cd6),
    syn_string: c(0xce9178),
    syn_comment: c(0x6a9955),
    syn_number: c(0xb5cea8),
    syn_function: c(0xdcdcaa),
    syn_type: c(0x4ec9b0),
    syn_operator: c(0xd4d4d4),
    syn_punctuation: c(0x808080),
    authorized: c(0x4ec9b0),
    zeropath: c(0xf44747),
};

pub const NORD: Theme = Theme {
    name: ThemeName::Nord,
    bg: c(0x2e3440),
    bg_sidebar: c(0x3b4252),
    bg_panel: c(0x2b303b),
    bg_status: c(0x5e81ac),
    bg_status_insert: c(0xa3be8c),
    bg_tab_active: c(0x2e3440),
    bg_tab_inactive: c(0x434c5e),
    bg_selection: c(0x434c5e),
    bg_line_current: c(0x3b4252),
    fg: c(0xd8dee9),
    fg_dim: c(0x7b88a1),
    fg_line_numbers: c(0x4c566a),
    fg_line_current_number: c(0xeceff4),
    accent: c(0x88c0d0),
    border: c(0x434c5e),
    border_focus: c(0x88c0d0),
    syn_keyword: c(0x81a1c1),
    syn_string: c(0xa3be8c),
    syn_comment: c(0x616e88),
    syn_number: c(0xb48ead),
    syn_function: c(0x88c0d0),
    syn_type: c(0x8fbcbb),
    syn_operator: c(0xd8dee9),
    syn_punctuation: c(0x7b88a1),
    authorized: c(0xa3be8c),
    zeropath: c(0xbf616a),
};

pub const GRUVBOX: Theme = Theme {
    name: ThemeName::Gruvbox,
    bg: c(0x282828),
    bg_sidebar: c(0x32302f),
    bg_panel: c(0x1d2021),
    bg_status: c(0x458588),
    bg_status_insert: c(0x98971a),
    bg_tab_active: c(0x282828),
    bg_tab_inactive: c(0x3c3836),
    bg_selection: c(0x504945),
    bg_line_current: c(0x3c3836),
    fg: c(0xebdbb2),
    fg_dim: c(0x928374),
    fg_line_numbers: c(0x665c54),
    fg_line_current_number: c(0xfbf1c7),
    accent: c(0x83a598),
    border: c(0x504945),
    border_focus: c(0x83a598),
    syn_keyword: c(0xfb4934),
    syn_string: c(0xb8bb26),
    syn_comment: c(0x928374),
    syn_number: c(0xd3869b),
    syn_function: c(0xfabd2f),
    syn_type: c(0x83a598),
    syn_operator: c(0xebdbb2),
    syn_punctuation: c(0x928374),
    authorized: c(0xb8bb26),
    zeropath: c(0xfb4934),
};

pub const LIGHT: Theme = Theme {
    name: ThemeName::Light,
    bg: c(0xffffff),
    bg_sidebar: c(0xf3f3f3),
    bg_panel: c(0xf8f8f8),
    bg_status: c(0x007acc),
    bg_status_insert: c(0x2ea043),
    bg_tab_active: c(0xffffff),
    bg_tab_inactive: c(0xececec),
    bg_selection: c(0xadd6ff),
    bg_line_current: c(0xf0f0f0),
    fg: c(0x3b3b3b),
    fg_dim: c(0x6f6f6f),
    fg_line_numbers: c(0xa0a0a0),
    fg_line_current_number: c(0x1a1a1a),
    accent: c(0x005fb8),
    border: c(0xd4d4d4),
    border_focus: c(0x005fb8),
    syn_keyword: c(0x0000ff),
    syn_string: c(0xa31515),
    syn_comment: c(0x008000),
    syn_number: c(0x098658),
    syn_function: c(0x795e26),
    syn_type: c(0x267f99),
    syn_operator: c(0x3b3b3b),
    syn_punctuation: c(0x6f6f6f),
    authorized: c(0x107c41),
    zeropath: c(0xd13438),
};

impl Theme {
    pub fn get(name: ThemeName) -> Theme {
        match name {
            ThemeName::NexusDark => NEXUS_DARK,
            ThemeName::Nord => NORD,
            ThemeName::Gruvbox => GRUVBOX,
            ThemeName::Light => LIGHT,
        }
    }

    pub fn next(name: ThemeName) -> ThemeName {
        match name {
            ThemeName::NexusDark => ThemeName::Nord,
            ThemeName::Nord => ThemeName::Gruvbox,
            ThemeName::Gruvbox => ThemeName::Light,
            ThemeName::Light => ThemeName::NexusDark,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_names() {
        assert_eq!(ThemeName::from_str("nord"), Some(ThemeName::Nord));
        assert_eq!(ThemeName::from_str("GRUVBOX"), Some(ThemeName::Gruvbox));
        assert_eq!(ThemeName::from_str("inconnu"), None);
        assert_eq!(ThemeName::all().len(), 4);
    }

    #[test]
    fn test_theme_cycle() {
        assert_eq!(Theme::next(ThemeName::NexusDark), ThemeName::Nord);
        assert_eq!(Theme::next(ThemeName::Light), ThemeName::NexusDark);
    }
}
