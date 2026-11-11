//! Реликвии: 16 артефактов 4 редкостей с пассивными бонусами.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Rarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

impl Rarity {
    pub fn name(self) -> &'static str {
        match self {
            Rarity::Common => "Обычная",
            Rarity::Rare => "Редкая",
            Rarity::Epic => "Эпическая",
            Rarity::Legendary => "Легендарная",
        }
    }

    /// RGB-цвет рамки карточки.
    pub fn rgb(self) -> [u8; 3] {
        match self {
            Rarity::Common => [150, 160, 180],
            Rarity::Rare => [80, 170, 255],
            Rarity::Epic => [190, 110, 255],
            Rarity::Legendary => [255, 190, 60],
        }
    }

    /// Базовый вес в таблице дропа.
    pub fn weight(self) -> f64 {
        match self {
            Rarity::Common => 58.0,
            Rarity::Rare => 26.0,
            Rarity::Epic => 12.0,
            Rarity::Legendary => 4.0,
        }
    }
}

/// Эффект реликвии: величина применяется ЗА КАЖДЫЙ уровень (дубликат).
#[derive(Clone, Copy, Debug)]
pub enum RelicEffect {
    /// +доля к множителю урона клика
    ClickDmg(f64),
    /// +доля к шансу крита
    CritChance(f64),
    /// +доля к множителю крит-урона
    CritDmg(f64),
    /// +доля к добыче руды
    Ore(f64),
    /// +доля к DPS дронов
    DronePower(f64),
    /// +доля к скорости дронов/автокликера
    DroneSpeed(f64),
    /// +шанс дропа реликвий (абсолютный, за уровень)
    Luck(f64),
    /// +доля к добыче тёмной материи
    DarkMatter(f64),
    /// +доля урона по боссам и щитам
    BossDmg(f64),
}

#[derive(Clone, Copy)]
pub struct RelicDef {
    pub id: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
    pub rarity: Rarity,
    pub effect: RelicEffect,
}

/// Каталог всех реликвий.
pub const RELICS: [RelicDef; 16] = [
    RelicDef { id: "meteor_shard", name: "Осколок метеорита", desc: "Урон клика +10% за ур.", rarity: Rarity::Common, effect: RelicEffect::ClickDmg(0.10) },
    RelicDef { id: "copper_vein", name: "Медная жила", desc: "Добыча руды +8% за ур.", rarity: Rarity::Common, effect: RelicEffect::Ore(0.08) },
    RelicDef { id: "worn_gyro", name: "Изношенный гироскоп", desc: "Скорость дронов +5% за ур.", rarity: Rarity::Common, effect: RelicEffect::DroneSpeed(0.05) },
    RelicDef { id: "quartz_lens", name: "Кварцевая линза", desc: "Шанс крита +1% за ур.", rarity: Rarity::Common, effect: RelicEffect::CritChance(0.01) },
    RelicDef { id: "plasma_core", name: "Плазменное ядро", desc: "Крит-урон +15% за ур.", rarity: Rarity::Rare, effect: RelicEffect::CritDmg(0.15) },
    RelicDef { id: "magnetar_coil", name: "Обмотка магнетара", desc: "Сила дронов +15% за ур.", rarity: Rarity::Rare, effect: RelicEffect::DronePower(0.15) },
    RelicDef { id: "cryo_crystal", name: "Криокристалл", desc: "Добыча руды +10% за ур.", rarity: Rarity::Rare, effect: RelicEffect::Ore(0.10) },
    RelicDef { id: "graviton_node", name: "Гравитонный узел", desc: "Урон клика +8% за ур.", rarity: Rarity::Rare, effect: RelicEffect::ClickDmg(0.08) },
    RelicDef { id: "pulsar_fragment", name: "Фрагмент пульсара", desc: "Урон клика +25% за ур.", rarity: Rarity::Epic, effect: RelicEffect::ClickDmg(0.25) },
    RelicDef { id: "neutron_heart", name: "Сердце нейтронной звезды", desc: "Сила дронов +30% за ур.", rarity: Rarity::Epic, effect: RelicEffect::DronePower(0.30) },
    RelicDef { id: "dark_reliquary", name: "Тёмный реликварий", desc: "Тёмная материя +10% за ур.", rarity: Rarity::Epic, effect: RelicEffect::DarkMatter(0.10) },
    RelicDef { id: "astro_compass", name: "Астрокомпас", desc: "Шанс дропа реликвий +8% за ур.", rarity: Rarity::Epic, effect: RelicEffect::Luck(0.08) },
    RelicDef { id: "galaxy_heart", name: "Сердце галактики", desc: "Добыча руды +50% за ур.", rarity: Rarity::Legendary, effect: RelicEffect::Ore(0.50) },
    RelicDef { id: "abyss_eye", name: "Око бездны", desc: "Шанс крита +5% за ур.", rarity: Rarity::Legendary, effect: RelicEffect::CritChance(0.05) },
    RelicDef { id: "nova_crown", name: "Корона сверхновой", desc: "Урон клика +50% за ур.", rarity: Rarity::Legendary, effect: RelicEffect::ClickDmg(0.50) },
    RelicDef { id: "singularity_spiral", name: "Спираль сингулярности", desc: "Урон боссам +40% за ур.", rarity: Rarity::Legendary, effect: RelicEffect::BossDmg(0.40) },
];

pub fn relic_by_id(id: &str) -> Option<&'static RelicDef> {
    RELICS.iter().find(|r| r.id == id)
}

/// Ролл реликвии при уничтожении планеты.
/// `depth` — глобальный номер планеты (0..80), усиливает редкости.
pub fn roll_relic(luck: f64, depth: f64) -> &'static RelicDef {
    let depth_boost = (depth * 0.012).min(1.2);
    let weights: [f64; 4] = [
        Rarity::Common.weight(),
        Rarity::Rare.weight() * (1.0 + depth_boost),
        Rarity::Epic.weight() * (1.0 + depth_boost * 1.8) * (1.0 + luck * 0.5),
        Rarity::Legendary.weight() * (1.0 + depth_boost * 2.6) * (1.0 + luck * 0.9),
    ];
    let total: f64 = weights.iter().sum();
    let roll = macroquad::prelude::rand::gen_range(0.0, total as f32) as f64;
    let mut acc = 0.0;
    let mut chosen = 0usize;
    for (idx, w) in weights.iter().enumerate() {
        acc += w;
        if roll < acc {
            chosen = idx;
            break;
        }
    }
    let pool: Vec<&RelicDef> = RELICS.iter().filter(|r| {
        matches!(
            (r.rarity, chosen),
            (Rarity::Common, 0) | (Rarity::Rare, 1) | (Rarity::Epic, 2) | (Rarity::Legendary, 3)
        )
    }).collect();
    let i = (macroquad::prelude::rand::gen_range(0.0, pool.len() as f32)) as usize;
    pool[i.min(pool.len() - 1)]
}

/// Базовый шанс дропа реликвии при уничтожении планеты.
pub const BASE_DROP_CHANCE: f64 = 0.10;
