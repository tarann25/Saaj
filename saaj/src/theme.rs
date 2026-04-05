// === src/theme.rs ===
use ratatui::style::Color;

/// Central theme palette for Saaj, anchored to #b2d8d8 (soft teal).
pub struct Theme;

impl Theme {
    /// Near-black background with a faint teal tint
    pub const BG: Color = Color::Rgb(8, 18, 18);

    /// Primary surface — dark teal, used for panels and containers
    pub const SURFACE: Color = Color::Rgb(14, 32, 32);

    /// Secondary surface — slightly lighter, used for selected rows
    pub const SURFACE2: Color = Color::Rgb(22, 50, 50);

    /// Primary border color — mid teal
    pub const BORDER: Color = Color::Rgb(40, 100, 100);

    /// Dimmed border — subtle panel outlines
    pub const BORDER_DIM: Color = Color::Rgb(20, 55, 55);

    /// Primary text — warm near-white with a teal tint
    pub const TEXT: Color = Color::Rgb(230, 245, 245);

    /// Muted text — for secondary labels and descriptions
    pub const MUTED: Color = Color::Rgb(140, 185, 185);

    /// Very dim text — for indexes, inactive items
    pub const DIM: Color = Color::Rgb(60, 100, 100);

    /// Primary accent — #b2d8d8 exactly, the anchor color
    pub const ACCENT: Color = Color::Rgb(178, 216, 216);

    /// Bright accent — lighter teal for titles, highlights, active elements
    pub const ACCENT_BRIGHT: Color = Color::Rgb(210, 238, 238);

    /// Status green — slightly warm green that harmonises with teal
    pub const GREEN: Color = Color::Rgb(100, 210, 160);

    /// Amber — used for repeat indicator and warnings
    pub const AMBER: Color = Color::Rgb(210, 175, 80);
}
