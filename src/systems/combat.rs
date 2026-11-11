//! Боевая система: урон кликов, криты, щиты боссов, уничтожение планет.

use crate::components::planet;
use crate::components::relics::{roll_relic, BASE_DROP_CHANCE};
use crate::resources::game_state::Bonuses;
use crate::resources::game_state::GameState;

/// Итог одного клика (для эффектов и звука).
#[allow(dead_code)] // часть полей — информационные, пригодятся в балансных правках
pub struct ClickOutcome {
    pub dmg: f64,
    pub crit: bool,
    /// Урон прошёл по щиту (не по HP)
    pub shield_hit: bool,
    pub shield_broken: bool,
    pub killed: bool,
    /// Руда, добытая с этого удара
    pub ore_gained: f64,
}

/// Полный урон одного клика с учётом бонусов, критов и админ-режимов.
pub fn click_damage(state: &GameState, b: &Bonuses, infinite: bool) -> (f64, bool) {
    if infinite {
        return (f64::MAX, true);
    }
    let base = state.base_click_damage();
    let mut dmg = base * b.click_mult;
    if state.is_boss() {
        dmg *= b.boss_mult;
    }
    let crit = (macroquad::prelude::rand::gen_range(0.0, 1.0) as f64) < b.crit_chance;
    if crit {
        dmg *= b.crit_mult;
    }
    (dmg, crit)
}

/// Применение урона к планете/щиту. Возвращает исход для визуала.
pub fn apply_click(
    state: &mut GameState,
    b: &Bonuses,
    dmg: f64,
    crit: bool,
    god: bool,
) -> ClickOutcome {
    let mut dmg = dmg;
    if god {
        dmg = state.max_hp() + state.shield_max() + 1.0;
    }
    let mut shield_hit = false;
    let mut shield_broken = false;

    // Руда с каждого удара — небольшая доля урона
    let ore_gained = if dmg >= f64::MAX { 0.0 } else { dmg * 0.06 * b.ore_mult };
    state.ore += ore_gained;
    state.run_ore_earned += ore_gained;
    state.total_ore_all_time += ore_gained;

    if state.shield_hp > 0.0 {
        shield_hit = true;
        state.shield_hp -= dmg;
        if state.shield_hp <= 0.0 {
            state.shield_hp = 0.0;
            shield_broken = true;
        }
    } else {
        state.planet_hp -= dmg;
        state.stats.total_damage += dmg.min(1e300);
    }

    let killed = state.planet_hp <= 0.0 && state.shield_hp <= 0.0;
    if killed {
        state.planet_hp = 0.0;
    }

    ClickOutcome { dmg, crit, shield_hit, shield_broken, killed, ore_gained }
}

/// Отчёт об уничтожении планеты (для анимаций/тостов).
#[allow(dead_code)]
pub struct DestroyReport {
    pub ore: f64,
    pub relic: Option<&'static crate::components::relics::RelicDef>,
    pub was_boss: bool,
    pub biome_cleared: bool,
    pub galaxy_cleared: bool,
    pub new_biome: usize,
    pub new_planet: usize,
}

/// Уничтожение планеты: награда, ролл реликвии, переход к следующей планете.
pub fn destroy_planet(state: &mut GameState) -> DestroyReport {
    let b = state.bonuses();
    let was_boss = state.is_boss();
    let reward = planet::planet_ore_reward(state.biome, state.planet, state.galaxy) * b.ore_mult;
    state.ore += reward;
    state.run_ore_earned += reward;
    state.total_ore_all_time += reward;
    state.stats.planets_destroyed += 1;
    if was_boss {
        state.stats.bosses_killed += 1;
    }

    // Ролл реликвии: у боссов шанс выше
    let depth = (state.biome * planet::PLANETS_PER_BIOME + state.planet) as f64;
    let mut chance = BASE_DROP_CHANCE + b.luck * 0.02;
    if was_boss {
        chance += 0.25 + b.luck * 0.05;
    }
    let relic = if (macroquad::prelude::rand::gen_range(0.0, 1.0) as f64) < chance {
        let def = roll_relic(b.luck, depth);
        let lvl = state.relics.entry(def.id.to_string()).or_insert(0);
        *lvl += 1;
        state.stats.relics_found += 1;
        Some(def)
    } else {
        None
    };

    let biome_cleared = state.planet >= planet::PLANETS_PER_BIOME - 1;
    let galaxy_cleared = biome_cleared && state.biome >= planet::BIOMES.len() - 1;

    let (new_biome, new_planet);
    if galaxy_cleared {
        // Бесконечный режим: новая галактика, всё заново, но жирнее
        state.galaxy += 1;
        new_biome = 0;
        new_planet = 0;
    } else if biome_cleared {
        new_biome = state.biome + 1;
        new_planet = 0;
    } else {
        new_biome = state.biome;
        new_planet = state.planet + 1;
    }
    state.biome = new_biome;
    state.planet = new_planet;
    state.planet_hp = planet::planet_max_hp(new_biome, new_planet, state.galaxy);
    state.shield_hp = planet::shield_max_hp(new_biome, new_planet, state.galaxy);

    DestroyReport { ore: reward, relic, was_boss, biome_cleared, galaxy_cleared, new_biome, new_planet }
}

/// Регенерация щита босса, если долго не бить.
pub fn shield_regen(state: &mut GameState, since_hit: f32, dt: f32) {
    let smax = state.shield_max();
    if smax > 0.0 && state.shield_hp > 0.0 && since_hit > 6.0 {
        state.shield_hp = (state.shield_hp + smax * 0.08 * dt as f64).min(smax);
    }
}
