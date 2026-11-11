//! Модальное окно престижа и Звёздного Ядра (апгрейды за Тёмную Материю).

use crate::resources::game_state::{CoreUpgrade, GameState};
use crate::systems::economy::{buy_core, do_prestige};
use crate::ui::icons::{draw_icon, Icon};
use crate::ui::theme::col;
use crate::ui::widgets::{self, button, round_rect, round_rect_lines, BtnStyle};
use crate::ui::UiAction;
use crate::util::fmt_num;
use macroquad::prelude::*;

pub struct PrestigeUi {
    /// Подтверждение престижа (двухшаговое)
    pub armed: bool,
}

impl PrestigeUi {
    pub fn new() -> PrestigeUi {
        PrestigeUi { armed: false }
    }

    pub fn draw(
        &mut self,
        a: &crate::resources::assets::Assets,
        state: &mut GameState,
    ) -> Vec<UiAction> {
        let mut actions = Vec::new();
        let (sw, sh) = (screen_width(), screen_height());
        draw_rectangle(0.0, 0.0, sw, sh, col::overlay());

        let w = 540.0f32.min(sw - 40.0);
        let h = 560.0f32.min(sh - 40.0);
        let (x, y) = ((sw - w) / 2.0, (sh - h) / 2.0);

        widgets::panel_shadow(x, y, w, h, 14.0);
        draw_rectangle(x, y, w, h, col::panel());
        round_rect_lines(x, y, w, h, 14.0, 1.4, col::border());
        draw_rectangle(x, y, w, 44.0, col::panel_bright());
        round_rect(x, y, w, 44.0, 14.0, col::panel_bright());
        draw_rectangle(x, y + 30.0, w, 14.0, col::panel_bright());
        draw_line(x, y + 44.0, x + w, y + 44.0, 1.0, col::border());

        widgets::text_b(a, "ЗВЁЗДНОЕ ЯДРО", x + 18.0, y + 14.0, 17.0, col::text());
        if button(a, crate::util::Rect::new(x + w - 42.0, y + 8.0, 30.0, 30.0), "", Some(Icon::Cross), true, BtnStyle::Normal) {
            actions.push(UiAction::CloseModal);
            self.armed = false;
        }

        let mut yy = y + 58.0;
        let sx = x + 20.0;
        let sw = w - 40.0;

        // --- Блок престижа
        widgets::text_b(a, "ПРЕСТИЖ", sx, yy, 13.0, col::text_dim());
        yy += 20.0;
        widgets::text(
            a,
            "Сброс: руда, планеты, дроны и апгрейды клика. Сохраняются: реликвии, Ядро, Тёмная Материя.",
            sx,
            yy,
            11.0,
            col::text_dim(),
        );
        yy += 26.0;

        let gain = state.prestige_gain();
        let need = 1.0e6f64;
        let progress = (state.run_ore_earned / need).clamp(0.0, 1.0);

        // Показатели
        round_rect(sx, yy, sw, 64.0, 8.0, Color::from_rgba(24, 18, 40, 255));
        draw_icon(Icon::DarkMatter, sx + 24.0, yy + 32.0, 30.0, col::dark_matter());
        widgets::text_b(a, &fmt_num(state.dark_matter), sx + 46.0, yy + 12.0, 15.0, col::dark_matter());
        widgets::text(a, "Тёмная материя сейчас", sx + 46.0, yy + 30.0, 10.0, col::text_dim());
        widgets::text_right(a, &format!("Награда: +{} ТМ", fmt_num(gain)), sx + sw - 12.0, yy + 16.0, 14.0, col::success());
        widgets::text_right(a, &format!("Забег: {} / {} руды", fmt_num(state.run_ore_earned), fmt_num(need)), sx + sw - 12.0, yy + 34.0, 10.0, col::text_dim());
        // Мини-прогресс
        draw_rectangle(sx + 10.0, yy + 50.0, sw - 20.0, 5.0, Color::from_rgba(30, 36, 58, 255));
        draw_rectangle(sx + 10.0, yy + 50.0, (sw - 20.0) * progress as f32, 5.0, col::dark_matter());
        yy += 76.0;

        let can = state.prestige_ready();
        if self.armed {
            if button(a, crate::util::Rect::new(sx, yy, sw, 30.0), "ПОДТВЕРДИТЬ: СЖЕЧЬ ВСЁ РАДИ ЯДРА?", Some(Icon::Prestige), can, BtnStyle::Danger) {
                let g = do_prestige(state);
                if g > 0.0 {
                    actions.push(UiAction::DoPrestige);
                }
                self.armed = false;
            }
        } else {
            if button(a, crate::util::Rect::new(sx, yy, sw, 30.0), "Престиж", Some(Icon::Prestige), can, BtnStyle::Accent) {
                self.armed = true;
            }
            if !can {
                widgets::text_center(a, "Нужно 1.0M руды за текущий забег", sx + sw / 2.0, yy + 40.0, 10.0, col::text_dim());
            }
        }
        yy += 46.0;

        // --- Ядро: апгрейды за ТМ
        widgets::text_b(a, "АПГРЕЙДЫ ЯДРА (сохраняются навсегда)", sx, yy, 13.0, col::text_dim());
        yy += 22.0;

        for &up in CoreUpgrade::ALL.iter() {
            let lvl = state.core_upgrades[up.idx()];
            let max = up.max_level();
            let cost = up.cost(lvl);
            let capped = lvl >= max;

            round_rect(sx, yy, sw, 44.0, 8.0, Color::from_rgba(20, 16, 36, 255));
            draw_icon(Icon::DarkMatter, sx + 18.0, yy + 22.0, 22.0, col::dark_matter());
            widgets::text_b(a, up.name(), sx + 36.0, yy + 4.0, 12.0, col::text());
            widgets::text(a, up.desc(), sx + 36.0, yy + 20.0, 10.0, col::text_dim());
            widgets::text(a, &format!("Ур. {}/{}", lvl, max), sx + 36.0, yy + 34.0, 10.0, col::text_dim());

            let bw = 96.0;
            let br = crate::util::Rect::new(sx + sw - bw - 8.0, yy + 9.0, bw, 26.0);
            if capped {
                button(a, br, "МАКС", None, false, BtnStyle::Normal);
            } else {
                let can_afford = state.dark_matter >= cost;
                if button(a, br, &format!("{} ТМ", fmt_num(cost)), None, can_afford, BtnStyle::Admin) {
                    if buy_core(state, up) {
                        actions.push(UiAction::BuyCore(up));
                    }
                }
            }
            yy += 50.0;
        }

        actions
    }
}
