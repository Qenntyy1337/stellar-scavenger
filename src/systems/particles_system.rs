//! Системы частиц и всплывающего текста: обновление и отрисовка.

use crate::components::particles::{FloatingText, Particle, ParticleKind};
use crate::resources::assets::Assets;
use macroquad::prelude::*;

const MAX_PARTICLES: usize = 700;
const MAX_TEXTS: usize = 90;

pub struct Fx {
    pub particles: Vec<Particle>,
    pub texts: Vec<FloatingText>,
    /// Тряска экрана (амплитуда, затухает)
    pub shake: f32,
    pub shake_x: f32,
    pub shake_y: f32,
}

impl Fx {
    pub fn new() -> Fx {
        Fx { particles: Vec::new(), texts: Vec::new(), shake: 0.0, shake_x: 0.0, shake_y: 0.0 }
    }

    /// Тряска: amplitude добавляется (в пикселях).
    pub fn add_shake(&mut self, amp: f32) {
        self.shake = (self.shake + amp).min(26.0);
    }

    /// Взрыв-очередь частиц (круговой разлёт).
    #[allow(clippy::too_many_arguments)]
    pub fn burst(
        &mut self,
        x: f32,
        y: f32,
        count: usize,
        speed: f32,
        size: f32,
        col0: Color,
        col1: Color,
        kind: ParticleKind,
        life: f32,
    ) {
        if self.particles.len() >= MAX_PARTICLES {
            return;
        }
        for _ in 0..count.min(MAX_PARTICLES - self.particles.len()) {
            let a = rand::gen_range(0.0, std::f32::consts::TAU);
            let sp = speed * rand::gen_range(0.35, 1.0);
            self.particles.push(Particle {
                x,
                y,
                vx: a.cos() * sp,
                vy: a.sin() * sp,
                drag: 2.2,
                life,
                max_life: life,
                size0: size * rand::gen_range(0.6, 1.3),
                size1: 0.0,
                col0,
                col1,
                kind,
            });
        }
    }

    /// Направленный конус искр (например, вверх из точки удара).
    #[allow(clippy::too_many_arguments)]
    pub fn cone(
        &mut self,
        x: f32,
        y: f32,
        dir: f32,
        spread: f32,
        count: usize,
        speed: f32,
        col0: Color,
        col1: Color,
    ) {
        if self.particles.len() >= MAX_PARTICLES {
            return;
        }
        for _ in 0..count.min(MAX_PARTICLES - self.particles.len()) {
            let a = dir + rand::gen_range(-spread, spread);
            let sp = speed * rand::gen_range(0.4, 1.1);
            self.particles.push(Particle {
                x,
                y,
                vx: a.cos() * sp,
                vy: a.sin() * sp,
                drag: 1.6,
                life: rand::gen_range(0.25, 0.6),
                max_life: 0.6,
                size0: rand::gen_range(1.5, 3.5),
                size1: 0.0,
                col0,
                col1,
                kind: if rand::gen_range(0.0, 1.0) < 0.4 {
                    ParticleKind::Streak
                } else {
                    ParticleKind::Spark
                },
            });
        }
    }

    /// Ударная волна — расширяющееся кольцо.
    pub fn ring(&mut self, x: f32, y: f32, size0: f32, col: Color) {
        self.particles.push(Particle {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            drag: 0.0,
            life: 0.55,
            max_life: 0.55,
            size0,
            size1: size0 + 130.0,
            col0: col,
            col1: Color::new(col.r, col.g, col.b, 0.0),
            kind: ParticleKind::Ring,
        });
    }

    /// Всплывающий текст (урон, руда, события).
    pub fn push_text(&mut self, x: f32, y: f32, text: String, color: Color, size: f32, crit: bool) {
        if self.texts.len() >= MAX_TEXTS {
            self.texts.remove(0);
        }
        self.texts.push(FloatingText {
            x,
            y,
            vx: rand::gen_range(-14.0, 14.0),
            vy: -55.0 - rand::gen_range(0.0, 30.0),
            text,
            life: if crit { 1.15 } else { 0.9 },
            max_life: if crit { 1.15 } else { 0.9 },
            size,
            color,
            age: 0.0,
        });
    }

    pub fn update(&mut self, dt: f32) {
        for p in self.particles.iter_mut() {
            p.life -= dt;
            let drag_k = (1.0 - p.drag * dt).max(0.0);
            p.vx *= drag_k;
            p.vy *= drag_k;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
        }
        self.particles.retain(|p| p.life > 0.0);

        for t in self.texts.iter_mut() {
            t.life -= dt;
            t.age += dt;
            t.x += t.vx * dt;
            t.y += t.vy * dt;
            t.vy += 42.0 * dt; // лёгкая гравитация вверх-compensated
        }
        self.texts.retain(|t| t.life > 0.0);

        // Тряска: экспоненциальное затухание + случайный вектор
        self.shake *= (1.0 - 9.0 * dt).max(0.0);
        if self.shake > 0.02 {
            self.shake_x = rand::gen_range(-self.shake, self.shake);
            self.shake_y = rand::gen_range(-self.shake, self.shake);
        } else {
            self.shake_x = 0.0;
            self.shake_y = 0.0;
        }
    }

    /// Отрисовка частиц (в мировом слое, со смещением тряски уже применённым).
    pub fn draw_particles(&self) {
        for p in &self.particles {
            let t = (p.life / p.max_life).clamp(0.0, 1.0);
            let col = Color::new(
                p.col0.r + (p.col1.r - p.col0.r) * (1.0 - t),
                p.col0.g + (p.col1.g - p.col0.g) * (1.0 - t),
                p.col0.b + (p.col1.b - p.col0.b) * (1.0 - t),
                p.col0.a + (p.col1.a - p.col0.a) * (1.0 - t),
            );
            match p.kind {
                ParticleKind::Ring => {
                    let r = p.size0 + (p.size1 - p.size0) * (1.0 - t);
                    draw_ring_lines(p.x, p.y, r, 40, 2.5, col);
                }
                ParticleKind::Streak => {
                    let len = 3.5 + 6.0 * (1.0 - t);
                    let inv = 1.0 / (p.vx * p.vx + p.vy * p.vy + 1e-6).sqrt();
                    let (nx, ny) = (p.vx * inv, p.vy * inv);
                    draw_line(
                        p.x - nx * len,
                        p.y - ny * len,
                        p.x + nx * len,
                        p.y + ny * len,
                        p.size0.max(1.2),
                        col,
                    );
                }
                ParticleKind::Spark => {
                    let s = p.size0 + (p.size1 - p.size0) * (1.0 - t);
                    draw_circle(p.x, p.y, s.max(0.4), col);
                }
            }
        }
    }

    /// Отрисовка всплывающего текста.
    pub fn draw_texts(&self, assets: &Assets) {
        for t in &self.texts {
            let tt = (t.life / t.max_life).clamp(0.0, 1.0);
            let alpha = if t.life < 0.25 { tt / 0.25 } else { 1.0 };
            // Появление с упругим масштабом
            let pop = crate::util::ease_out_back((t.age / 0.14).min(1.0));
            let size = t.size * (0.6 + 0.4 * pop);
            let col = Color::new(t.color.r, t.color.g, t.color.b, t.color.a * alpha);
            // Тень для читаемости
            let dims = measure_text(&t.text, Some(&assets.font_bold), size as u16, 1.0);
            let cx = t.x - dims.width / 2.0;
            let cy = t.y - dims.height / 2.0;
            let shadow = Color::new(0.0, 0.0, 0.0, 0.75 * alpha);
            draw_text_ex(
                &t.text,
                cx + 1.5,
                cy + 1.5,
                TextParams {
                    font: Some(&assets.font_bold),
                    font_size: size as u16,
                    color: shadow,
                    ..Default::default()
                },
            );
            draw_text_ex(
                &t.text,
                cx,
                cy,
                TextParams {
                    font: Some(&assets.font_bold),
                    font_size: size as u16,
                    color: col,
                    ..Default::default()
                },
            );
        }
    }
}

/// Окружность линией через полигональный контур (без draw_circle_lines —
/// его сигнатура плавает между версиями macroquad).
pub fn draw_ring_lines(x: f32, y: f32, r: f32, sides: usize, thickness: f32, color: Color) {
    let sides = sides.min(255) as u8;
    draw_poly_lines(x, y, sides, r, 0.0, thickness, color);
}
