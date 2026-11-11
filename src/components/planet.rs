//! Биомы галактики, кривые HP/наград и генерация имён планет.

use serde::{Deserialize, Serialize};

/// Визуальный стиль планеты (влияет на процедурную генерацию текстуры).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum PlanetStyle {
    Rocky,
    Lava,
    Ice,
    Organic,
    Void,
    Crystal,
    Infernal,
    Exotic,
}

#[derive(Clone, Copy)]
pub struct BiomeDef {
    pub name: &'static str,
    pub code: &'static str,
    pub desc: &'static str,
    /// Основные цвета поверхности (RGB 0..1).
    pub deep: [f32; 3],
    pub mid: [f32; 3],
    pub light: [f32; 3],
    /// Цвет атмосферы (свечение по краю).
    pub atmo: [f32; 3],
    pub hp_mult: f64,
    pub ore_mult: f64,
    pub shield_mult: f64,
    pub ringed: bool,
    pub style: PlanetStyle,
}

/// 8 биомов галактики — от каменного пояса до Края Бесконечности.
pub const BIOMES: [BiomeDef; 8] = [
    BiomeDef {
        name: "Каменный Пояс",
        code: "RL",
        desc: "Обломки протопланет и бедная руда. Начало пути scavenger'а.",
        deep: [0.16, 0.14, 0.13], mid: [0.38, 0.33, 0.29], light: [0.62, 0.56, 0.50],
        atmo: [0.55, 0.60, 0.70], hp_mult: 1.0, ore_mult: 1.0, shield_mult: 1.0,
        ringed: false, style: PlanetStyle::Rocky,
    },
    BiomeDef {
        name: "Багровая Окраина",
        code: "CR",
        desc: "Молодые вулканы и моря расплавленной породы.",
        deep: [0.25, 0.06, 0.04], mid: [0.55, 0.16, 0.08], light: [0.90, 0.45, 0.15],
        atmo: [1.00, 0.35, 0.15], hp_mult: 1.8, ore_mult: 1.5, shield_mult: 1.2,
        ringed: false, style: PlanetStyle::Lava,
    },
    BiomeDef {
        name: "Крио Пояс",
        code: "CY",
        desc: "Ледяные гиганты с океанами под коркой азотного льда.",
        deep: [0.05, 0.18, 0.30], mid: [0.20, 0.45, 0.65], light: [0.75, 0.90, 1.00],
        atmo: [0.40, 0.80, 1.00], hp_mult: 3.2, ore_mult: 2.2, shield_mult: 1.5,
        ringed: true, style: PlanetStyle::Ice,
    },
    BiomeDef {
        name: "Изумрудная Заря",
        code: "VD",
        desc: "Живые миры: споровые леса и кислородные океаны.",
        deep: [0.03, 0.22, 0.16], mid: [0.12, 0.50, 0.30], light: [0.50, 0.85, 0.55],
        atmo: [0.35, 1.00, 0.55], hp_mult: 6.0, ore_mult: 3.0, shield_mult: 1.8,
        ringed: false, style: PlanetStyle::Organic,
    },
    BiomeDef {
        name: "Пустота Нексус",
        code: "VX",
        desc: "Мёртвые миры у барьера тёмной материи.",
        deep: [0.07, 0.04, 0.14], mid: [0.20, 0.10, 0.35], light: [0.45, 0.25, 0.70],
        atmo: [0.70, 0.35, 1.00], hp_mult: 11.0, ore_mult: 4.5, shield_mult: 2.2,
        ringed: true, style: PlanetStyle::Void,
    },
    BiomeDef {
        name: "Квантовый Сад",
        code: "QN",
        desc: "Кристаллические планеты, поющие на частоте 432 Гц.",
        deep: [0.05, 0.25, 0.30], mid: [0.10, 0.55, 0.65], light: [0.30, 0.95, 1.00],
        atmo: [0.30, 1.00, 0.90], hp_mult: 20.0, ore_mult: 7.0, shield_mult: 2.6,
        ringed: false, style: PlanetStyle::Crystal,
    },
    BiomeDef {
        name: "Инфернальное Ядро",
        code: "IF",
        desc: "Мироздание в стадии постоянного взрыва. Осторожно.",
        deep: [0.12, 0.02, 0.02], mid: [0.45, 0.08, 0.05], light: [1.00, 0.60, 0.10],
        atmo: [1.00, 0.20, 0.10], hp_mult: 36.0, ore_mult: 10.0, shield_mult: 3.0,
        ringed: true, style: PlanetStyle::Infernal,
    },
    BiomeDef {
        name: "Край Бесконечности",
        code: "IN",
        desc: "Здесь кончается карта. И начинается легенда.",
        deep: [0.30, 0.24, 0.05], mid: [0.65, 0.55, 0.20], light: [1.00, 0.95, 0.70],
        atmo: [1.00, 0.85, 0.35], hp_mult: 65.0, ore_mult: 15.0, shield_mult: 3.5,
        ringed: true, style: PlanetStyle::Exotic,
    },
];

pub fn biome(idx: usize) -> &'static BiomeDef {
    &BIOMES[idx.min(BIOMES.len() - 1)]
}

/// Босс — каждая 10-я планета биома (индекс 9, отображается как 10/10).
pub fn is_boss(planet_idx: usize) -> bool {
    planet_idx % PLANETS_PER_BIOME == PLANETS_PER_BIOME - 1
}

pub const PLANETS_PER_BIOME: usize = 10;

/// Максимальный HP планеты: экспоненциальная кривая * множители биома * галактика.
pub fn planet_max_hp(biome_idx: usize, planet_idx: usize, galaxy: u32) -> f64 {
    let g = (biome_idx * PLANETS_PER_BIOME + planet_idx) as f64;
    let base = 30.0 * 1.62f64.powf(g);
    let b = biome(biome_idx).hp_mult;
    let boss = if is_boss(planet_idx) { 7.0 } else { 1.0 };
    let galaxy_mult = 20f64.powi(galaxy as i32);
    base * b * boss * galaxy_mult
}

/// Прочность планетарного щита босса.
pub fn shield_max_hp(biome_idx: usize, planet_idx: usize, galaxy: u32) -> f64 {
    if !is_boss(planet_idx) {
        return 0.0;
    }
    planet_max_hp(biome_idx, planet_idx, galaxy) * 0.8 * biome(biome_idx).shield_mult
}

/// Награда рудой за уничтожение планеты.
pub fn planet_ore_reward(biome_idx: usize, planet_idx: usize, galaxy: u32) -> f64 {
    let hp = planet_max_hp(biome_idx, planet_idx, galaxy);
    let boss = if is_boss(planet_idx) { 6.0 } else { 1.0 };
    let galaxy_mult = 8f64.powi(galaxy as i32);
    hp.powf(0.75) * 2.2 * biome(biome_idx).ore_mult * boss * galaxy_mult
}

/// Детерминированное имя планеты вида "CY-347".
pub fn planet_name(biome_idx: usize, planet_idx: usize) -> String {
    let seed = (biome_idx * 7919 + planet_idx * 104729 + 17) as u64;
    let num = (seed.wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407)
        >> 33)
        % 900
        + 100;
    format!("{}-{}", biome(biome_idx).code, num)
}

/// Название босса.
pub fn boss_name(biome_idx: usize, planet_idx: usize) -> String {
    let prefixes = [
        "Титан", "Левиафан", "Монолит", "Колосс", "Гидра", "Феникс", "Химера", "Спектр",
    ];
    format!("{} {}", prefixes[(biome_idx + planet_idx) % prefixes.len()], planet_name(biome_idx, planet_idx))
}
