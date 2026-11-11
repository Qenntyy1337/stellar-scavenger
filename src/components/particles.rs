//! Частицы и всплывающий текст (данные; логика в systems/particles_system).

use macroquad::prelude::Color;

#[derive(Clone, Copy, PartialEq)]
pub enum ParticleKind {
    /// Круглая искра (клик, взрыв).
    Spark,
    /// Короткий штрих-линия по направлению скорости.
    Streak,
    /// Расширяющееся кольцо (ударная волна).
    Ring,
}

#[derive(Clone, Copy)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    /// Демпфирование скорости (0 = нет сопротивления).
    pub drag: f32,
    pub life: f32,
    pub max_life: f32,
    pub size0: f32,
    pub size1: f32,
    pub col0: Color,
    pub col1: Color,
    pub kind: ParticleKind,
}

#[derive(Clone)]
pub struct FloatingText {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub text: String,
    pub life: f32,
    pub max_life: f32,
    pub size: f32,
    pub color: Color,
    /// Появление с масштабом 0 -> 1 (ease_out_back).
    pub age: f32,
}
