//! Игровое состояние: валюты, прогресс, дроны, реликвии, престиж, статистика.
//! Всё сериализуется в saves/save.json.

use crate::components::drones::DroneKind;
use crate::components::planet;
use crate::components::relics::{RelicDef, RelicEffect};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Апгрейды «Звёздного Ядра» — покупаются за Тёмную Материю, сохраняются при престиже.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum CoreUpgrade {
    Might,    // +30% урона клика за уровень
    Industry, // +25% DPS дронов за уровень
    Fortune,  // +1% крит-шанс и +15% крит-урона за уровень
    Insight,  // +6% удачи (дроп реликвий) за уровень
    Tempo,    // +5% скорости дронов и +0.25 клика/сек автокликера
}

impl CoreUpgrade {
    pub const ALL: [CoreUpgrade; 5] = [
        CoreUpgrade::Might,
        CoreUpgrade::Industry,
        CoreUpgrade::Fortune,
        CoreUpgrade::Insight,
        CoreUpgrade::Tempo,
    ];

    pub fn idx(self) -> usize {
        match self {
            CoreUpgrade::Might => 0,
            CoreUpgrade::Industry => 1,
            CoreUpgrade::Fortune => 2,
            CoreUpgrade::Insight => 3,
            CoreUpgrade::Tempo => 4,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            CoreUpgrade::Might => "Ядро Мощи",
            CoreUpgrade::Industry => "Ядро Индустрии",
            CoreUpgrade::Fortune => "Ядро Фортуны",
            CoreUpgrade::Insight => "Ядро Прозрения",
            CoreUpgrade::Tempo => "Ядро Темпа",
        }
    }

    pub fn desc(self) -> &'static str {
        match self {
            CoreUpgrade::Might => "Урон клика +30% за ур.",
            CoreUpgrade::Industry => "Сила дронов +25% за ур.",
            CoreUpgrade::Fortune => "Крит-шанс +1%, крит-урон +15% за ур.",
            CoreUpgrade::Insight => "Удача (дроп реликвий) +6% за ур.",
            CoreUpgrade::Tempo => "Скорость дронов +5%, автокликер +0.25/сек",
        }
    }

    pub fn max_level(self) -> u32 {
        match self {
            CoreUpgrade::Might | CoreUpgrade::Industry => 20,
            CoreUpgrade::Fortune => 15,
            CoreUpgrade::Insight => 12,
            CoreUpgrade::Tempo => 10,
        }
    }

    pub fn cost(self, level: u32) -> f64 {
        let base = match self {
            CoreUpgrade::Might => 3.0,
            CoreUpgrade::Industry => 3.0,
            CoreUpgrade::Fortune => 5.0,
            CoreUpgrade::Insight => 4.0,
            CoreUpgrade::Tempo => 6.0,
        };
        let growth = match self {
            CoreUpgrade::Might | CoreUpgrade::Industry => 1.7f64,
            _ => 1.9f64,
        };
        base * growth.powi(level as i32)
    }
}

/// Апгрейды клика за руду (сбрасываются престижем).
pub const CLICK_UPGRADES: [&str; 3] = ["Манипулятор", "Крит-модуль", "Резонатор"];

pub fn click_upgrade_cost(idx: usize, level: u32) -> f64 {
    match idx {
        0 => 20.0 * 1.55f64.powi(level as i32),           // +1 базовый урон
        1 => 150.0 * 2.20f64.powi(level as i32),          // +0.4% крит-шанса (макс 20)
        _ => 120.0 * 2.00f64.powi(level as i32),          // +10% крит-урона
    }
}

pub fn click_upgrade_max(idx: usize) -> u32 {
    match idx {
        1 => 20,
        _ => u32::MAX,
    }
}

/// Агрегированные бонусы (пересчитываются каждый кадр — дёшево).
#[derive(Clone, Copy, Default)]
pub struct Bonuses {
    pub click_mult: f64,
    pub crit_chance: f64,
    pub crit_mult: f64,
    pub ore_mult: f64,
    pub drone_mult: f64,
    pub drone_speed_mult: f64,
    pub luck: f64,
    pub dm_mult: f64,
    pub boss_mult: f64,
    /// Доп. клики/сек от Ядра Темпа
    pub tempo_cps: f64,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Stats {
    pub clicks: u64,
    pub auto_clicks: u64,
    pub total_damage: f64,
    pub planets_destroyed: u32,
    pub bosses_killed: u32,
    pub relics_found: u32,
    pub best_crit: f64,
    pub playtime: f64,
}

/// Главное игровое состояние (сериализуется целиком).
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GameState {
    pub version: u32,
    /// Текущая руда
    pub ore: f64,
    /// Тёмная материя (валюта престижа)
    pub dark_matter: f64,
    /// Руда, добытая за текущий забег (сбрасывается престижем)
    pub run_ore_earned: f64,
    /// Руда за всё время (статистика)
    pub total_ore_all_time: f64,
    /// Номер галактики (бесконечный режим после 8-го биома)
    pub galaxy: u32,
    pub biome: usize,
    pub planet: usize,
    /// Остаток HP текущей планеты
    pub planet_hp: f64,
    /// Остаток щита босса (0 = нет щита или сломан)
    pub shield_hp: f64,
    /// Уровни дронов [Probe, Laser, Plasma, Quantum, Singularity, Harvester]
    pub drones: [u32; 6],
    /// Апгрейды клика [Манипулятор, Крит-модуль, Резонатор]
    pub click_upgrades: [u32; 3],
    /// Найденные реликвии: id -> уровень (кол-во дубликатов)
    pub relics: HashMap<String, u32>,
    /// Число престижей
    pub prestiges: u32,
    /// Уровни Ядра: [Might, Industry, Fortune, Insight, Tempo]
    pub core_upgrades: [u32; 5],
    pub stats: Stats,
}

impl Default for GameState {
    fn default() -> Self {
        let (biome, planet_idx) = (0usize, 0usize);
        let hp = planet::planet_max_hp(biome, planet_idx, 0);
        Self {
            version: crate::save_system::SAVE_VERSION,
            ore: 0.0,
            dark_matter: 0.0,
            run_ore_earned: 0.0,
            total_ore_all_time: 0.0,
            galaxy: 0,
            biome,
            planet: planet_idx,
            planet_hp: hp,
            shield_hp: planet::shield_max_hp(biome, planet_idx, 0),
            drones: [0; 6],
            click_upgrades: [0; 3],
            relics: HashMap::new(),
            prestiges: 0,
            core_upgrades: [0; 5],
            stats: Stats::default(),
        }
    }
}

impl GameState {
    pub fn max_hp(&self) -> f64 {
        planet::planet_max_hp(self.biome, self.planet, self.galaxy)
    }

    pub fn is_boss(&self) -> bool {
        planet::is_boss(self.planet)
    }

    pub fn shield_max(&self) -> f64 {
        planet::shield_max_hp(self.biome, self.planet, self.galaxy)
    }

    /// Базовый урон клика: 1 + 2 * уровень Манипулятора.
    pub fn base_click_damage(&self) -> f64 {
        1.0 + 2.0 * self.click_upgrades[0] as f64
    }

    pub fn relic_level(&self, id: &str) -> u32 {
        *self.relics.get(id).unwrap_or(&0)
    }

    /// Агрегация всех бонусов: реликвии + Ядро + престиж.
    pub fn bonuses(&self) -> Bonuses {
        let mut b = Bonuses {
            click_mult: 1.0,
            crit_chance: 0.05,
            crit_mult: 2.0,
            ore_mult: 1.0,
            drone_mult: 1.0,
            drone_speed_mult: 1.0,
            luck: 0.0,
            dm_mult: 1.0,
            boss_mult: 1.0,
            tempo_cps: 0.0,
        };

        // Реликвии
        for (id, lvl) in &self.relics {
            let lvl = *lvl as f64;
            if let Some(def) = RelicDef::lookup(id) {
                match def.effect {
                    RelicEffect::ClickDmg(v) => b.click_mult += v * lvl,
                    RelicEffect::CritChance(v) => b.crit_chance += v * lvl,
                    RelicEffect::CritDmg(v) => b.crit_mult += v * lvl,
                    RelicEffect::Ore(v) => b.ore_mult += v * lvl,
                    RelicEffect::DronePower(v) => b.drone_mult += v * lvl,
                    RelicEffect::DroneSpeed(v) => b.drone_speed_mult += v * lvl,
                    RelicEffect::Luck(v) => b.luck += v * lvl,
                    RelicEffect::DarkMatter(v) => b.dm_mult += v * lvl,
                    RelicEffect::BossDmg(v) => b.boss_mult += v * lvl,
                }
            }
        }

        // Ядро (престиж-апгрейды)
        b.click_mult *= 1.0 + 0.30 * self.core_upgrades[0] as f64;
        b.drone_mult *= 1.0 + 0.25 * self.core_upgrades[1] as f64;
        b.crit_chance += 0.01 * self.core_upgrades[2] as f64;
        b.crit_mult += 0.15 * self.core_upgrades[2] as f64;
        b.luck += 0.06 * self.core_upgrades[3] as f64;
        b.drone_speed_mult *= 1.0 + 0.05 * self.core_upgrades[4] as f64;
        b.tempo_cps = 0.25 * self.core_upgrades[4] as f64;

        // Престиж: +5% руды за каждый престиж
        b.ore_mult *= 1.0 + 0.05 * self.prestiges as f64;
        b.crit_chance = b.crit_chance.min(0.85);
        b
    }

    /// Доход всех дронов в секунду с бонусами.
    pub fn drones_dps(&self) -> f64 {
        let b = self.bonuses();
        DroneKind::ALL
            .iter()
            .map(|&k| crate::components::drones::drone_dps(k, self.drones[k.idx()], b.drone_mult))
            .sum()
    }

    /// Расчёт Тёмной Материи за престиж.
    pub fn prestige_gain(&self) -> f64 {
        if self.run_ore_earned < 1.0e6 {
            return 0.0;
        }
        let b = self.bonuses();
        let base = 5.0 * (self.run_ore_earned / 1.0e6).powf(0.5);
        (base * b.dm_mult).floor()
    }

    /// Можно ли престижнуться.
    pub fn prestige_ready(&self) -> bool {
        self.prestige_gain() >= 1.0
    }
}

impl RelicDef {
    /// Хелпер для поиска по строковому id из сейва.
    pub fn lookup(id: &str) -> Option<&'static RelicDef> {
        crate::components::relics::relic_by_id(id)
    }
}
