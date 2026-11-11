//! Панель администратора (вызывается по F12 или ~).
//!
//! Функции:
//! - Выдача ресурсов и Тёмной Материи
//! - Мгновенное уничтожение планеты / слом щита
//! - Телепорт на любой биом / пропуск планет
//! - Спавн реликвий в инвентарь
//! - Режим Бога и Бесконечный урон
//! - Регулировка time scale (скорость времени игры)
//!
//! Состояние админки НЕ сохраняется — это исключительно dev-инструмент.

use crate::components::relics::RELICS;
use crate::resources::game_state::GameState;
use crate::ui::theme::{col, BOTTOM_BAR_H, TOP_BAR_H};
use crate::ui::widgets::{self, button, round_rect, round_rect_lines, slider, BtnStyle};
use crate::util::{fmt_num, Rect};
use macroquad::prelude::*;

/// Действия админки, требующие гейм-контекста.
#[derive(Clone, Copy, PartialEq)]
pub enum AdminAction {
    GiveOre(f64),
    GiveDarkMatter(f64),
    KillPlanet,
    BreakShield,
    TeleportBiome(usize),
    SkipPlanets(u32),
    SpawnRelic(usize),
    SpawnAllRelics,
    ClearRelics,
}

#[derive(Clone, Copy)]
pub struct AdminState {
    pub open: bool,
    pub god: bool,
    pub infinite_damage: bool,
    pub time_scale: f32,
}

impl Default for AdminState {
    fn default() -> Self {
        AdminState { open: false, god: false, infinite_damage: false, time_scale: 1.0 }
    }
}

pub fn toggle(state: &mut AdminState) {
    state.open = !state.open;
}

/// Отрисовка панели. Возвращает (область, действия).
pub fn draw(
    a: &crate::resources::assets::Assets,
    admin: &mut AdminState,
    state: &mut GameState,
) -> (Rect, Vec<AdminAction>) {
    let mut actions = Vec::new();
    let w = 320.0f32.min(screen_width() - 20.0);
    let h = screen_height() - TOP_BAR_H - BOTTOM_BAR_H - 20.0;
    let x = screen_width() - w - 10.0;
    let y = TOP_BAR_H + 10.0;

    widgets::panel_shadow(x, y, w, h, 12.0);
    round_rect(x, y, w, h, 12.0, col::admin_bg());
    round_rect_lines(x, y, w, h, 12.0, 1.4, col::admin_accent());
    round_rect(x, y, w, 40.0, 12.0, Color::from_rgba(46, 26, 20, 250));
    draw_rectangle(x, y + 28.0, w, 12.0, Color::from_rgba(46, 26, 20, 250));
    draw_line(x, y + 40.0, x + w, y + 40.0, 1.0, col::admin_accent());

    widgets::text_b(a, "ADMIN PANEL", x + 14.0, y + 13.0, 15.0, col::admin_accent());
    widgets::text(a, "dev only", x + w - 60.0, y + 14.0, 10.0, col::text_dim());

    let mut yy = y + 52.0;
    let sx = x + 14.0;
    let sw = w - 28.0;

    // --- Режимы
    widgets::text_b(a, "РЕЖИМЫ", sx, yy, 12.0, col::admin_accent());
    yy += 20.0;
    if widgets::toggle(a, sx, yy, sw * 0.5, 18.0, "Режим Бога", admin.god) == Some(!admin.god) {
        admin.god = !admin.god;
    }
    if widgets::toggle(a, sx + sw * 0.52, yy, sw * 0.48, 18.0, "Бескон. урон", admin.infinite_damage) == Some(!admin.infinite_damage) {
        admin.infinite_damage = !admin.infinite_damage;
    }
    yy += 30.0;

    // --- Time scale
    if let Some(v) = slider(a, sx, yy, sw, 18.0, admin.time_scale, 0.0, 5.0, "Time scale", &|v| format!("{:.1}x", v)) {
        admin.time_scale = v;
    }
    yy += 40.0;

    // --- Ресурсы
    widgets::text_b(a, "РЕСУРСЫ", sx, yy, 12.0, col::admin_accent());
    yy += 20.0;
    for (i, amount) in [1.0e3, 1.0e6, 1.0e9, 1.0e12].iter().enumerate() {
        let bw = (sw - 12.0) / 2.0;
        let bx = sx + (i % 2) as f32 * (bw + 12.0);
        let by = yy + (i / 2) as f32 * 30.0;
        if button(a, Rect::new(bx, by, bw, 24.0), &format!("Руда +{}", fmt_num(*amount)), None, true, BtnStyle::Normal) {
            actions.push(AdminAction::GiveOre(*amount));
        }
    }
    yy += 60.0;
    for (i, amount) in [10.0, 100.0, 1000.0].iter().enumerate() {
        let bw = (sw - 16.0) / 3.0;
        let bx = sx + i as f32 * (bw + 8.0);
        if button(a, Rect::new(bx, yy, bw, 24.0), &format!("ТМ +{}", amount), None, true, BtnStyle::Admin) {
            actions.push(AdminAction::GiveDarkMatter(*amount));
        }
    }
    yy += 36.0;

    // --- Бой
    widgets::text_b(a, "БОЙ", sx, yy, 12.0, col::admin_accent());
    yy += 20.0;
    let bw = (sw - 12.0) / 2.0;
    if button(a, Rect::new(sx, yy, bw, 24.0), "Уничтожить планету", None, true, BtnStyle::Danger) {
        actions.push(AdminAction::KillPlanet);
    }
    if button(a, Rect::new(sx + bw + 12.0, yy, bw, 24.0), "Сломать щит", None, true, BtnStyle::Danger) {
        actions.push(AdminAction::BreakShield);
    }
    yy += 32.0;

    // --- Телепорт
    widgets::text_b(a, "ТЕЛЕПОРТ", sx, yy, 12.0, col::admin_accent());
    yy += 20.0;
    let bw = (sw - 7.0 * 7.0) / 8.0;
    for i in 0..8 {
        let bx = sx + i as f32 * (bw + 7.0);
        if button(a, Rect::new(bx, yy, bw, 24.0), &format!("{}", i + 1), None, true, BtnStyle::Normal) {
            actions.push(AdminAction::TeleportBiome(i));
        }
    }
    yy += 30.0;
    let bw = (sw - 12.0) / 2.0;
    if button(a, Rect::new(sx, yy, bw, 24.0), "Планета +1", None, true, BtnStyle::Normal) {
        actions.push(AdminAction::SkipPlanets(1));
    }
    if button(a, Rect::new(sx + bw + 12.0, yy, bw, 24.0), "Планета +10", None, true, BtnStyle::Normal) {
        actions.push(AdminAction::SkipPlanets(10));
    }
    yy += 34.0;

    // --- Реликвии
    widgets::text_b(a, "РЕЛИКВИИ (Shift = +10)", sx, yy, 12.0, col::admin_accent());
    yy += 20.0;
    let cols = 4usize;
    let bw = (sw - 7.0 * (cols as f32 - 1.0)) / cols as f32;
    for (i, def) in RELICS.iter().enumerate() {
        let bx = sx + (i % cols) as f32 * (bw + 7.0);
        let by = yy + (i / cols) as f32 * 26.0;
        let rgb = def.rarity.rgb();
        let rc = macroquad::prelude::Color::from_rgba(rgb[0], rgb[1], rgb[2], 255);
        let lvl = state.relic_level(def.id);
        // Мини-кнопка с уровнем
        round_rect(bx, by, bw, 20.0, 5.0, if lvl > 0 { Color::from_rgba(30, 40, 60, 255) } else { Color::from_rgba(22, 26, 42, 255) });
        round_rect_lines(bx, by, bw, 20.0, 5.0, 1.0, rc);
        widgets::text_center_b(a, &format!("{}", i + 1), bx + bw / 2.0, by + 4.0, 11.0, if lvl > 0 { rc } else { col::text_dim() });
        let (mx, my) = mouse_position();
        if Rect::new(bx, by, bw, 20.0).contains(mx, my) {
            widgets::show_tooltip(a, def.name, mx, by + 24.0);
            if is_mouse_button_released(macroquad::prelude::MouseButton::Left) {
                actions.push(AdminAction::SpawnRelic(i));
            }
        }
    }
    yy += (RELICS.len() as f32 / cols as f32).ceil() * 26.0 + 8.0;
    let bw = (sw - 12.0) / 2.0;
    if button(a, Rect::new(sx, yy, bw, 24.0), "Выдать все", None, true, BtnStyle::Normal) {
        actions.push(AdminAction::SpawnAllRelics);
    }
    if button(a, Rect::new(sx + bw + 12.0, yy, bw, 24.0), "Очистить", None, true, BtnStyle::Danger) {
        actions.push(AdminAction::ClearRelics);
    }
    yy += 34.0;

    // Инфо
    widgets::text(
        a,
        "Урон клика сейчас:",
        sx,
        yy,
        10.0,
        col::text_dim(),
    );
    let dmg = crate::systems::combat::click_damage(state, &state.bonuses(), admin.infinite_damage);
    widgets::text_right(a, &fmt_num(dmg.0), sx + sw, yy, 10.0, col::admin_accent());

    (Rect::new(x, y, w, h), actions)
}
