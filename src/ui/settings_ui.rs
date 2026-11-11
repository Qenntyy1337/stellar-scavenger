//! Модальное окно настроек: звук, окно, анимации, автокликер, сброс, статистика.

use crate::resources::settings::Settings;
use crate::ui::icons::Icon;
use crate::ui::theme::col;
use crate::ui::widgets::{self, button, round_rect, round_rect_lines, slider, toggle, BtnStyle};
use crate::ui::UiAction;
use crate::util::fmt_int;
use macroquad::prelude::*;

/// Действия настроек, требующие Game-контекста.
pub enum SettingsAction {
    Ui(UiAction),
    ResetProgress,
}

pub fn draw(
    a: &crate::resources::assets::Assets,
    settings: &mut Settings,
    state: &crate::resources::game_state::GameState,
    reset_armed: &mut bool,
) -> Vec<SettingsAction> {
    let mut actions = Vec::new();
    let (sw, sh) = (screen_width(), screen_height());
    draw_rectangle(0.0, 0.0, sw, sh, col::overlay());

    let w = 470.0f32.min(sw - 40.0);
    let h = 560.0f32.min(sh - 40.0);
    let (x, y) = ((sw - w) / 2.0, (sh - h) / 2.0);

    widgets::panel_shadow(x, y, w, h, 14.0);
    draw_rectangle(x, y, w, h, col::panel());
    round_rect_lines(x, y, w, h, 14.0, 1.4, col::border());
    draw_rectangle(x, y, w, 44.0, col::panel_bright());
    round_rect(x, y, w, 44.0, 14.0, col::panel_bright());
    draw_rectangle(x, y + 30.0, w, 14.0, col::panel_bright());
    draw_line(x, y + 44.0, x + w, y + 44.0, 1.0, col::border());

    widgets::text_b(a, "НАСТРОЙКИ", x + 18.0, y + 14.0, 17.0, col::text());
    if button(a, crate::util::Rect::new(x + w - 42.0, y + 8.0, 30.0, 30.0), "", Some(Icon::Cross), true, BtnStyle::Normal) {
        actions.push(SettingsAction::Ui(UiAction::CloseModal));
    }

    let mut yy = y + 58.0;
    let sx = x + 20.0;
    let sw = w - 40.0;

    // --- Звук
    widgets::text_b(a, "ЗВУК", sx, yy, 13.0, col::text_dim());
    yy += 22.0;
    if let Some(v) = slider(a, sx, yy, sw, 20.0, settings.master_volume, 0.0, 1.0, "Общая громкость", &|v| format!("{:.0}%", v * 100.0)) {
        settings.master_volume = v;
    }
    yy += 42.0;
    if let Some(v) = slider(a, sx, yy, sw, 20.0, settings.music_volume, 0.0, 1.0, "Музыка", &|v| format!("{:.0}%", v * 100.0)) {
        settings.music_volume = v;
    }
    yy += 42.0;
    if let Some(v) = slider(a, sx, yy, sw, 20.0, settings.sfx_volume, 0.0, 1.0, "Эффекты", &|v| format!("{:.0}%", v * 100.0)) {
        settings.sfx_volume = v;
    }
    yy += 48.0;

    // --- Экран и анимации
    widgets::text_b(a, "ЭКРАН И АНИМАЦИИ", sx, yy, 13.0, col::text_dim());
    yy += 24.0;
    if toggle(a, sx, yy, sw * 0.55, 20.0, "Полноэкранный режим", settings.fullscreen) == Some(!settings.fullscreen) {
        settings.fullscreen = !settings.fullscreen;
        actions.push(SettingsAction::Ui(UiAction::ToggleFullscreen));
    }
    yy += 28.0;
    if toggle(a, sx, yy, sw * 0.55, 20.0, "Показывать FPS", settings.show_fps) == Some(!settings.show_fps) {
        settings.show_fps = !settings.show_fps;
    }
    yy += 34.0;
    if let Some(v) = slider(a, sx, yy, sw, 20.0, settings.animation_speed, 0.5, 2.0, "Скорость анимаций", &|v| format!("{:.1}x", v)) {
        settings.animation_speed = v;
    }
    yy += 48.0;

    // --- Разрешение
    widgets::text_b(a, "РАЗРЕШЕНИЕ ОКНА", sx, yy, 13.0, col::text_dim());
    yy += 24.0;
    let resolutions: [(u32, u32); 3] = [(1280, 720), (1600, 900), (1920, 1080)];
    let bw = (sw - 16.0) / 3.0;
    for (i, (rw, rh)) in resolutions.iter().enumerate() {
        let bx = sx + i as f32 * (bw + 8.0);
        let label = format!("{}x{}", rw, rh);
        let active = settings.res_w == *rw && settings.res_h == *rh;
        if button(a, crate::util::Rect::new(bx, yy, bw, 26.0), &label, None, true, if active { BtnStyle::Accent } else { BtnStyle::Normal }) {
            settings.res_w = *rw;
            settings.res_h = *rh;
            if !settings.fullscreen {
                macroquad::window::request_new_screen_size(*rw as f32, *rh as f32);
            }
        }
    }
    yy += 40.0;

    // --- Автокликер
    widgets::text_b(a, "ВСТРОЕННЫЙ АВТОКЛИКЕР", sx, yy, 13.0, col::text_dim());
    yy += 24.0;
    if toggle(a, sx, yy, sw * 0.55, 20.0, "Включён", settings.autoclicker_enabled) == Some(!settings.autoclicker_enabled) {
        settings.autoclicker_enabled = !settings.autoclicker_enabled;
    }
    yy += 32.0;
    if let Some(v) = slider(a, sx, yy, sw, 20.0, settings.autoclicker_rate, 1.0, 20.0, "Кликов в секунду", &|v| format!("{:.0}", v)) {
        settings.autoclicker_rate = v;
    }
    yy += 46.0;

    // --- Данные
    widgets::text_b(a, "ДАННЫЕ", sx, yy, 13.0, col::text_dim());
    yy += 24.0;
    if button(a, crate::util::Rect::new(sx, yy, sw * 0.48, 28.0), "Сохранить сейчас", Some(Icon::Save), true, BtnStyle::Normal) {
        actions.push(SettingsAction::Ui(UiAction::SaveNow));
    }
    let reset_w = sw * 0.48;
    if *reset_armed {
        if button(a, crate::util::Rect::new(sx + sw - reset_w, yy, reset_w, 28.0), "ТОЧНО СБРОСИТЬ?", Some(Icon::Trash), true, BtnStyle::Danger) {
            actions.push(SettingsAction::ResetProgress);
            *reset_armed = false;
        }
    } else if button(a, crate::util::Rect::new(sx + sw - reset_w, yy, reset_w, 28.0), "Сбросить прогресс", Some(Icon::Trash), true, BtnStyle::Danger) {
        *reset_armed = true;
    }
    yy += 40.0;

    // --- Статистика
    let stats = &state.stats;
    let lines = [
        format!("Кликов: {} (авто: {})", fmt_int(stats.clicks), fmt_int(stats.auto_clicks)),
        format!("Суммарный урон: {}", crate::util::fmt_num(stats.total_damage)),
        format!("Планет уничтожено: {} (боссов: {})", stats.planets_destroyed, stats.bosses_killed),
        format!("Лучший крит: {} | Время: {:.0} мин", crate::util::fmt_num(stats.best_crit), stats.playtime / 60.0),
    ];
    round_rect(sx, yy, sw, 74.0, 8.0, Color::from_rgba(18, 24, 44, 255));
    for (i, l) in lines.iter().enumerate() {
        widgets::text(a, l, sx + 10.0, yy + 6.0 + i as f32 * 17.0, 11.0, col::text_dim());
    }

    actions
}
