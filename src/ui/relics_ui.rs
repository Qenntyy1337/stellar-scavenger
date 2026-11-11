//! Модальное окно инвентаря реликвий: сетка карточек 4x4.

use crate::components::relics::RELICS;
use crate::resources::game_state::GameState;
use crate::ui::icons::{draw_icon, Icon};
use crate::ui::theme::col;
use crate::ui::widgets::{self, button, round_rect, round_rect_lines, BtnStyle};
use crate::ui::UiAction;
use macroquad::prelude::*;

pub fn draw(a: &crate::resources::assets::Assets, state: &GameState) -> Vec<UiAction> {
    let mut actions = Vec::new();
    let (sw, sh) = (screen_width(), screen_height());

    // Затемнение
    draw_rectangle(0.0, 0.0, sw, sh, col::overlay());

    let w = 720.0f32.min(sw - 40.0);
    let h = 500.0f32.min(sh - 40.0);
    let (x, y) = ((sw - w) / 2.0, (sh - h) / 2.0);

    widgets::panel_shadow(x, y, w, h, 14.0);
    draw_rectangle(x, y, w, h, col::panel());
    round_rect_lines(x, y, w, h, 14.0, 1.4, col::border());
    draw_rectangle(x, y, w, 44.0, col::panel_bright());
    // Скругление только сверху у шапки — рисуем поверх
    round_rect(x, y, w, 44.0, 14.0, col::panel_bright());
    draw_rectangle(x, y + 30.0, w, 14.0, col::panel_bright());
    draw_line(x, y + 44.0, x + w, y + 44.0, 1.0, col::border());

    widgets::text_b(a, "РЕЛИКВАРИЙ", x + 18.0, y + 14.0, 17.0, col::text());
    let found = state.relics.len();
    widgets::text(
        a,
        &format!("Найдено {}/{} | Дубликаты повышают уровень эффекта", found, RELICS.len()),
        x + 18.0,
        y + 52.0,
        11.0,
        col::text_dim(),
    );

    // Кнопка закрытия
    if button(a, crate::util::Rect::new(x + w - 42.0, y + 8.0, 30.0, 30.0), "", Some(Icon::Cross), true, BtnStyle::Normal) {
        actions.push(UiAction::CloseModal);
    }

    // Сетка 4x4
    let cols = 4usize;
    let rows = 4usize;
    let pad = 10.0;
    let cw = (w - 36.0 - pad * (cols - 1) as f32) / cols as f32;
    let ch = (h - 96.0 - pad * (rows - 1) as f32) / rows as f32;
    for (i, def) in RELICS.iter().enumerate() {
        let (cx_i, cy_i) = (i % cols, i / cols);
        let cx = x + 18.0 + cx_i as f32 * (cw + pad);
        let cy = y + 74.0 + cy_i as f32 * (ch + pad);
        let lvl = state.relic_level(def.id);

        let rgb = def.rarity.rgb();
        let rc = Color::from_rgba(rgb[0], rgb[1], rgb[2], 255);

        // Карточка
        round_rect(cx, cy, cw, ch, 9.0, Color::from_rgba(20, 26, 46, 245));
        round_rect_lines(cx, cy, cw, ch, 9.0, if lvl > 0 { 1.8 } else { 1.0 }, if lvl > 0 { rc } else { col::border() });
        if lvl > 0 {
            // Лёгкое свечение слева
            draw_rectangle(cx, cy + 6.0, 2.5, ch - 12.0, rc);
        }

        // Иконка редкости
        let ic_col = if lvl > 0 { rc } else { col::text_dim() };
        draw_icon(Icon::Relic, cx + 20.0, cy + 22.0, 26.0, ic_col);
        widgets::text_b(a, def.rarity.name(), cx + 38.0, cy + 10.0, 10.0, ic_col);

        // Название
        let name_size = 11.5f32;
        widgets::text_b(a, def.name, cx + 10.0, cy + 40.0, name_size, if lvl > 0 { col::text() } else { col::text_dim() });

        // Описание эффекта
        widgets::text(a, def.desc, cx + 10.0, cy + 55.0, 10.0, col::text_dim());

        // Уровень
        if lvl > 0 {
            widgets::text_b(a, &format!("УР. {}", lvl), cx + 10.0, cy + ch - 10.0, 12.0, rc);
        } else {
            draw_icon(Icon::Lock, cx + 16.0, cy + ch - 14.0, 14.0, col::text_dim());
            widgets::text(a, "не найдена", cx + 28.0, cy + ch - 18.0, 10.0, col::text_dim());
        }
    }

    actions
}
