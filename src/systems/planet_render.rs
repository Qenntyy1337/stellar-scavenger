//! Процедурный рендер планет с псевдо-3D:
//! 1) На смене планеты генерируется equirectangular-полоса поверхности
//!    (1024x256) с бесшовным 3D value-noise на единичной сфере.
//! 2) Предрассчитывается карта сферических пикселей (нормали, освещение).
//! 3) Раз в два кадра сфера пересобирается со сдвигом долготы (вращение),
//!    умножается на освещение и заливается в GPU-текстуру.

use crate::components::planet::{BiomeDef, PlanetStyle, BIOMES};
use macroquad::prelude::*;

const STRIP_W: usize = 1024;
const STRIP_H: usize = 256;
const SPHERE: usize = 320;

/// Предрассчитанные параметры одного пикселя сферы.
#[derive(Clone, Copy)]
struct SpherePx {
    u0: f32,   // долгота при нулевом повороте [0..1)
    v: f32,    // широта [0..1)
    shade: f32,// диффузное освещение 0..1
    rim: f32,  // яркость атмосферного ободка 0..1
    nz: f32,   // z-компонента нормали (лимбовое затемнение)
    inside: bool,
}

pub struct PlanetRenderer {
    strip: Image,
    sphere: Image,
    tex: Texture2D,
    map: Vec<SpherePx>,
    rot: f32,
    frame: u64,
    atmo: [f32; 3],
    ringed: bool,
    #[allow(dead_code)]
    seed: u64,
}

// --- Хэш и value-noise -------------------------------------------------------

#[inline]
fn hash3(x: i32, y: i32, z: i32, seed: u64) -> f32 {
    let mut h = (x as u64).wrapping_mul(0x9E37_79B1_85EB_CA87)
        ^ (y as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
        ^ (z as u64).wrapping_mul(0x1656_67B1_9E37_79F9)
        ^ seed.wrapping_mul(0x27D4_EB2F_1656_67C5);
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    h ^= h >> 33;
    (h >> 40) as f32 / (1u32 << 24) as f32
}

#[inline]
fn fade(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Trilinear value noise на целочисленной решётке.
fn noise3(x: f32, y: f32, z: f32, seed: u64) -> f32 {
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let iz = z.floor() as i32;
    let fx = fade(x - ix as f32);
    let fy = fade(y - iy as f32);
    let fz = fade(z - iz as f32);

    let c000 = hash3(ix, iy, iz, seed);
    let c100 = hash3(ix + 1, iy, iz, seed);
    let c010 = hash3(ix, iy + 1, iz, seed);
    let c110 = hash3(ix + 1, iy + 1, iz, seed);
    let c001 = hash3(ix, iy, iz + 1, seed);
    let c101 = hash3(ix + 1, iy, iz + 1, seed);
    let c011 = hash3(ix, iy + 1, iz + 1, seed);
    let c111 = hash3(ix + 1, iy + 1, iz + 1, seed);

    let x00 = c000 + (c100 - c000) * fx;
    let x10 = c010 + (c110 - c010) * fx;
    let x01 = c001 + (c101 - c001) * fx;
    let x11 = c011 + (c111 - c011) * fx;
    let y0 = x00 + (x10 - x00) * fy;
    let y1 = x01 + (x11 - x01) * fy;
    y0 + (y1 - y0) * fz
}

/// Фрактальный шум (4 октавы), диапазон примерно 0..1.
fn fbm(x: f32, y: f32, z: f32, seed: u64) -> f32 {
    let mut v = 0.0;
    let mut amp = 0.5;
    let mut f = 1.0;
    for _ in 0..4 {
        v += amp * noise3(x * f, y * f, z * f, seed);
        f *= 2.15;
        amp *= 0.5;
    }
    v
}

/// Гребневый шум для разломов/кратеров/трещин.
fn ridge(x: f32, y: f32, z: f32, seed: u64) -> f32 {
    let n = fbm(x, y, z, seed);
    (1.0 - (n - 0.5).abs() * 4.0).clamp(0.0, 1.0)
}

#[inline]
fn mix3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Канал альфа полосы: 0 = земля (освещается), 128 = океан (блик), 200 = эмиссия.
const A_LAND: u8 = 0;
const A_OCEAN: u8 = 128;
const A_EMISSIVE: u8 = 200;

impl PlanetRenderer {
    pub fn new() -> PlanetRenderer {
        let strip = Image::gen_image_color(STRIP_W as u16, STRIP_H as u16, BLANK);
        let sphere = Image::gen_image_color(SPHERE as u16, SPHERE as u16, BLANK);
        let tex = Texture2D::from_image(&sphere);

        // Предрассчёт сферической карты (ортографическая проекция).
        let mut map = Vec::with_capacity(SPHERE * SPHERE);
        // Направление на свет: слева-сверху-спереди (в экранных осях, y вниз).
        let (lx, ly, lz) = (-0.52f32, -0.38f32, 0.77f32);
        let inv = 1.0 / (lx * lx + ly * ly + lz * lz).sqrt();
        let (lx, ly, lz) = (lx * inv, ly * inv, lz * inv);
        let cx = SPHERE as f32 * 0.5;
        let cy = SPHERE as f32 * 0.5;
        let r = SPHERE as f32 * 0.5 - 1.0;
        for y in 0..SPHERE {
            for x in 0..SPHERE {
                let dx = (x as f32 + 0.5 - cx) / r;
                let dy = (y as f32 + 0.5 - cy) / r;
                let d2 = dx * dx + dy * dy;
                if d2 > 1.0 {
                    map.push(SpherePx { u0: 0.0, v: 0.0, shade: 0.0, rim: 0.0, nz: 0.0, inside: false });
                    continue;
                }
                let nz = (1.0 - d2).sqrt();
                let lat = dy.asin();
                let lon = dx.atan2(nz);
                let u0 = (lon / std::f32::consts::TAU + 0.5).fract();
                let v = (0.5 - lat / std::f32::consts::PI).clamp(0.0, 1.0);
                let shade = (dx * lx + dy * ly + nz * lz).max(0.0).powf(1.15);
                let rim = (1.0 - nz).powf(3.0);
                map.push(SpherePx { u0, v, shade, rim, nz, inside: true });
            }
        }
        PlanetRenderer {
            strip,
            sphere,
            tex,
            map,
            rot: 0.0,
            frame: 0,
            atmo: [0.5, 0.7, 1.0],
            ringed: false,
            seed: 0,
        }
    }

    /// Полная перегенерация поверхности под биом и сид планеты.
    pub fn generate(&mut self, biome: &BiomeDef, seed: u64) {
        self.atmo = biome.atmo;
        self.ringed = biome.ringed;
        self.seed = seed;
        let scale = match biome.style {
            PlanetStyle::Ice | PlanetStyle::Crystal => 3.2,
            _ => 2.4,
        };
        let px = self.strip.bytes.as_mut_slice();
        for y in 0..STRIP_H {
            let lat = (0.5 - (y as f32 + 0.5) / STRIP_H as f32) * std::f32::consts::PI;
            let (cl, sl) = (lat.cos(), lat.sin());
            for x in 0..STRIP_W {
                let lon = ((x as f32 + 0.5) / STRIP_W as f32) * std::f32::consts::TAU;
                let px3 = cl * lon.cos() * scale;
                let py3 = sl * scale;
                let pz3 = cl * lon.sin() * scale;

                let n = fbm(px3, py3, pz3, seed);
                let r2 = ridge(px3 * 1.7, py3 * 1.7, pz3 * 1.7, seed ^ 0xBEEF);
                let cap_edge = fbm(px3 * 0.8, py3 * 0.8, pz3 * 0.8, seed ^ 0xCAFE) * 0.15;

                let (mut c, mut a) = Self::style_color(biome.style, n, r2);

                // Полярные шапки для твёрдых миров
                if matches!(biome.style, PlanetStyle::Rocky | PlanetStyle::Ice | PlanetStyle::Organic) {
                    if sl.abs() > 0.60 + cap_edge {
                        let t = ((sl.abs() - 0.60 - cap_edge) * 6.0).min(1.0);
                        c = mix3(c, [0.93, 0.96, 1.0], t);
                        a = A_LAND;
                    }
                }
                // Облака для живых миров
                if biome.style == PlanetStyle::Organic {
                    let cloud = fbm(px3 * 3.0, py3 * 3.0 + 5.0, pz3 * 3.0, seed ^ 0x5EED);
                    if cloud > 0.58 && a != A_EMISSIVE {
                        c = mix3(c, [0.92, 0.95, 0.98], ((cloud - 0.58) * 4.0).min(0.85));
                        a = A_LAND;
                    }
                }
                // Лёгкая полярная подсветка для ледяных гигантов
                if biome.style == PlanetStyle::Ice {
                    c = mix3(c, [0.95, 0.98, 1.0], (sl.abs().powf(3.0) * 0.25).min(0.8));
                }

                let i = (y * STRIP_W + x) * 4;
                px[i] = (c[0] * 255.0) as u8;
                px[i + 1] = (c[1] * 255.0) as u8;
                px[i + 2] = (c[2] * 255.0) as u8;
                px[i + 3] = a;
            }
        }
        self.frame = 0;
        self.update(0.0, 1.0);
    }

    /// Цвет поверхности по стилю биома. Возвращает (RGB, alpha-канал).
    fn style_color(style: PlanetStyle, n: f32, ridge: f32) -> ([f32; 3], u8) {
        match style {
            PlanetStyle::Rocky => {
                if n < 0.38 {
                    (mix3(BIOMES[0].deep, BIOMES[0].mid, n / 0.38), A_OCEAN)
                } else {
                    let c = mix3(BIOMES[0].mid, BIOMES[0].light, (n - 0.38) / 0.62);
                    let crater = if ridge > 0.82 { 0.35 } else { 0.0 };
                    (mix3(c, [0.10, 0.09, 0.08], crater), A_LAND)
                }
            }
            PlanetStyle::Lava => {
                if ridge > 0.62 || n < 0.30 {
                    (mix3([1.0, 0.55, 0.05], [1.0, 0.85, 0.2], ridge), A_EMISSIVE)
                } else {
                    (mix3(BIOMES[1].deep, BIOMES[1].mid, n), A_LAND)
                }
            }
            PlanetStyle::Ice => {
                if n < 0.42 {
                    (mix3(BIOMES[2].deep, BIOMES[2].mid, n / 0.42), A_OCEAN)
                } else {
                    (mix3(BIOMES[2].mid, BIOMES[2].light, (n - 0.42) / 0.58), A_LAND)
                }
            }
            PlanetStyle::Organic => {
                if n < 0.40 {
                    (mix3([0.02, 0.10, 0.25], [0.05, 0.20, 0.40], n / 0.40), A_OCEAN)
                } else if n < 0.62 {
                    (mix3(BIOMES[3].mid, BIOMES[3].light, (n - 0.40) / 0.22), A_LAND)
                } else {
                    (mix3(BIOMES[3].light, [0.75, 0.72, 0.60], (n - 0.62) / 0.38), A_LAND)
                }
            }
            PlanetStyle::Void => {
                let vein = ridge > 0.70 && n > 0.35;
                let base = mix3(BIOMES[4].deep, BIOMES[4].mid, n);
                if vein {
                    (mix3(BIOMES[4].light, [0.9, 0.5, 1.0], ridge), A_EMISSIVE)
                } else if n < 0.25 {
                    (mix3([0.02, 0.01, 0.04], base, n * 4.0), A_OCEAN)
                } else {
                    (base, A_LAND)
                }
            }
            PlanetStyle::Crystal => {
                let band = (n * 9.0).sin().abs();
                let c = mix3(BIOMES[5].deep, BIOMES[5].light, band * 0.7 + n * 0.3);
                if ridge > 0.78 {
                    ([0.8, 1.0, 1.0], A_EMISSIVE)
                } else if n < 0.30 {
                    (c, A_OCEAN)
                } else {
                    (c, A_LAND)
                }
            }
            PlanetStyle::Infernal => {
                if ridge > 0.55 || n < 0.28 {
                    (mix3([1.0, 0.35, 0.02], [1.0, 0.9, 0.3], ridge), A_EMISSIVE)
                } else {
                    (mix3(BIOMES[6].deep, BIOMES[6].mid, n), A_LAND)
                }
            }
            PlanetStyle::Exotic => {
                let marble = (n * 5.0 + ridge * 3.0).sin().abs();
                if marble > 0.75 {
                    ([1.0, 0.85, 0.3], A_EMISSIVE)
                } else {
                    (mix3(BIOMES[7].deep, BIOMES[7].mid, n), A_LAND)
                }
            }
        }
    }

    /// Сборка сферы со сдвигом долготы и освещением.
    /// Вызывается при generate() и раз в 2 кадра из игрового цикла.
    pub fn update(&mut self, dt: f32, anim_speed: f32) {
        self.rot = (self.rot + dt * 0.035 * anim_speed.max(0.0)) % 1.0;
        let rot = self.rot;
        let atmo = self.atmo;
        let sw = STRIP_W as f32;
        let sh = STRIP_H as f32;
        let strip = self.strip.bytes.clone(); // 1 МБ копия — дешевле борьбы с borrow
        let sph = self.sphere.bytes.as_mut_slice();
        let time = get_time() as f32;
        let flicker = 1.0 + 0.05 * (time * 3.0).sin();

        for (i, m) in self.map.iter().enumerate() {
            let o = i * 4;
            if !m.inside {
                sph[o] = 0;
                sph[o + 1] = 0;
                sph[o + 2] = 0;
                sph[o + 3] = 0;
                continue;
            }
            // Билинейная выборка полосы со сдвигом долготы (обёртка по X)
            let u = (m.u0 + rot).fract();
            let sx = u * sw;
            let sy = (m.v * sh).clamp(0.0, sh - 1.001);
            let x0 = sx as usize % STRIP_W;
            let x1 = (x0 + 1) % STRIP_W;
            let y0 = (sy as usize).min(STRIP_H - 1);
            let y1 = (y0 + 1).min(STRIP_H - 1);
            let fx = sx.fract();
            let fy = sy.fract();
            let i00 = (y0 * STRIP_W + x0) * 4;
            let i10 = (y0 * STRIP_W + x1) * 4;
            let i01 = (y1 * STRIP_W + x0) * 4;
            let i11 = (y1 * STRIP_W + x1) * 4;
            let mut r = (strip[i00] as f32 * (1.0 - fx) + strip[i10] as f32 * fx) * (1.0 - fy)
                + (strip[i01] as f32 * (1.0 - fx) + strip[i11] as f32 * fx) * fy;
            let mut g = (strip[i00 + 1] as f32 * (1.0 - fx) + strip[i10 + 1] as f32 * fx) * (1.0 - fy)
                + (strip[i01 + 1] as f32 * (1.0 - fx) + strip[i11 + 1] as f32 * fx) * fy;
            let mut b = (strip[i00 + 2] as f32 * (1.0 - fx) + strip[i10 + 2] as f32 * fx) * (1.0 - fy)
                + (strip[i01 + 2] as f32 * (1.0 - fx) + strip[i11 + 2] as f32 * fx) * fy;
            let a = (strip[i00 + 3] as f32 * (1.0 - fx) + strip[i10 + 3] as f32 * fx) * (1.0 - fy)
                + (strip[i01 + 3] as f32 * (1.0 - fx) + strip[i11 + 3] as f32 * fx) * fy;

            let limb = 0.55 + 0.45 * m.nz;
            if a >= 199.0 {
                // Эмиссия (лава/кристаллы): светится сама, слегка пульсирует
                let k = 0.85 + 0.15 * flicker;
                r *= k;
                g *= k;
                b *= k;
            } else {
                let bright = (0.16 + 1.02 * m.shade) * limb;
                r *= bright;
                g *= bright;
                b *= bright;
                if a >= 127.0 {
                    // Зеркальный блик океана/льда
                    let spec = m.shade.powf(14.0) * 130.0;
                    r += spec;
                    g += spec;
                    b += spec;
                }
            }
            // Атмосферный ободок (аддитивно)
            let rim = m.rim * 0.85;
            r += atmo[0] * 255.0 * rim;
            g += atmo[1] * 255.0 * rim;
            b += atmo[2] * 255.0 * rim;

            sph[o] = r.clamp(0.0, 255.0) as u8;
            sph[o + 1] = g.clamp(0.0, 255.0) as u8;
            sph[o + 2] = b.clamp(0.0, 255.0) as u8;
            sph[o + 3] = if rim > 0.02 { ((rim * 3.0).min(1.0) * 255.0) as u8 } else { 255 };
        }

        self.tex.update(&self.sphere);
        self.frame = self.frame.wrapping_add(1);
    }

    /// Пересобирать ли сферу в этом кадре (каждый 2-й, экономим CPU).
    pub fn wants_update(&self) -> bool {
        self.frame % 2 == 0
    }

    /// Диагностика: экспорт текстуры сферы.
    pub fn debug_export(&self, path: &str) {
        self.sphere.export_png(path);
    }

    /// RGB атмосферы (для внешней ауры).
    pub fn atmo_rgb(&self) -> (f32, f32, f32) {
        (self.atmo[0], self.atmo[1], self.atmo[2])
    }

    /// Отрисовка планеты (и колец, если биом «кольцевой»).
    pub fn draw(&self, cx: f32, cy: f32, radius: f32) {
        if self.ringed {
            Self::draw_ring(cx, cy, radius, true);
        }
        let d = radius * 2.35; // запас под свечение ободка
        let params = DrawTextureParams {
            dest_size: Some(Vec2::new(d, d)),
            ..Default::default()
        };
        draw_texture_ex(&self.tex, cx - d / 2.0, cy - d / 2.0, WHITE, params);
        if self.ringed {
            Self::draw_ring(cx, cy, radius, false);
        }
    }

    /// Кольца планеты: эллипс из точек, задняя половина рисуется за планетой.
    fn draw_ring(cx: f32, cy: f32, radius: f32, back: bool) {
        let n = 56;
        let tilt = 0.30f32;
        for i in 0..n {
            let a = i as f32 / n as f32 * std::f32::consts::TAU;
            let is_back = a.sin() < 0.0;
            if is_back != back {
                continue;
            }
            let sx = a.sin();
            let ca = a.cos();
            let sy = (ca * tilt).abs();
            for &(rr, alpha, wcol) in [
                (1.55f32, 0.35f32, [0.9f32, 0.85, 0.7]),
                (1.78, 0.22, [0.8, 0.8, 0.9]),
            ]
            .iter()
            {
                let px = cx + sx * radius * rr;
                let py = cy + ca * radius * rr * tilt;
                let col = Color::new(wcol[0], wcol[1], wcol[2], alpha * (0.5 + 0.5 * sy));
                draw_circle(px, py, radius * 0.045, col);
            }
        }
    }
}
