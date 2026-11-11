//! Immediate-mode виджеты: кнопки, тумблеры, слайдеры, бары, панели, тултипы.
//! Все рисуются векторно, шрифт — Inter (Regular/Bold).

use crate::resources::assets::Assets;
use crate::ui::icons::{draw_icon, Icon};
use crate::ui::theme::col;
use crate::util::Rect;
use macroquad::prelude::*;

/// Текст обычным шрифтом (x — левый край, y — верх строки).
pub fn text(a: &Assets, s: &str, x: f32, y: f32, size: f32, color: Color) {
    draw_text_ex(
        s,
        x,
        y + size * 0.8,
        TextParams {
            font: Some(&a.font_regular),
            font_size: size as u16,
            color,
            ..Default::default()
        },
    );
}

/// Текст жирным шрифтом.
pub fn text_b(a: &Assets, s: &str, x: f32, y: f32, size: f32, color: Color) {
    draw_text_ex(
        s,
        x,
        y + size * 0.8,
        TextParams {
            font: Some(&a.font_bold),
            font_size: size as u16,
            color,
            ..Default::default()
        },
    );
}

/// Центрированный текст.
pub fn text_center(a: &Assets, s: &str, cx: f32, y: f32, size: f32, color: Color) {
    let w = text_w(a, s, size);
    text(a, s, cx - w / 2.0, y, size, color);
}

/// Жирный центрированный.
pub fn text_center_b(a: &Assets, s: &str, cx: f32, y: f32, size: f32, color: Color) {
    let w = text_w_b(a, s, size);
    text_b(a, s, cx - w / 2.0, y, size, color);
}

/// Текст с выравниванием по правому краю.
pub fn text_right(a: &Assets, s: &str, right: f32, y: f32, size: f32, color: Color) {
    let w = text_w(a, s, size);
    text(a, s, right - w, y, size, color);
}

pub fn text_w(a: &Assets, s: &str, size: f32) -> f32 {
    measure_text(s, Some(&a.font_regular), size as u16, 1.0).width
}

pub fn text_w_b(a: &Assets, s: &str, size: f32) -> f32 {
    measure_text(s, Some(&a.font_bold), size as u16, 1.0).width
}

/// Скруглённый прямоугольник (заливка).
pub fn round_rect(x: f32, y: f32, w: f32, h: f32, r: f32, color: Color) {
    let r = r.min(w / 2.0).min(h / 2.0);
    draw_rectangle(x + r, y, w - 2.0 * r, h, color);
    draw_rectangle(x, y + r, w, h - 2.0 * r, color);
    draw_circle(x + r, y + r, r, color);
    draw_circle(x + w - r, y + r, r, color);
    draw_circle(x + r, y + h - r, r, color);
    draw_circle(x + w - r, y + h - r, r, color);
}

/// Скруглённый прямоугольник (только рамка).
pub fn round_rect_lines(x: f32, y: f32, w: f32, h: f32, r: f32, thickness: f32, color: Color) {
    let r = r.min(w / 2.0).min(h / 2.0);
    draw_rectangle_lines(x + r, y, w - 2.0 * r, h, thickness, color);
    draw_rectangle_lines(x, y + r, w, h - 2.0 * r, thickness, color);
    // Четвертинки-дуги на углах
    corner_arc(x + r, y + r, r, 180.0, color, thickness);
    corner_arc(x + w - r, y + r, r, 270.0, color, thickness);
    corner_arc(x + w - r, y + h - r, r, 0.0, color, thickness);
    corner_arc(x + r, y + h - r, r, 90.0, color, thickness);
}

fn corner_arc(cx: f32, cy: f32, r: f32, start_deg: f32, color: Color, thickness: f32) {
    let n = 7;
    let start = start_deg.to_radians();
    let sweep = std::f32::consts::FRAC_PI_2;
    let mut prev = (cx + start.cos() * r, cy + start.sin() * r);
    for i in 1..=n {
        let a = start + sweep * i as f32 / n as f32;
        let p = (cx + a.cos() * r, cy + a.sin() * r);
        draw_line(prev.0, prev.1, p.0, p.1, thickness, color);
        prev = p;
    }
}

/// Тень под панелью.
pub fn panel_shadow(x: f32, y: f32, w: f32, h: f32, r: f32) {
    round_rect(x + 3.0, y + 4.0, w, h, r, col::shadow());
}

#[derive(Clone, Copy, PartialEq)]
pub enum BtnStyle {
    Normal,
    Accent,
    Danger,
    Admin,
}

/// Универсальная кнопка. true — клик в этом кадре.
pub fn button(
    a: &Assets,
    r: Rect,
    label: &str,
    icon: Option<Icon>,
    enabled: bool,
    style: BtnStyle,
) -> bool {
    let (mx, my) = mouse_position();
    let hover = enabled && r.contains(mx, my);
    let press = hover && is_mouse_button_down(MouseButton::Left);

    let bg = if !enabled {
        col::button_bg_disabled()
    } else if press {
        match style {
            BtnStyle::Normal => col::button_bg(),
            BtnStyle::Accent => Color::from_rgba(40, 150, 190, 255),
            BtnStyle::Danger => Color::from_rgba(150, 50, 60, 255),
            BtnStyle::Admin => Color::from_rgba(150, 80, 40, 255),
        }
    } else if hover {
        col::button_bg_hover()
    } else {
        match style {
            BtnStyle::Normal => col::button_bg(),
            BtnStyle::Accent => Color::from_rgba(28, 110, 140, 255),
            BtnStyle::Danger => Color::from_rgba(120, 40, 50, 255),
            BtnStyle::Admin => Color::from_rgba(120, 60, 30, 255),
        }
    };
    let border = if !enabled {
        col::border()
    } else {
        match style {
            BtnStyle::Normal => {
                if hover { col::border_bright() } else { col::border() }
            }
            BtnStyle::Accent => {
                if hover { col::accent() } else { col::accent_dim() }
            }
            BtnStyle::Danger => col::danger(),
            BtnStyle::Admin => col::admin_accent(),
        }
    };
    let text_col = if enabled { col::text() } else { col::text_dim() };

    panel_shadow(r.x, r.y, r.w, r.h, 7.0);
    round_rect(r.x, r.y, r.w, r.h, 7.0, bg);
    round_rect_lines(r.x, r.y, r.w, r.h, 7.0, 1.2, border);

    let size = (r.h * 0.38).clamp(11.0, 15.0);
    let tw = text_w(a, label, size);
    let iw = if icon.is_some() { size * 1.6 } else { 0.0 };
    let total = tw + iw;
    let mut tx = r.x + (r.w - total) / 2.0;
    let ty = r.y + (r.h - size * 0.8) / 2.0 - 1.0;
    if let Some(ic) = icon {
        draw_icon(ic, tx + size * 0.8, r.y + r.h / 2.0, size * 1.25, text_col);
        tx += iw;
    }
    text(a, label, tx, ty, size, text_col);

    hover && is_mouse_button_released(MouseButton::Left)
}

/// Круглая иконка-кнопка. true — клик.
pub fn icon_button(a: &Assets, x: f32, y: f32, size: f32, icon: Icon, tooltip: Option<&str>) -> bool {
    let r = Rect::new(x, y, size, size);
    let (mx, my) = mouse_position();
    let hover = r.contains(mx, my);
    let press = hover && is_mouse_button_down(MouseButton::Left);
    if hover {
        round_rect(x - 2.0, y - 2.0, size + 4.0, size + 4.0, 7.0, if press { col::panel_bright() } else { col::panel_hover() });
    }
    draw_icon(icon, x + size / 2.0, y + size / 2.0, size * 0.62, if hover { col::text() } else { col::text_dim() });
    if let Some(t) = tooltip {
        if hover {
            show_tooltip(a, t, mx, my + 18.0);
        }
    }
    hover && is_mouse_button_released(MouseButton::Left)
}

/// Тумблер-переключатель. Some(new_value) — состояние сменилось.
pub fn toggle(a: &Assets, x: f32, y: f32, w: f32, h: f32, label: &str, on: bool) -> Option<bool> {
    let r = Rect::new(x, y, w, h);
    let (mx, my) = mouse_position();
    let hover = r.contains(mx, my);
    let size = 14.0;
    text(a, label, x, y + (h - size * 0.8) / 2.0 - 1.0, size, col::text());
    let pill_x = x + text_w(a, label, size) + 10.0;
    let pill_w = w - (pill_x - x);
    let pill_h = h * 0.55;
    let py = y + (h - pill_h) / 2.0;
    round_rect(pill_x, py, pill_w, pill_h, pill_h / 2.0, if on { col::accent_dim() } else { Color::from_rgba(40, 46, 66, 255) });
    round_rect_lines(pill_x, py, pill_w, pill_h, pill_h / 2.0, 1.0, if on { col::accent() } else { col::border() });
    let knob_t = if on { 1.0 } else { 0.0 };
    let kx = pill_x + 3.0 + (pill_w - pill_h - 4.0) * knob_t;
    draw_circle(kx + pill_h / 2.0 - 1.0, py + pill_h / 2.0, pill_h / 2.0 - 2.0, if on { col::accent() } else { col::text_dim() });
    if hover && is_mouse_button_released(MouseButton::Left) {
        return Some(!on);
    }
    None
}

/// Горизонтальный слайдер. Some(new_value) — значение меняется в этом кадре.
pub fn slider(a: &Assets, x: f32, y: f32, w: f32, _h: f32, val: f32, min: f32, max: f32, label: &str, fmt: &dyn Fn(f32) -> String) -> Option<f32> {
    let size = 13.0;
    text(a, label, x, y, size, col::text());
    let val_str = fmt(val);
    text_right(a, &val_str, x + w, y, size, col::accent());
    let ty = y + size + 4.0;
    let th = 6.0;
    let frac = ((val - min) / (max - min)).clamp(0.0, 1.0);
    round_rect(x, ty, w, th, th / 2.0, Color::from_rgba(30, 36, 58, 255));
    round_rect(x, ty, (w * frac).max(th), th, th / 2.0, col::accent_dim());
    round_rect(x, ty, (w * frac).max(th), th, th / 2.0, col::accent_dim());
    draw_rectangle(x, ty, w * frac, th, col::accent());
    // Крутилка
    let kx = x + w * frac;
    let (mx, my) = mouse_position();
    let track = Rect::new(x - 6.0, ty - 6.0, w + 12.0, th + 12.0);
    let dragging = is_mouse_button_down(MouseButton::Left) && track.contains(mx, my);
    let hover = track.contains(mx, my);
    let kr = if dragging { 8.5 } else if hover { 7.5 } else { 6.5 };
    draw_circle(kx, ty + th / 2.0, kr + 2.0, Color::new(0.4, 0.8, 1.0, 0.25));
    draw_circle(kx, ty + th / 2.0, kr, col::accent());
    if dragging {
        let nv = ((mx - x) / w).clamp(0.0, 1.0) * (max - min) + min;
        return Some(nv);
    }
    None
}

/// Прогресс-бар с «призрачным» хвостом.
pub fn progress_bar(x: f32, y: f32, w: f32, h: f32, frac: f32, ghost_frac: f32, fill: Color, ghost: Option<Color>) {
    round_rect(x, y, w, h, h / 2.0, Color::from_rgba(18, 22, 38, 255));
    round_rect_lines(x, y, w, h, h / 2.0, 1.0, col::border());
    let inner = h - 4.0;
    if let Some(g) = ghost {
        if ghost_frac > frac {
            draw_rectangle(x + 2.0, y + 2.0, (w - 4.0) * ghost_frac.clamp(0.0, 1.0), inner, g);
        }
    }
    draw_rectangle(x + 2.0, y + 2.0, (w - 4.0) * frac.clamp(0.0, 1.0), inner, fill);
    // Блик сверху
    if frac > 0.01 {
        draw_rectangle(
            x + 2.0,
            y + 2.0,
            (w - 4.0) * frac.clamp(0.0, 1.0),
            inner * 0.35,
            Color::new(1.0, 1.0, 1.0, 0.12),
        );
    }
}

/// Тултип под курсором.
pub fn show_tooltip(a: &Assets, s: &str, x: f32, y: f32) {
    let size = 12.0;
    let w = text_w(a, s, size) + 14.0;
    let h = size + 9.0;
    let x = (x - w / 2.0).clamp(4.0, screen_width() - w - 4.0);
    let y = y.min(screen_height() - h - 4.0);
    round_rect(x, y, w, h, 5.0, Color::from_rgba(10, 14, 26, 245));
    round_rect_lines(x, y, w, h, 5.0, 1.0, col::border());
    text(a, s, x + 7.0, y + 3.5, size, col::text());
}

/// Заголовок секции с линией.
pub fn section_title(a: &Assets, x: f32, y: f32, w: f32, s: &str) {
    text_b(a, s, x, y, 15.0, col::text_dim());
    draw_line(x, y + 21.0, x + w, y + 21.0, 1.0, col::border());
}
