//! Векторные иконки UI, отрисовываемые примитивами macroquad.
//! ЭМОДЗИ ЗАПРЕЩЕНЫ — только геометрия: линии, круги, полигоны.

use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)] // полный каталог векторных иконок; часть — задел под будущие экраны
pub enum Icon {
    Ore,
    DarkMatter,
    Drone,
    Drill,
    Laser,
    Plasma,
    Quantum,
    Singularity,
    Harvester,
    Relic,
    Crit,
    Settings,
    Music,
    Sound,
    Fullscreen,
    Save,
    Prestige,
    Shield,
    Boss,
    Bolt,
    Clock,
    Lock,
    Check,
    Cross,
    ArrowUp,
    Infinity,
    Trash,
    Planet,
    Admin,
}

/// Рисует иконку в квадрате size x size с центром (cx, cy).
pub fn draw_icon(icon: Icon, cx: f32, cy: f32, size: f32, color: Color) {
    let h = size / 2.0;
    match icon {
        Icon::Ore => {
            // Кристалл-руда: ромб с гранью
            let pts = [
                (cx, cy - h),
                (cx + h * 0.85, cy - h * 0.15),
                (cx, cy + h),
                (cx - h * 0.85, cy - h * 0.15),
            ];
            for i in 0..4 {
                let a = pts[i];
                let b = pts[(i + 1) % 4];
                draw_line(a.0, a.1, b.0, b.1, size * 0.08, color);
            }
            draw_line(cx, cy - h, cx, cy + h, size * 0.05, Color::new(color.r, color.g, color.b, color.a * 0.5));
        }
        Icon::DarkMatter => {
            // Атом: ядро + две орбиты + электроны
            draw_circle(cx, cy, size * 0.14, color);
            for rot in [0.55f32, -0.85] {
                draw_ellipse_arc(cx, cy, size * 0.46, size * 0.2, rot, color);
            }
            let t = macroquad::prelude::get_time() as f32 * 1.4;
            let ex = cx + (t.cos()) * size * 0.44;
            let ey = cy + (t.sin()) * size * 0.19;
            draw_circle(ex, ey, size * 0.06, color);
        }
        Icon::Drone => {
            // Корпус + две стойки + винты
            draw_circle(cx, cy, size * 0.22, color);
            draw_line(cx - h * 0.8, cy - h * 0.7, cx - h * 0.3, cy - h * 0.2, size * 0.06, color);
            draw_line(cx + h * 0.8, cy - h * 0.7, cx + h * 0.3, cy - h * 0.2, size * 0.06, color);
            draw_line(cx - h * 0.95, cy - h * 0.72, cx - h * 0.55, cy - h * 0.72, size * 0.08, color);
            draw_line(cx + h * 0.55, cy - h * 0.72, cx + h * 0.95, cy - h * 0.72, size * 0.08, color);
        }
        Icon::Drill => {
            // Треугольный бур вниз
            draw_poly(cx, cy, 3, h, 90.0, color);
            draw_line(cx - h * 0.5, cy - h * 0.4, cx + h * 0.5, cy - h * 0.4, size * 0.09, color);
        }
        Icon::Laser => {
            // Луч: две скобки и линия
            draw_line(cx - h * 0.9, cy, cx + h * 0.9, cy, size * 0.09, color);
            draw_circle(cx + h * 0.9, cy, size * 0.13, color);
            draw_line(cx - h * 0.9, cy - h * 0.5, cx - h * 0.9, cy + h * 0.5, size * 0.07, color);
        }
        Icon::Plasma => {
            // Завихрение: круг + дуга
            draw_circle_lines_1(cx, cy, h * 0.55, size * 0.07, color);
            draw_arc(cx, cy, h * 0.9, 0.4, 2.6, size * 0.06, color);
            draw_circle(cx + h * 0.9, cy - h * 0.35, size * 0.09, color);
        }
        Icon::Quantum => {
            // Ромб + внутренний ромб
            draw_poly(cx, cy, 4, h, 45.0, color);
            draw_poly_lines(cx, cy, 4, h * 0.45, 45.0, size * 0.05, Color::new(0.05, 0.06, 0.1, 1.0));
        }
        Icon::Singularity => {
            // Чёрная дыра с аккреционным диском
            draw_circle(cx, cy, size * 0.2, Color::new(0.04, 0.03, 0.08, 1.0));
            draw_circle_lines_1(cx, cy, size * 0.2, size * 0.07, color);
            draw_arc(cx, cy, h * 0.8, 0.2, 2.2, size * 0.07, color);
            draw_arc(cx, cy, h * 0.8, 3.3, 2.2, size * 0.07, color);
        }
        Icon::Harvester => {
            // Звезда-жнец
            draw_poly(cx, cy, 8, h * 0.95, 22.5, color);
            draw_circle(cx, cy, size * 0.16, Color::new(0.05, 0.06, 0.12, 1.0));
        }
        Icon::Relic => {
            // Реликварий: гексагон с искрой
            draw_poly_lines(cx, cy, 6, h * 0.95, 30.0, size * 0.07, color);
            draw_poly(cx, cy, 4, h * 0.32, 45.0, color);
        }
        Icon::Crit => {
            // Молния
            let pts = [
                (cx + h * 0.15, cy - h),
                (cx - h * 0.45, cy + h * 0.1),
                (cx - h * 0.02, cy + h * 0.1),
                (cx - h * 0.15, cy + h),
                (cx + h * 0.5, cy - h * 0.15),
                (cx + h * 0.05, cy - h * 0.15),
            ];
            for i in 0..pts.len() {
                let a = pts[i];
                let b = pts[(i + 1) % pts.len()];
                if i % 2 == 0 {
                    draw_line(a.0, a.1, b.0, b.1, size * 0.09, color);
                } else {
                    draw_line(a.0, a.1, b.0, b.1, size * 0.09, color);
                }
            }
        }
        Icon::Settings => {
            // Шестерня: 8 зубьев + кольцо
            for i in 0..8 {
                let a = i as f32 / 8.0 * std::f32::consts::TAU;
                let tx = cx + a.cos() * h * 0.85;
                let ty = cy + a.sin() * h * 0.85;
                draw_circle(tx, ty, size * 0.09, color);
            }
            draw_circle_lines_1(cx, cy, h * 0.62, size * 0.09, color);
            draw_circle_lines_1(cx, cy, h * 0.28, size * 0.07, color);
        }
        Icon::Music => {
            // Нота
            draw_circle(cx - h * 0.35, cy + h * 0.55, size * 0.15, color);
            draw_line(cx - h * 0.2, cy + h * 0.5, cx - h * 0.2, cy - h * 0.75, size * 0.07, color);
            draw_line(cx - h * 0.2, cy - h * 0.75, cx + h * 0.55, cy - h * 0.5, size * 0.07, color);
            draw_line(cx + h * 0.55, cy - h * 0.5, cx + h * 0.55, cy + h * 0.25, size * 0.07, color);
            draw_circle(cx + h * 0.4, cy + h * 0.35, size * 0.15, color);
        }
        Icon::Sound => {
            // Динамик: рупор + дуги
            draw_poly(cx - h * 0.25, cy, 3, h * 0.5, -90.0, color);
            draw_rectangle(cx - h * 0.55, cy - h * 0.18, size * 0.14, size * 0.36, color);
            draw_arc(cx - h * 0.1, cy, h * 0.6, -0.7, 1.4, size * 0.06, color);
            draw_arc(cx - h * 0.1, cy, h * 0.95, -0.7, 1.4, size * 0.06, color);
        }
        Icon::Fullscreen => {
            // Четыре уголка
            let d = h * 0.75;
            let l = size * 0.28;
            for (sx, sy) in [(-1.0f32, -1.0f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                let px = cx + sx * d;
                let py = cy + sy * d;
                draw_line(px, py, px - sx * l, py, size * 0.07, color);
                draw_line(px, py, px, py - sy * l, size * 0.07, color);
            }
        }
        Icon::Save => {
            // Дискета
            let w = h * 0.9;
            draw_rectangle_lines(cx - w, cy - w, w * 2.0, w * 2.0, size * 0.08, color);
            draw_rectangle(cx + w * 0.25, cy - w, w * 0.75, w * 0.9, color);
            draw_rectangle_lines(cx - w * 0.45, cy + w * 0.15, w * 0.9, w * 0.85, size * 0.06, color);
        }
        Icon::Prestige => {
            // Ракета
            draw_poly(cx, cy - h * 0.2, 3, h * 0.55, 0.0, color);
            draw_rectangle(cx - h * 0.22, cy, size * 0.16, h * 0.6, color);
            draw_rectangle(cx + h * 0.06, cy, size * 0.16, h * 0.6, color);
            draw_triangle(
                Vec2::new(cx - h * 0.35, cy + h * 0.55),
                Vec2::new(cx - h * 0.05, cy + h * 0.55),
                Vec2::new(cx - h * 0.2, cy + h * 0.95),
                color,
            );
            draw_triangle(
                Vec2::new(cx + h * 0.05, cy + h * 0.55),
                Vec2::new(cx + h * 0.35, cy + h * 0.55),
                Vec2::new(cx + h * 0.2, cy + h * 0.95),
                color,
            );
        }
        Icon::Shield => {
            // Щит: пятиугольник
            let pts = [
                (cx, cy - h),
                (cx + h * 0.8, cy - h * 0.5),
                (cx + h * 0.55, cy + h * 0.6),
                (cx, cy + h),
                (cx - h * 0.55, cy + h * 0.6),
                (cx - h * 0.8, cy - h * 0.5),
            ];
            for i in 0..pts.len() {
                let a = pts[i];
                let b = pts[(i + 1) % pts.len()];
                draw_line(a.0, a.1, b.0, b.1, size * 0.08, color);
            }
            draw_line(cx, cy - h * 0.5, cx, cy + h * 0.5, size * 0.05, Color::new(color.r, color.g, color.b, color.a * 0.6));
        }
        Icon::Boss => {
            // Череп-корона босса: круг + глаза + зубцы
            draw_circle_lines_1(cx, cy - h * 0.1, h * 0.62, size * 0.08, color);
            draw_circle(cx - h * 0.25, cy - h * 0.2, size * 0.09, color);
            draw_circle(cx + h * 0.25, cy - h * 0.2, size * 0.09, color);
            for i in -1..=1 {
                draw_line(cx + i as f32 * h * 0.3, cy + h * 0.4, cx + i as f32 * h * 0.3, cy + h * 0.75, size * 0.07, color);
            }
        }
        Icon::Bolt => {
            // Двойная молния скорости
            draw_line(cx - h * 0.3, cy - h * 0.8, cx - h * 0.6, cy + h * 0.1, size * 0.08, color);
            draw_line(cx - h * 0.6, cy + h * 0.1, cx - h * 0.15, cy + h * 0.1, size * 0.08, color);
            draw_line(cx + h * 0.3, cy - h * 0.1, cx, cy + h * 0.8, size * 0.08, color);
            draw_line(cx, cy + h * 0.8, cx + h * 0.45, cy + h * 0.8, size * 0.08, color);
        }
        Icon::Clock => {
            draw_circle_lines_1(cx, cy, h * 0.85, size * 0.08, color);
            draw_line(cx, cy, cx, cy - h * 0.5, size * 0.06, color);
            draw_line(cx, cy, cx + h * 0.35, cy + h * 0.15, size * 0.06, color);
        }
        Icon::Lock => {
            draw_rectangle_lines(cx - h * 0.55, cy - h * 0.1, h * 1.1, h * 0.95, size * 0.08, color);
            draw_arc(cx, cy - h * 0.1, h * 0.4, 3.6, 2.9, size * 0.07, color);
        }
        Icon::Check => {
            draw_line(cx - h * 0.55, cy, cx - h * 0.1, cy + h * 0.45, size * 0.1, color);
            draw_line(cx - h * 0.1, cy + h * 0.45, cx + h * 0.6, cy - h * 0.45, size * 0.1, color);
        }
        Icon::Cross => {
            draw_line(cx - h * 0.5, cy - h * 0.5, cx + h * 0.5, cy + h * 0.5, size * 0.1, color);
            draw_line(cx + h * 0.5, cy - h * 0.5, cx - h * 0.5, cy + h * 0.5, size * 0.1, color);
        }
        Icon::ArrowUp => {
            draw_line(cx, cy - h * 0.7, cx, cy + h * 0.7, size * 0.09, color);
            draw_line(cx - h * 0.4, cy - h * 0.3, cx, cy - h * 0.7, size * 0.09, color);
            draw_line(cx + h * 0.4, cy - h * 0.3, cx, cy - h * 0.7, size * 0.09, color);
        }
        Icon::Infinity => {
            // Лемниската из дуг
            draw_arc(cx - h * 0.35, cy, h * 0.38, 1.2, 3.65, size * 0.07, color);
            draw_arc(cx + h * 0.35, cy, h * 0.38, 4.35, 3.65, size * 0.07, color);
        }
        Icon::Trash => {
            draw_rectangle_lines(cx - h * 0.5, cy - h * 0.35, h, h * 0.95, size * 0.07, color);
            draw_line(cx - h * 0.65, cy - h * 0.35, cx + h * 0.65, cy - h * 0.35, size * 0.07, color);
            draw_line(cx - h * 0.2, cy - h * 0.55, cx + h * 0.2, cy - h * 0.55, size * 0.07, color);
        }
        Icon::Planet => {
            draw_circle_lines_1(cx, cy, h * 0.7, size * 0.08, color);
            draw_arc(cx, cy - h * 0.05, h * 1.05, 3.4, 2.5, size * 0.06, color);
        }
        Icon::Admin => {
            // Терминал: скобка + курсор
            draw_rectangle_lines(cx - h * 0.8, cy - h * 0.6, h * 1.6, h * 1.2, size * 0.07, color);
            draw_line(cx - h * 0.45, cy - h * 0.2, cx - h * 0.1, cy + h * 0.1, size * 0.06, color);
            draw_line(cx - h * 0.1, cy + h * 0.1, cx - h * 0.45, cy + h * 0.4, size * 0.06, color);
            draw_line(cx + h * 0.05, cy + h * 0.4, cx + h * 0.45, cy + h * 0.4, size * 0.06, color);
        }
    }
}

/// Эллиптическая дуга (для орбит/динамиков) — набор точек линиями.
pub fn draw_arc(cx: f32, cy: f32, r: f32, start: f32, sweep: f32, width: f32, color: Color) {
    let steps = 24;
    let mut prev: Option<(f32, f32)> = None;
    for i in 0..=steps {
        let a = start + sweep * i as f32 / steps as f32;
        let p = (cx + a.cos() * r, cy + a.sin() * r * 0.62);
        if let Some(p0) = prev {
            draw_line(p0.0, p0.1, p.0, p.1, width, color);
        }
        prev = Some(p);
    }
}

/// Наклонная орбита-эллипс (для атома).
pub fn draw_ellipse_arc(cx: f32, cy: f32, rx: f32, ry: f32, rotation: f32, color: Color) {
    let steps = 30;
    let (cos_r, sin_r) = rotation.sin_cos();
    let mut prev: Option<(f32, f32)> = None;
    for i in 0..=steps {
        let a = i as f32 / steps as f32 * std::f32::consts::TAU;
        let lx = a.cos() * rx;
        let ly = a.sin() * ry;
        let p = (cx + lx * cos_r - ly * sin_r, cy + lx * sin_r + ly * cos_r);
        if let Some(p0) = prev {
            draw_line(p0.0, p0.1, p.0, p.1, 2.0, color);
        }
        prev = Some(p);
    }
}

/// Окружность линией заданной толщины.
pub fn draw_circle_lines_1(x: f32, y: f32, r: f32, thickness: f32, color: Color) {
    draw_poly_lines(x, y, 40, r, 0.0, thickness, color);
}
