//! Процедурный аудио-синтез: все SFX и эмбиент-музыка генерируются кодом
//! в PCM16 WAV прямо в памяти (никаких внешних аудиофайлов).
//!
//! Почему не rodio: macroquad::audio (quad-snd) уже умеет WAV/OGG,
//! не тянет ALSA-костыли в бинарь и работает на всех платформах одинаково.

use crate::resources::assets::SfxBank;
use macroquad::audio::{load_sound_from_bytes, Sound};

const SR: u32 = 22050;

/// Запаковка моно PCM f32-семплов в WAV-байты.
fn wav_bytes(samples: &[f32]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + samples.len() * 2);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes());      // размер fmt-чанка
    out.extend_from_slice(&1u16.to_le_bytes());       // PCM
    out.extend_from_slice(&1u16.to_le_bytes());       // моно
    out.extend_from_slice(&SR.to_le_bytes());         // sample rate
    out.extend_from_slice(&(SR * 2).to_le_bytes());   // byte rate
    out.extend_from_slice(&2u16.to_le_bytes());       // block align
    out.extend_from_slice(&16u16.to_le_bytes());      // бит на сэмпл
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for &s in samples {
        let v = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

async fn decode(bytes: Vec<u8>) -> Option<Sound> {
    load_sound_from_bytes(&bytes).await.ok()
}

/// Секция-строитель: пишет в буфер с проверкой границ.
struct Track {
    buf: Vec<f32>,
}

impl Track {
    fn new(dur: f32) -> Track {
        Track { buf: vec![0.0; (dur * SR as f32) as usize] }
    }

    /// Добавить осциллятор с огибающей ADSR-подобного вида.
    /// f(t) — мгновенная частота, pan игнорируем (моно).
    fn osc(
        &mut self,
        start: f32,
        dur: f32,
        amp: f32,
        attack: f32,
        release: f32,
        wave: Wave,
        freq: impl Fn(f32) -> f32,
    ) {
        let n = self.buf.len();
        let s0 = ((start * SR as f32) as usize).min(n);
        let s1 = (((start + dur) * SR as f32) as usize).min(n);
        if s1 <= s0 {
            return;
        }
        let mut phase = 0.0f32;
        let dt = 1.0 / SR as f32;
        for i in s0..s1 {
            let t = (i - s0) as f32 * dt;
            let f = (freq)(t).max(0.0);
            phase += f * dt;
            let p = phase % 1.0;
            let raw = match wave {
                Wave::Sine => (p * std::f32::consts::TAU).sin(),
                Wave::Square => {
                    if p < 0.5 { 0.6 } else { -0.6 }
                }
                Wave::Saw => 2.0 * p - 1.0,
                Wave::Noise => macroquad::prelude::rand::gen_range(-1.0, 1.0),
            };
            let env = env_at(t, dur, attack, release);
            self.buf[i] += raw * amp * env;
        }
    }

    /// Однополюсный lowpass поверх всего трека (для пэдов и бумов).
    fn lowpass(&mut self, alpha: f32) {
        let mut y = 0.0f32;
        for s in self.buf.iter_mut() {
            y += alpha * (*s - y);
            *s = y;
        }
    }

    fn normalize(&mut self, peak: f32) {
        let m = self.buf.iter().fold(0.0f32, |a, &s| a.max(s.abs()));
        if m > 1e-6 {
            let k = peak / m;
            for s in self.buf.iter_mut() {
                *s *= k;
            }
        }
    }
}

pub enum Wave {
    Sine,
    Square,
    Saw,
    Noise,
}

/// Огибающая: линейная атака + степенная затухающая фаза + релиз.
fn env_at(t: f32, dur: f32, attack: f32, release: f32) -> f32 {
    let a = if t < attack { t / attack.max(1e-4) } else { 1.0 };
    let r = if t > dur - release {
        ((dur - t) / release.max(1e-4)).clamp(0.0, 1.0)
    } else {
        1.0
    };
    // лёгкое экспоненциальное затухание тела звука
    let body = (-t * 2.2 / dur.max(1e-4)).exp().max(0.12);
    a * r * body
}

// ---------- Конкретные звуки -------------------------------------------------

fn make_click() -> Vec<u8> {
    let mut t = Track::new(0.10);
    // Низкий «тычок» с падением высоты + щелчок-шум
    t.osc(0.0, 0.09, 0.55, 0.002, 0.05, Wave::Sine, |t| 170.0 - 900.0 * t);
    t.osc(0.0, 0.02, 0.18, 0.001, 0.015, Wave::Noise, |_| 0.0);
    t.lowpass(0.55);
    t.normalize(0.6);
    wav_bytes(&t.buf)
}

fn make_crit() -> Vec<u8> {
    let mut t = Track::new(0.18);
    t.osc(0.0, 0.16, 0.40, 0.002, 0.08, Wave::Saw, |t| 700.0 + 1400.0 * t);
    t.osc(0.0, 0.16, 0.25, 0.002, 0.08, Wave::Sine, |t| 1400.0 + 1800.0 * t);
    t.osc(0.0, 0.03, 0.15, 0.001, 0.02, Wave::Noise, |_| 0.0);
    t.lowpass(0.35);
    t.normalize(0.62);
    wav_bytes(&t.buf)
}

fn make_buy() -> Vec<u8> {
    let mut t = Track::new(0.30);
    t.osc(0.00, 0.12, 0.35, 0.004, 0.06, Wave::Sine, |_| 660.0);
    t.osc(0.10, 0.18, 0.38, 0.004, 0.10, Wave::Sine, |_| 990.0);
    t.osc(0.10, 0.18, 0.12, 0.004, 0.10, Wave::Sine, |_| 1320.0);
    t.normalize(0.55);
    wav_bytes(&t.buf)
}

fn make_relic() -> Vec<u8> {
    let mut t = Track::new(0.62);
    let notes = [880.0f32, 1174.66, 1567.98];
    for (i, &f) in notes.iter().enumerate() {
        let st = 0.06 * i as f32;
        t.osc(st, 0.35, 0.22, 0.004, 0.22, Wave::Sine, move |_| f);
        t.osc(st, 0.35, 0.07, 0.004, 0.22, Wave::Sine, move |_| f * 2.76);
    }
    t.osc(0.20, 0.40, 0.18, 0.01, 0.30, Wave::Sine, |x| 1567.98 - 400.0 * x);
    t.normalize(0.58);
    wav_bytes(&t.buf)
}

fn make_destroy() -> Vec<u8> {
    let mut t = Track::new(0.85);
    // Подземный толчок: саб-синус с падением
    t.osc(0.0, 0.70, 0.85, 0.004, 0.30, Wave::Sine, |t| 120.0 - 90.0 * t);
    // Рокот из интегрированного шума (коричневый шум)
    let n = t.buf.len();
    let mut acc = 0.0f32;
    for i in 0..n {
        acc += macroquad::prelude::rand::gen_range(-1.0f32, 1.0) * 0.02;
        acc *= 0.995;
        let env = (-(i as f32 / n as f32) * 4.5).exp();
        t.buf[i] += acc * 6.0 * env * 0.5;
    }
    t.lowpass(0.25);
    t.normalize(0.85);
    wav_bytes(&t.buf)
}

fn make_boss() -> Vec<u8> {
    let mut t = Track::new(0.34);
    t.osc(0.0, 0.28, 0.5, 0.003, 0.12, Wave::Saw, |t| 150.0 + 60.0 * (t * 40.0).sin());
    t.osc(0.0, 0.30, 0.22, 0.003, 0.14, Wave::Sine, |_| 440.0);
    t.osc(0.02, 0.26, 0.12, 0.003, 0.14, Wave::Sine, |_| 443.0); // биения
    t.lowpass(0.4);
    t.normalize(0.6);
    wav_bytes(&t.buf)
}

fn make_ui() -> Vec<u8> {
    let mut t = Track::new(0.06);
    t.osc(0.0, 0.05, 0.20, 0.002, 0.03, Wave::Sine, |_| 1250.0);
    t.normalize(0.35);
    wav_bytes(&t.buf)
}

fn make_error() -> Vec<u8> {
    let mut t = Track::new(0.20);
    t.osc(0.00, 0.08, 0.30, 0.003, 0.03, Wave::Square, |_| 160.0);
    t.osc(0.10, 0.09, 0.30, 0.003, 0.04, Wave::Square, |_| 120.0);
    t.lowpass(0.4);
    t.normalize(0.5);
    wav_bytes(&t.buf)
}

fn make_prestige() -> Vec<u8> {
    let mut t = Track::new(1.5);
    // Подъём
    t.osc(0.0, 0.9, 0.30, 0.05, 0.2, Wave::Sine, |x| 180.0 + 700.0 * x / 0.9);
    // Финальный аккорд (ля мажор)
    for &f in &[220.0f32, 277.18, 329.63, 440.0] {
        t.osc(0.85, 0.6, 0.16, 0.02, 0.4, Wave::Sine, move |_| f);
        t.osc(0.85, 0.6, 0.05, 0.02, 0.4, Wave::Sine, move |_| f * 2.0);
    }
    t.normalize(0.7);
    wav_bytes(&t.buf)
}

/// Эмбиент-луп 16с: 4 аккорда (Am, F, C, G) + пентатонические арпеджио.
fn make_music() -> Vec<u8> {
    let mut t = Track::new(16.0);
    let chords: [[f32; 4]; 4] = [
        [110.0, 220.0, 261.63, 329.63],   // Am
        [87.31, 174.61, 220.0, 261.63],   // F
        [130.81, 196.0, 261.63, 329.63],  // C
        [98.0, 196.0, 246.94, 293.66],    // G
    ];
    for (ci, chord) in chords.iter().enumerate() {
        let start = 4.0 * ci as f32;
        for (ni, &f) in chord.iter().enumerate() {
            let amp = if ni == 0 { 0.16 } else { 0.10 };
            // Детюн-пэд: два сине-осциллятора с расстройкой
            t.osc(start, 4.0, amp, 1.0, 1.0, Wave::Sine, move |_| f * 0.9985);
            t.osc(start, 4.0, amp, 1.0, 1.0, Wave::Sine, move |_| f * 1.0015);
        }
    }
    // Арпеджио: спокойный паттерн по пентатонике ля-минор
    let pent = [440.0f32, 523.25, 659.25, 783.99, 880.0];
    let pattern = [0usize, 2, 4, 2, 3, 1, 2, 0, 4, 3, 2, 1, 0, 2, 3, 4,
                   2, 4, 3, 0, 1, 3, 4, 2, 0, 1, 2, 3, 4, 2, 1, 0];
    for (i, &p) in pattern.iter().enumerate() {
        let st = i as f32 * 0.5;
        let f = pent[p];
        t.osc(st, 0.45, 0.055, 0.01, 0.35, Wave::Sine, move |_| f);
    }
    t.lowpass(0.22);
    t.normalize(0.72);
    wav_bytes(&t.buf)
}

/// Синтез всего банка звуков. Вызывается один раз при старте (async — декодер).
pub async fn synth_all() -> SfxBank {
    SfxBank {
        click: decode(make_click()).await,
        crit: decode(make_crit()).await,
        buy: decode(make_buy()).await,
        relic: decode(make_relic()).await,
        destroy: decode(make_destroy()).await,
        boss: decode(make_boss()).await,
        ui: decode(make_ui()).await,
        error: decode(make_error()).await,
        prestige: decode(make_prestige()).await,
        music: decode(make_music()).await,
    }
}
