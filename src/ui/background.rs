//! Фон: параллакс-звёзды, туманности и виньетка — всё процедурное.

use macroquad::prelude::*;

#[derive(Clone, Copy)]
struct Star {
    x: f32, // 0..1 (относительно экрана)
    y: f32,
    size: f32,
    speed: f32,
    phase: f32,
    layer: u8, // 0 дальний, 2 ближний
    warm: bool,
}

pub struct Background {
    stars: Vec<Star>,
    nebula: Texture2D,
    vignette: Texture2D,
    t: f32,
}

impl Background {
    pub fn new() -> Background {
        // Туманность: несколько цветных гауссовых пятен + шум-дымка
        let size = 256usize;
        let mut img = Image::gen_image_color(size as u16, size as u16, BLANK);
        let blobs: [(f32, f32, f32, [f32; 3]); 5] = [
            (0.25, 0.30, 0.16, [0.45, 0.25, 0.75]),
            (0.70, 0.25, 0.12, [0.20, 0.45, 0.80]),
            (0.55, 0.70, 0.15, [0.15, 0.55, 0.60]),
            (0.85, 0.60, 0.10, [0.60, 0.25, 0.55]),
            (0.15, 0.80, 0.10, [0.25, 0.30, 0.70]),
        ];
        {
            let px = img.bytes.as_mut_slice();
            for y in 0..size {
                for x in 0..size {
                    let (fx, fy) = (x as f32 / size as f32, y as f32 / size as f32);
                    let mut r = 0.0f32;
                    let mut g = 0.0f32;
                    let mut b = 0.0f32;
                    for &(bx, by, br, col) in blobs.iter() {
                        let dx = fx - bx;
                        let dy = fy - by;
                        let d2 = dx * dx + dy * dy;
                        let v = (-d2 / (br * br)).exp();
                        r += col[0] * v;
                        g += col[1] * v;
                        b += col[2] * v;
                    }
                    let i = (y * size + x) * 4;
                    px[i] = (r.min(1.0) * 255.0) as u8;
                    px[i + 1] = (g.min(1.0) * 255.0) as u8;
                    px[i + 2] = (b.min(1.0) * 255.0) as u8;
                    px[i + 3] = ((r.max(g).max(b)).min(1.0) * 150.0) as u8;
                }
            }
        }
        let nebula = Texture2D::from_image(&img);

        // Виньетка: затемнение краёв
        let mut vig = Image::gen_image_color(size as u16, size as u16, BLANK);
        {
            let px = vig.bytes.as_mut_slice();
            for y in 0..size {
                for x in 0..size {
                    let (fx, fy) = (x as f32 / size as f32 - 0.5, y as f32 / size as f32 - 0.5);
                    let d = (fx * fx * 1.4 + fy * fy).sqrt() * 2.0;
                    let dark = (d - 0.55).clamp(0.0, 1.0).powf(1.6) * 0.55;
                    let i = (y * size + x) * 4;
                    px[i] = 0;
                    px[i + 1] = 0;
                    px[i + 2] = 4;
                    px[i + 3] = (dark * 255.0) as u8;
                }
            }
        }
        let vignette = Texture2D::from_image(&vig);

        // Звёзды трёх слоёв параллакса
        let mut stars = Vec::new();
        for i in 0..230 {
            let layer = if i < 130 { 0u8 } else if i < 195 { 1 } else { 2 };
            stars.push(Star {
                x: rand::gen_range(0.0, 1.0),
                y: rand::gen_range(0.0, 1.0),
                size: match layer {
                    0 => rand::gen_range(0.7, 1.3),
                    1 => rand::gen_range(1.2, 2.0),
                    _ => rand::gen_range(1.8, 2.8),
                },
                speed: match layer {
                    0 => rand::gen_range(1.5, 3.0),
                    1 => rand::gen_range(4.0, 7.0),
                    _ => rand::gen_range(8.0, 13.0),
                },
                phase: rand::gen_range(0.0, std::f32::consts::TAU),
                layer,
                warm: rand::gen_range(0.0, 1.0) < 0.25,
            });
        }
        Background { stars, nebula, vignette, t: 0.0 }
    }

    pub fn update(&mut self, dt: f32, anim_speed: f32) {
        self.t += dt * anim_speed.max(0.0);
    }

    /// Рисует фон на весь экран.
    pub fn draw(&mut self, w: f32, h: f32) {
        clear_background(Color::from_rgba(7, 9, 18, 255));
        // Туманность медленно дышит
        let pulse = 0.9 + 0.1 * (self.t * 0.15).sin();
        draw_texture_ex(
            &self.nebula,
            -w * 0.05,
            -h * 0.08,
            Color::new(1.0, 1.0, 1.0, 0.5 * pulse as f32),
            DrawTextureParams {
                dest_size: Some(Vec2::new(w * 1.1, h * 1.16)),
                ..Default::default()
            },
        );
        // Звёзды: медленный дрейф вниз-влево + мерцание
        for s in &self.stars {
            let drift = self.t * s.speed * 0.004;
            let mut x = (s.x - drift * 0.35).rem_euclid(1.0);
            let mut y = (s.y + drift).rem_euclid(1.0);
            x *= w;
            y *= h;
            let tw = 0.55 + 0.45 * (self.t * s.speed + s.phase).sin();
            let alpha = match s.layer {
                0 => 0.35 * tw,
                1 => 0.6 * tw,
                _ => 0.9 * tw,
            };
            let col = if s.warm {
                Color::new(1.0, 0.85, 0.65, alpha as f32)
            } else {
                Color::new(0.85, 0.92, 1.0, alpha as f32)
            };
            if s.layer == 2 {
                draw_circle(x, y, s.size * 0.8, Color::new(col.r, col.g, col.b, alpha as f32 * 0.25));
            }
            draw_rectangle(x, y, s.size, s.size, col);
        }
        // Виньетка поверх
        draw_texture_ex(
            &self.vignette,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(w, h)),
                ..Default::default()
            },
        );
    }
}
