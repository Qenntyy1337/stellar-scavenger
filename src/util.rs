//! Утилиты: форматирование чисел, интерполяции, пружины, плавные бары.

/// Суффиксы больших чисел: 1.25K, 3.4M, 8.1B, ...
const SUFFIXES: [&str; 16] = [
    "", "K", "M", "B", "T", "Qa", "Qi", "Sx", "Sp", "Oc", "No", "Dc", "UDc", "DDc", "TDc", "QaDc",
];

/// Красивый формат большого числа: до 3 значащих цифр + суффикс.
pub fn fmt_num(v: f64) -> String {
    if !v.is_finite() {
        return "?".to_string();
    }
    let neg = v < 0.0;
    let v = v.abs();
    if v < 1.0 {
        let s = format!("{:.1}", v);
        return if neg { format!("-{}", s) } else { s };
    }
    if v < 1000.0 {
        let s = if v >= 100.0 {
            format!("{}", v.round() as i64)
        } else {
            format!("{:.1}", v)
        };
        return if neg { format!("-{}", s) } else { s };
    }
    let mut x = v;
    let mut idx = 0usize;
    while x >= 1000.0 && idx < SUFFIXES.len() - 1 {
        x /= 1000.0;
        idx += 1;
    }
    let s = if x >= 100.0 {
        format!("{:.0}{}", x, SUFFIXES[idx])
    } else if x >= 10.0 {
        format!("{:.1}{}", x, SUFFIXES[idx])
    } else {
        format!("{:.2}{}", x, SUFFIXES[idx])
    };
    if neg { format!("-{}", s) } else { s }
}

/// Целое с разделителями тысяч: 1 234 567.
pub fn fmt_int(v: u64) -> String {
    let s = v.to_string();
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 {
            out.push(' ');
        }
        out.push(b as char);
    }
    out
}

pub fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

pub fn ease_out_back(t: f32) -> f32 {
    let t = clamp01(t);
    let c = 1.70158;
    1.0 + (c + 1.0) * (t - 1.0).powi(3) + c * (t - 1.0).powi(2)
}

/// Экспоненциальное сглаживание к цели (независимо от FPS).
pub fn smooth(cur: f32, target: f32, rate: f32, dt: f32) -> f32 {
    cur + (target - cur) * (1.0 - (-rate * dt).exp())
}

/// Пружина (для пульса планеты при ударе).
pub struct Spring {
    pub value: f32,
    pub vel: f32,
    pub target: f32,
    pub stiffness: f32,
    pub damping: f32,
}

impl Spring {
    pub fn new(value: f32, stiffness: f32, damping: f32) -> Self {
        Self { value, vel: 0.0, target: value, stiffness, damping }
    }

    pub fn impulse(&mut self, vel: f32) {
        self.vel += vel;
    }

    pub fn update(&mut self, dt: f32) {
        // Полунеявный интегратор — стабилен при любых dt.
        let steps = ((dt / 0.008).ceil() as usize).clamp(1, 8);
        let h = dt / steps as f32;
        for _ in 0..steps {
            let acc = (self.target - self.value) * self.stiffness - self.vel * self.damping;
            self.vel += acc * h;
            self.value += self.vel * h;
        }
    }
}

/// Плавный прогресс-бар с «призрачным» хвостом (как в файтингах).
pub struct SmoothBar {
    pub value: f32,
    pub ghost: f32,
}

impl SmoothBar {
    pub fn new(value: f32) -> Self {
        Self { value, ghost: value }
    }

    pub fn set(&mut self, v: f32) {
        self.value = v;
    }

    pub fn update(&mut self, target: f32, dt: f32) {
        self.value = smooth(self.value, target, 18.0, dt);
        // Призрак догоняет медленнее, но только вниз (показывает свежий урон).
        if self.ghost > target {
            self.ghost = smooth(self.ghost, target, 3.5, dt);
        } else {
            self.ghost = smooth(self.ghost, target, 30.0, dt);
        }
    }
}

/// Прямоугольник (для хит-тестов UI).
#[derive(Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.w && py >= self.y && py <= self.y + self.h
    }
}
