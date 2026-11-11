//! Тема UI: палитра, константы компоновки.

pub const TOP_BAR_H: f32 = 54.0;
pub const BOTTOM_BAR_H: f32 = 46.0;
pub const SHOP_W: f32 = 312.0;

pub mod col {
    use macroquad::prelude::Color;

    pub fn panel() -> Color {
        Color::from_rgba(15, 20, 38, 235)
    }
    pub fn panel_bright() -> Color {
        Color::from_rgba(24, 32, 56, 240)
    }
    pub fn panel_hover() -> Color {
        Color::from_rgba(30, 40, 68, 245)
    }
    pub fn border() -> Color {
        Color::from_rgba(58, 76, 120, 255)
    }
    pub fn border_bright() -> Color {
        Color::from_rgba(90, 120, 180, 255)
    }
    pub fn accent() -> Color {
        Color::from_rgba(64, 208, 255, 255)
    }
    pub fn accent_dim() -> Color {
        Color::from_rgba(64, 208, 255, 70)
    }
    pub fn ore() -> Color {
        Color::from_rgba(255, 184, 77, 255)
    }
    pub fn dark_matter() -> Color {
        Color::from_rgba(186, 112, 255, 255)
    }
    pub fn crit() -> Color {
        Color::from_rgba(255, 92, 92, 255)
    }
    pub fn success() -> Color {
        Color::from_rgba(80, 226, 150, 255)
    }
    pub fn danger() -> Color {
        Color::from_rgba(255, 85, 102, 255)
    }
    pub fn text() -> Color {
        Color::from_rgba(232, 238, 249, 255)
    }
    pub fn text_dim() -> Color {
        Color::from_rgba(138, 148, 176, 255)
    }
    pub fn shield() -> Color {
        Color::from_rgba(96, 170, 255, 255)
    }
    pub fn button_bg() -> Color {
        Color::from_rgba(34, 46, 78, 255)
    }
    pub fn button_bg_hover() -> Color {
        Color::from_rgba(48, 64, 104, 255)
    }
    pub fn button_bg_disabled() -> Color {
        Color::from_rgba(26, 30, 46, 200)
    }
    pub fn shadow() -> Color {
        Color::from_rgba(0, 0, 0, 110)
    }
    pub fn overlay() -> Color {
        Color::from_rgba(4, 6, 12, 190)
    }
    pub fn admin_bg() -> Color {
        Color::from_rgba(20, 12, 30, 242)
    }
    pub fn admin_accent() -> Color {
        Color::from_rgba(255, 120, 60, 255)
    }
}
