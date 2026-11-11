//! Верхняя и нижняя панели HUD.

use crate::resources::game_state::GameState;
use crate::resources::settings::Settings;
use crate::systems::particles_system::draw_ring_lines;
use crate::ui::icons::{draw_icon, Icon};
use crate::ui::theme::{col, BOTTOM_BAR_H, TOP_BAR_H};
use crate::ui::widgets::{self, icon_button, round_rect, round_rect_lines};
use crate::ui::UiAction;
use crate::util::{fmt_num, Rect};
use macroquad::prelude::*;

pub struct Hud {
    /// Кэшированные полосы экономии аллокаций — не нужны, всё immediate
    pub admin_hint_t: f32,
}

impl Hud {
    pub fn new() -> Hud {
        Hud { admin_hint_t: 0.0 }
    }

    /// Верхняя панель. Возвращает прямоугольники областей для блокировки кликов.
    pub fn draw_top(&mut self, a: &crate::resources::assets::Assets, state: &GameState) -> (Rect, Vec<UiAction>) {
        let w = screen_width();
        let actions: Vec<UiAction> = Vec::new();
        draw_rectangle(0.0, 0.0, w, TOP_BAR_H, col::panel());
        draw_rectangle(0.0, TOP_BAR_H, w, 2.0, Color::from_rgba(30, 40, 70, 255));
        draw_line(0.0, TOP_BAR_H + 1.0, w, TOP_BAR_H + 1.0, 1.0, col::accent_dim());

        // --- Чипы валют (слева)
        let mut x = 12.0;
        x = self.chip(a, x, Icon::Ore, &fmt_num(state.ore), col::ore(), "Руда");
        if state.dark_matter > 0.0 || state.prestiges > 0 {
            x = self.chip(a, x, Icon::DarkMatter, &fmt_num(state.dark_matter), col::dark_matter(), "Тёмная материя");
        }
        if state.prestiges > 0 {
            let _ = self.chip(a, x, Icon::Prestige, &format!("{}", state.prestiges), col::success(), "Престижи");
        }

        // --- Центр: биом и планета
        let biome = crate::components::planet::biome(state.biome);
        let label = if state.is_boss() {
            format!("БОСС | {} | Планета {}/10", biome.name, state.planet + 1)
        } else {
            format!("{} | Планета {}/10", biome.name, state.planet + 1)
        };
        let size = 14.0;
        let lw = widgets::text_w_b(a, &label, size);
        widgets::text_center_b(a, &label, w / 2.0, (TOP_BAR_H - size) / 2.0, size, col::text());
        // Галактика (бесконечный режим)
        if state.galaxy > 0 {
            let g_label = format!("Галактика +{}", state.galaxy);
            widgets::text_center(a, &g_label, w / 2.0, TOP_BAR_H - 12.0, 10.0, col::dark_matter());
        }
        let _ = lw;

        // --- Кнопки справа
        let bs = 34.0;
        let gap = 6.0;
        let mut bx = w - 12.0 - bs;
        let by = (TOP_BAR_H - bs) / 2.0;
        let settings_click = icon_button(a, bx, by, bs, Icon::Settings, Some("Настройки (Esc)"));
        bx -= bs + gap;
        let prestige_click = icon_button(a, bx, by, bs, Icon::Prestige, Some("Престиж и Ядро"));
        bx -= bs + gap;
        let relics_click = icon_button(a, bx, by, bs, Icon::Relic, Some("Реликвии"));
        bx -= bs + gap;
        let admin_click = icon_button(a, bx, by, bs, Icon::Admin, Some("Админ-панель (F12 / ~)"));

        // Кэп прогресса биома: маленькая полоска из точек под текстом центра
        let dots_w = 120.0;
        let dx0 = w / 2.0 - dots_w / 2.0;
        for i in 0..10 {
            let cx = dx0 + 6.0 + i as f32 * (dots_w - 12.0) / 9.0;
            let cy = TOP_BAR_H - 6.0;
            if i < state.planet {
                draw_circle(cx, cy, 2.2, col::success());
            } else if i == state.planet {
                draw_ring_lines(cx, cy, 3.2, 10, 1.2, col::accent());
            } else if i == 9 {
                draw_circle(cx, cy, 2.6, col::danger());
            } else {
                draw_circle(cx, cy, 2.0, col::border());
            }
        }

        let rect = Rect::new(0.0, 0.0, w, TOP_BAR_H);
        let mut acts = actions;
        if settings_click {
            acts.push(UiAction::OpenSettings);
        }
        if relics_click {
            acts.push(UiAction::OpenRelics);
        }
        if prestige_click {
            acts.push(UiAction::OpenPrestige);
        }
        if admin_click {
            acts.push(UiAction::ToggleAdmin);
        }
        (rect, acts)
    }
    /// Чип валюты; возвращает x после чипа.
    fn chip(&self, a: &crate::resources::assets::Assets, x: f32, icon: Icon, value: &str, tint: Color, name: &str) -> f32 {
        let h = 34.0;
        let y = (TOP_BAR_H - h) / 2.0;
        let iw = h - 10.0;
        let vw = widgets::text_w_b(a, value, 15.0);
        let w = 8.0 + iw + 6.0 + vw + 10.0;
        round_rect(x, y, w, h, h / 2.0, Color::from_rgba(22, 28, 50, 235));
        round_rect_lines(x, y, w, h, h / 2.0, 1.0, Color::new(tint.r, tint.g, tint.b, 0.45));
        draw_icon(icon, x + 8.0 + iw / 2.0, y + h / 2.0, iw * 0.72, tint);
        widgets::text_b(a, value, x + 8.0 + iw + 6.0, y + (h - 15.0 * 0.8) / 2.0 - 1.0, 15.0, col::text());
        let (mx, my) = mouse_position();
        if Rect::new(x, y, w, h).contains(mx, my) {
            widgets::show_tooltip(a, name, mx, y + h + 8.0);
        }
        x + w + 8.0
    }

    /// Нижняя панель. Прямое управление настройками автокликера.
    pub fn draw_bottom(&mut self, a: &crate::resources::assets::Assets, state: &GameState, settings: &mut Settings, fps: f32) -> Rect {
        let w = screen_width();
        let h = BOTTOM_BAR_H;
        let y = screen_height() - h;
        draw_rectangle(0.0, y, w, h, col::panel());
        draw_rectangle(0.0, y - 2.0, w, 2.0, Color::from_rgba(30, 40, 70, 255));

        // --- Встроенный автокликер
        let mut cy = y + 8.0;
        let mut cx = 12.0;
        let on = settings.autoclicker_enabled;
        if widgets::toggle(a, cx, cy, 205.0, 18.0, "Автокликер", on) == Some(!on) {
            settings.autoclicker_enabled = !on;
        }
        cy += 0.0;
        cx += 212.0;
        // Частота
        let rate_label = format!("{:.0}/с", settings.autoclicker_rate);
        widgets::text(a, &rate_label, cx, cy + 1.0, 12.0, col::text_dim());
        cx += widgets::text_w(a, &rate_label, 12.0) + 8.0;
        if let Some(v) = widgets::slider(a, cx, cy - 2.0, 110.0, 16.0, settings.autoclicker_rate, 1.0, 20.0, "", &|v| format!("{:.0}", v)) {
            settings.autoclicker_rate = v;
        }

        // --- Центр: DPS + подсказка про админку
        let dps_label = format!("DPS дронов: {}", fmt_num(state.drones_dps()));
        widgets::text_center(a, &dps_label, w / 2.0, y + 12.0, 12.0, col::text_dim());
        self.admin_hint_t += get_frame_time();
        let hint_alpha = (0.35 + 0.25 * (self.admin_hint_t * 1.5).sin()) as f32;
        widgets::text_center(
            a,
            "F12 или ~ - админ-панель | Ctrl+S - сохранить | Esc - выход",
            w / 2.0,
            y + h - 14.0,
            11.0,
            Color::new(col::text_dim().r, col::text_dim().g, col::text_dim().b, hint_alpha),
        );

        // --- Справа: FPS, mute
        if settings.show_fps {
            let fps_str = format!("FPS {:.0}", fps);
            widgets::text_right(a, &fps_str, w - 90.0, y + 10.0, 12.0, col::success());
        }
        let bs = 30.0;
        let by = y + (h - bs) / 2.0;
        let mute_icon = if settings.muted { Icon::Cross } else { Icon::Sound };
        if icon_button(a, w - 12.0 - bs, by, bs, mute_icon, Some("Звук (M)")) {
            settings.muted = !settings.muted;
        }

        Rect::new(0.0, y, w, h)
    }
}
