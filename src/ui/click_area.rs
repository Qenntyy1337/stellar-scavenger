//! Игровое поле: планета, щит босса, орбитальные дроны, HP-бары, хит-тест кликов.

use crate::components::planet::planet_name;
use crate::resources::assets::Assets;
use crate::resources::game_state::GameState;
use crate::systems::particles_system::draw_ring_lines;
use crate::systems::planet_render::PlanetRenderer;
use crate::ui::icons::{draw_icon, Icon};
use crate::ui::theme::col;
use crate::ui::widgets;
use crate::util::fmt_num;
use macroquad::prelude::*;

/// Импульс выстрела дрона (визуальный луч к планете).
#[derive(Clone, Copy)]
pub struct BeamPulse {
    pub kind: usize, // индекс типа дрона
    pub angle: f32,
    pub t: f32, // 1.0 -> 0.0
}

/// Геометрия игрового поля.
#[derive(Clone, Copy)]
pub struct PlayField {
    pub cx: f32,
    pub cy: f32,
    pub radius: f32,
}

impl PlayField {
    pub fn compute() -> PlayField {
        let w = screen_width();
        let h = screen_height();
        let top = crate::ui::theme::TOP_BAR_H;
        let bottom = h - crate::ui::theme::BOTTOM_BAR_H;
        let left = crate::ui::theme::SHOP_W + 20.0;
        let cx = (left + w) / 2.0;
        let cy = (top + bottom) / 2.0 - 30.0;
        let radius = ((w - left) / 2.0 * 0.30).min((bottom - top) / 2.0 * 0.30).clamp(70.0, 170.0);
        PlayField { cx, cy, radius }
    }

    /// Попадание клика в планету (с небольшим допуском).
    pub fn hit(&self, mx: f32, my: f32) -> bool {
        let dx = mx - self.cx;
        let dy = my - self.cy;
        dx * dx + dy * dy <= (self.radius * 1.12) * (self.radius * 1.12)
    }

    /// Случайная точка на видимой части планеты (для автокликера).
    pub fn random_point(&self) -> (f32, f32) {
        let a = rand::gen_range(0.0, std::f32::consts::TAU);
        let r = self.radius * rand::gen_range(0.15f32, 0.9f32).sqrt();
        (self.cx + a.cos() * r, self.cy + a.sin() * r)
    }
}

/// Сцена планеты: рендер + орбитальные дроны + лучи + HP/щит.
/// hp_disp/hp_ghost/shield_disp — сглаженные значения баров (плавная анимация).
#[allow(clippy::too_many_arguments)]
pub fn draw_scene(
    a: &Assets,
    pr: &PlanetRenderer,
    fx: &crate::systems::particles_system::Fx,
    state: &GameState,
    field: &PlayField,
    pulse: f32,
    beams: &[BeamPulse],
    hover: bool,
    hp_disp: f32,
    hp_ghost: f32,
    shield_disp: f32,
) {
    let t = get_time() as f32;

    // Дыхание планеты + пульс от ударов
    let breath = 1.0 + 0.012 * (t * 1.6).sin();
    let scale = breath + pulse;

    // Свечение-аура под планетой
    let glow_r = field.radius * scale * (1.55 + 0.05 * (t * 2.0).sin());
    for (i, alpha) in [0.05f32, 0.04, 0.03].iter().enumerate() {
        draw_circle(
            field.cx,
            field.cy,
            glow_r + i as f32 * 12.0,
            Color::new(pr_atmo(pr).0, pr_atmo(pr).1, pr_atmo(pr).2, *alpha),
        );
    }

    // Курсор-hover: кольцо-подсветка
    if hover {
        draw_ring_lines(field.cx, field.cy, field.radius * scale * 1.22, 48, 1.5, Color::new(0.4, 0.8, 1.0, 0.35));
    }

    // Планета (рендерер сам рисует кольца)
    pr.draw(field.cx, field.cy, field.radius * scale);

    // --- Щит босса
    let smax = state.shield_max();
    if smax > 0.0 && state.shield_hp > 0.0 {
        let r = field.radius * scale * 1.18;
        // Пузырь
        draw_circle(field.cx, field.cy, r, Color::new(0.35, 0.65, 1.0, 0.10 + 0.04 * (t * 2.4).sin()));
        draw_ring_lines(field.cx, field.cy, r, 48, 2.0, Color::new(0.4, 0.7, 1.0, 0.4 + 0.15 * (t * 2.4).sin()));
        // Вращающиеся гекс-сегменты
        for k in 0..2 {
            let rot = (t * (12.0 + k as f32 * 7.0)) % 360.0;
            draw_poly_lines(field.cx, field.cy, 6, r * (1.06 + k as f32 * 0.07), rot + k as f32 * 30.0, 1.6, Color::new(0.5, 0.8, 1.0, 0.35 - k as f32 * 0.1));
        }
    }

    // --- Орбитальные дроны
    let mut drones_on_screen = 0usize;
    for (ki, &lvl) in state.drones.iter().enumerate() {
        if lvl == 0 {
            continue;
        }
        let count = (lvl.min(10)) as usize;
        let ring_r = field.radius * scale * (1.5 + ki as f32 * 0.16);
        for d in 0..count {
            let base = d as f32 / count as f32 * std::f32::consts::TAU;
            let speed = 0.35 + ki as f32 * 0.08;
            let ang = base + t * speed;
            let dx = ang.cos() * ring_r;
            let dy = ang.sin() * ring_r * 0.55; // орбита — эллипс (перспектива)
            let px = field.cx + dx;
            let py = field.cy + dy;
            let size = 7.0 + ki as f32 * 0.8;
            // Корпус
            let kcol = match ki {
                0 => Color::new(0.75, 0.78, 0.85, 1.0),
                1 => Color::new(0.45, 0.85, 1.0, 1.0),
                2 => Color::new(0.55, 1.0, 0.75, 1.0),
                3 => Color::new(0.95, 0.75, 1.0, 1.0),
                4 => Color::new(1.0, 0.6, 0.5, 1.0),
                _ => Color::new(1.0, 0.85, 0.45, 1.0),
            };
            // Хвост-трейл
            let tx = (ang - 0.25).cos() * ring_r;
            let ty = (ang - 0.25).sin() * ring_r * 0.55;
            draw_line(field.cx + tx, field.cy + ty, px, py, 1.2, Color::new(kcol.r, kcol.g, kcol.b, 0.25));
            draw_poly(px, py, 3, size * 0.5, ang.to_degrees() + 90.0, kcol);
            draw_circle(px, py, size * 0.22, WHITE);
            drones_on_screen += 1;
        }
    }
    let _ = drones_on_screen;

    // --- Лучи дронов (импульсы атаки)
    for b in beams {
        let kind_col = match b.kind {
            0 => Color::new(0.8, 0.8, 0.85, 1.0),
            1 => Color::new(0.4, 0.85, 1.0, 1.0),
            2 => Color::new(0.5, 1.0, 0.7, 1.0),
            3 => Color::new(0.9, 0.7, 1.0, 1.0),
            4 => Color::new(1.0, 0.55, 0.45, 1.0),
            _ => Color::new(1.0, 0.85, 0.4, 1.0),
        };
        let ring_r = field.radius * scale * (1.5 + b.kind as f32 * 0.16);
        let sx = field.cx + b.angle.cos() * ring_r;
        let sy = field.cy + b.angle.sin() * ring_r * 0.55;
        let tx = field.cx + b.angle.cos() * field.radius * scale * 0.9;
        let ty = field.cy + b.angle.sin() * field.radius * scale * 0.9;
        let alpha = b.t.clamp(0.0, 1.0) * 0.8;
        draw_line(sx, sy, tx, ty, 2.0, Color::new(kind_col.r, kind_col.g, kind_col.b, alpha * 0.5));
        draw_circle(tx, ty, 3.5 * b.t, Color::new(kind_col.r, kind_col.g, kind_col.b, alpha));
    }

    // --- Частицы и текст (поверх планеты)
    fx.draw_particles();
    fx.draw_texts(a);

    // --- Подпись планеты
    let label = if state.is_boss() {
        crate::components::planet::boss_name(state.biome, state.planet)
    } else {
        planet_name(state.biome, state.planet)
    };
    widgets::text_center_b(a, &label, field.cx, field.cy - field.radius * scale - 26.0, 16.0, col::text());
    let biome = crate::components::planet::biome(state.biome);
    widgets::text_center(a, biome.desc, field.cx, field.cy - field.radius * scale - 8.0, 10.5, col::text_dim());

    // --- HP-бар планеты (сглаженный, с «призрачным» хвостом урона)
    let bar_w = (field.radius * 3.4).clamp(320.0, 560.0);
    let bar_x = field.cx - bar_w / 2.0;
    let bar_y = field.cy + field.radius * scale + 34.0;

    widgets::text_center_b(
        a,
        &format!("ПРОЧНОСТЬ: {} / {}", fmt_num(state.planet_hp), fmt_num(state.max_hp())),
        field.cx,
        bar_y - 16.0,
        12.5,
        col::text(),
    );
    widgets::progress_bar(bar_x, bar_y, bar_w, 16.0, hp_disp, hp_ghost, col::ore(), Some(Color::new(1.0, 0.95, 0.85, 0.35)));

    // --- Щит-бар (сглаженный)
    if smax > 0.0 && state.shield_hp > 0.0 {
        let sf = shield_disp;
        widgets::text_center_b(
            a,
            &format!("ЩИТ: {} / {}", fmt_num(state.shield_hp), fmt_num(smax)),
            field.cx,
            bar_y + 22.0,
            12.0,
            col::shield(),
        );
        widgets::progress_bar(bar_x, bar_y + 34.0, bar_w, 12.0, sf, 0.0, col::shield(), None);
        draw_icon(Icon::Shield, bar_x - 14.0, bar_y + 40.0, 16.0, col::shield());
    }
}

/// Достать RGB атмосферы из рендерера (для ауры) — публичный доступ.
fn pr_atmo(pr: &PlanetRenderer) -> (f32, f32, f32) {
    pr.atmo_rgb()
}
