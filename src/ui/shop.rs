//! Левая панель магазина: дроны (x1/x10/MAX) и апгрейды клика.

use crate::components::drones::{drone_cost, drone_dps, DroneKind};
use crate::resources::game_state::{click_upgrade_cost, click_upgrade_max, CLICK_UPGRADES, GameState};
use crate::systems::economy::{buy_click_upgrade, buy_drones, BuyMode, BuyResult};
use crate::ui::icons::{draw_icon, Icon};
use crate::ui::theme::{col, SHOP_W, TOP_BAR_H};
use crate::ui::widgets::{self, button, section_title, BtnStyle};
use crate::ui::UiAction;
use crate::util::{fmt_num, Rect};
use macroquad::prelude::*;

pub struct Shop {
    /// Текущий режим покупки
    pub mode: BuyMode,
    /// Мигание строки при неудачной покупке
    pub error_flash: [f32; 6],
    /// Флаг последней неудачной покупки (для звука ошибки)
    pub buy_failed: bool,
}

impl Shop {
    pub fn new() -> Shop {
        Shop { mode: BuyMode::One, error_flash: [0.0; 6], buy_failed: false }
    }

    pub fn update(&mut self, dt: f32) {
        for f in self.error_flash.iter_mut() {
            *f = (*f - dt * 2.5).max(0.0);
        }
    }

    /// Сброс флага ошибки после обработки звука.
    pub fn clear_error(&mut self) {
        self.buy_failed = false;
    }

    /// Отрисовка панели магазина. Возвращает (область панели, действия).
    pub fn draw(
        &mut self,
        a: &crate::resources::assets::Assets,
        state: &mut GameState,
    ) -> (Rect, Vec<UiAction>) {
        let mut actions: Vec<UiAction> = Vec::new();
        let h_avail = screen_height() - TOP_BAR_H - crate::ui::theme::BOTTOM_BAR_H;
        let px = 10.0;
        let py = TOP_BAR_H + 10.0;
        let ph = h_avail - 20.0;
        widgets::panel_shadow(px, py, SHOP_W, ph, 12.0);
        draw_rectangle(px, py, SHOP_W, ph, col::panel());
        draw_rectangle_lines(px, py, SHOP_W, ph, 1.2, col::border());
        draw_rectangle(px, py, 3.0, ph, col::accent_dim());

        let x0 = px + 14.0;
        let w0 = SHOP_W - 28.0;

        // --- Заголовок + режим покупки
        widgets::text_b(a, "АНГАР ДРОНОВ", x0, py + 12.0, 16.0, col::text());
        let modes: [(&str, BuyMode); 3] = [("x1", BuyMode::One), ("x10", BuyMode::Many(10)), ("MAX", BuyMode::Max)];
        let mut mbx = px + SHOP_W - 14.0;
        for (label, m) in modes.iter().rev() {
            let bw = 42.0;
            mbx -= bw + 4.0;
            let active = self.mode == *m;
            let r = Rect::new(mbx, py + 8.0, bw, 22.0);
            if button(a, r, label, None, true, if active { BtnStyle::Accent } else { BtnStyle::Normal }) {
                self.mode = *m;
            }
        }
        draw_line(x0, py + 38.0, x0 + w0, py + 38.0, 1.0, col::border());

        // --- Строки дронов
        let mut y = py + 46.0;
        let row_h = 58.0;
        for (i, &kind) in DroneKind::ALL.iter().enumerate() {
            let lvl = state.drones[kind.idx()];
            let cost = drone_cost(kind, lvl);
            let unlocked = i == 0 || state.drones[i - 1] > 0 || lvl > 0;
            let can_afford = state.ore >= cost;

            if self.error_flash[i] > 0.0 {
                draw_rectangle(px + 6.0, y - 4.0, SHOP_W - 12.0, row_h, Color::new(1.0, 0.2, 0.2, 0.08 * self.error_flash[i]));
            }

            // Иконка дрона
            let ic_col = if unlocked { col::accent() } else { col::text_dim() };
            draw_icon(Self::drone_icon(kind), x0 + 16.0, y + row_h / 2.0 - 6.0, 30.0, ic_col);

            // Имя и уровень
            let name = if unlocked { kind.name() } else { "???" };
            widgets::text_b(a, name, x0 + 40.0, y + 2.0, 13.0, if unlocked { col::text() } else { col::text_dim() });
            widgets::text(
                a,
                &format!("Ур. {}", lvl),
                x0 + 40.0,
                y + 20.0,
                11.0,
                col::text_dim(),
            );
            // DPS строки
            let dps = drone_dps(kind, lvl, state.bonuses().drone_mult);
            let dps_str = if lvl > 0 {
                format!("{} DPS", fmt_num(dps))
            } else {
                kind.desc().chars().take(30).collect::<String>()
            };
            widgets::text(a, &dps_str, x0 + 40.0, y + 34.0, 10.0, if lvl > 0 { col::success() } else { col::text_dim() });

            // Кнопка покупки
            let bw = 92.0;
            let bh = 26.0;
            let br = Rect::new(px + SHOP_W - 14.0 - bw, y + row_h / 2.0 - bh, bw, bh);
            let enabled = unlocked && can_afford;
            let cost_str = if !unlocked { "ЗАКРЫТО" } else { &fmt_num(cost) };
            if button(a, br, &format!("Купить {}", cost_str), None, enabled, BtnStyle::Accent) {
                let res: BuyResult = buy_drones(state, kind, self.mode);
                if res.bought > 0 {
                    actions.push(UiAction::BuyDrone(kind, self.mode)); // только для звука
                } else {
                    self.error_flash[i] = 1.0;
                    self.buy_failed = true;
                }
            }

            y += row_h + 4.0;
        }

        // --- Апгрейды клика
        y += 6.0;
        section_title(a, x0, y, w0, "УЛУЧШЕНИЯ МАНИПУЛЯТОРА");
        y += 30.0;
        let up_h = 44.0;
        for (idx, name) in CLICK_UPGRADES.iter().enumerate() {
            let lvl = state.click_upgrades[idx];
            let max = click_upgrade_max(idx);
            let cost = click_upgrade_cost(idx, lvl);
            let capped = lvl >= max;
            widgets::text_b(a, name, x0, y, 12.0, col::text());
            let bonus_str = match idx {
                0 => format!("+{} базовый урон", lvl * 2),
                1 => format!("+{:.1}% крит-шанс", lvl as f32 * 0.4),
                _ => format!("+{}% крит-урон", lvl * 10),
            };
            widgets::text(a, &bonus_str, x0, y + 16.0, 10.0, col::text_dim());
            let bw = 92.0;
            let br = Rect::new(px + SHOP_W - 14.0 - bw, y - 2.0, bw, 24.0);
            if capped {
                button(a, br, "МАКС", None, false, BtnStyle::Normal);
            } else {
                let enabled = state.ore >= cost;
                if button(a, br, &fmt_num(cost), None, enabled, BtnStyle::Normal) {
                    let res = buy_click_upgrade(state, idx, self.mode);
                    if res.bought == 0 {
                        self.buy_failed = true;
                    } else {
                        actions.push(UiAction::BuyClickUpgrade(idx, self.mode));
                    }
                }
            }
            y += up_h;
        }

        // Подсказка о Ядре
        y += 2.0;
        if y < py + ph - 30.0 {
            widgets::text(
                a,
                "Ядро и престиж - кнопка ракеты сверху",
                x0,
                y + 4.0,
                10.0,
                col::text_dim(),
            );
        }

        (Rect::new(px, py, SHOP_W, ph), actions)
    }

    pub fn drone_icon(kind: DroneKind) -> Icon {
        match kind {
            DroneKind::Probe => Icon::Drill,
            DroneKind::Laser => Icon::Laser,
            DroneKind::Plasma => Icon::Plasma,
            DroneKind::Quantum => Icon::Quantum,
            DroneKind::Singularity => Icon::Singularity,
            DroneKind::Harvester => Icon::Harvester,
        }
    }
}
