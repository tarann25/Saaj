// === src/theme.rs ===
use ratatui::style::Color;

/// Theme constants for the application.
pub struct Theme;

impl Theme {
    pub const BG:           Color = Color::Rgb(10, 10, 12);      // near black
    pub const SURFACE:      Color = Color::Rgb(20, 6, 6);        // dark crimson surface
    pub const SURFACE2:     Color = Color::Rgb(35, 10, 10);      // slightly lighter surface
    pub const BORDER:       Color = Color::Rgb(111, 2, 2);       // #6f0202 — primary brand
    pub const BORDER_DIM:   Color = Color::Rgb(60, 10, 10);      // muted border
    pub const TEXT:         Color = Color::Rgb(240, 220, 220);   // warm white
    pub const MUTED:        Color = Color::Rgb(160, 120, 120);   // muted warm gray
    pub const DIM:          Color = Color::Rgb(80, 50, 50);      // very muted
    pub const ACCENT:       Color = Color::Rgb(111, 2, 2);       // #6f0202
    pub const ACCENT_BRIGHT:Color = Color::Rgb(200, 60, 60);     // hover/highlight crimson
    pub const GREEN:        Color = Color::Rgb(80, 200, 120);    // status ok
    pub const AMBER:        Color = Color::Rgb(210, 160, 40);    // warnings / repeat indicator
}
