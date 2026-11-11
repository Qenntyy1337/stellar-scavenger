//! Экономика: покупки дронов и апгрейдов клика (x1 / x10 / max).

use crate::components::drones::{drone_bulk_cost, drone_cost, DroneKind};

/// Сколько уровней можно купить на `ore` (максимум 1000 за раз).
pub fn affordable_levels(cost_fn: impl Fn(u32) -> f64, ore: f64, max_levels: u32) -> u32 {
    let mut n = 0u32;
    let mut total = 0.0f64;
    while n < max_levels && n < 1000 {
        let c = cost_fn(n);
        if total + c > ore {
            break;
        }
        total += c;
        n += 1;
    }
    n
}

/// Результат попытки покупки дронов.
#[allow(dead_code)]
pub struct BuyResult {
    pub bought: u32,
    pub spent: f64,
}

pub fn buy_drones(state: &mut crate::resources::game_state::GameState, kind: DroneKind, mode: BuyMode) -> BuyResult {
    let lvl = state.drones[kind.idx()];
    match mode {
        BuyMode::One => {
            let c = drone_cost(kind, lvl);
            if state.ore >= c {
                state.ore -= c;
                state.drones[kind.idx()] += 1;
                BuyResult { bought: 1, spent: c }
            } else {
                BuyResult { bought: 0, spent: 0.0 }
            }
        }
        BuyMode::Many(n) => {
            let c = drone_bulk_cost(kind, lvl, n);
            if state.ore >= c {
                state.ore -= c;
                state.drones[kind.idx()] += n;
                BuyResult { bought: n, spent: c }
            } else {
                BuyResult { bought: 0, spent: 0.0 }
            }
        }
        BuyMode::Max => {
            let n = affordable_levels(|i| drone_cost(kind, lvl + i), state.ore, 1000);
            if n == 0 {
                return BuyResult { bought: 0, spent: 0.0 };
            }
            let c = drone_bulk_cost(kind, lvl, n);
            state.ore -= c;
            state.drones[kind.idx()] += n;
            BuyResult { bought: n, spent: c }
        }
    }
}

/// Режим покупки в магазине.
#[derive(Clone, Copy, PartialEq)]
pub enum BuyMode {
    One,
    Many(u32),
    Max,
}

/// Покупка апгрейда клика (idx 0..2).
pub fn buy_click_upgrade(state: &mut crate::resources::game_state::GameState, idx: usize, mode: BuyMode) -> BuyResult {
    use crate::resources::game_state::{click_upgrade_cost, click_upgrade_max};
    let lvl = state.click_upgrades[idx];
    let cost_of = |i: u32| click_upgrade_cost(idx, lvl + i);
    let max = click_upgrade_max(idx);
    let room = max.saturating_sub(lvl);
    if room == 0 {
        return BuyResult { bought: 0, spent: 0.0 };
    }
    match mode {
        BuyMode::One => {
            let c = cost_of(0);
            if state.ore >= c {
                state.ore -= c;
                state.click_upgrades[idx] += 1;
                BuyResult { bought: 1, spent: c }
            } else {
                BuyResult { bought: 0, spent: 0.0 }
            }
        }
        BuyMode::Many(n) => {
            let n = n.min(room);
            let mut total = 0.0;
            for i in 0..n {
                total += cost_of(i);
            }
            if state.ore >= total {
                state.ore -= total;
                state.click_upgrades[idx] += n;
                BuyResult { bought: n, spent: total }
            } else {
                BuyResult { bought: 0, spent: 0.0 }
            }
        }
        BuyMode::Max => {
            let n = affordable_levels(cost_of, state.ore, room);
            if n == 0 {
                return BuyResult { bought: 0, spent: 0.0 };
            }
            let mut total = 0.0;
            for i in 0..n {
                total += cost_of(i);
            }
            state.ore -= total;
            state.click_upgrades[idx] += n;
            BuyResult { bought: n, spent: total }
        }
    }
}

/// Покупка уровня Звёздного Ядра за Тёмную Материю.
pub fn buy_core(state: &mut crate::resources::game_state::GameState, up: crate::resources::game_state::CoreUpgrade) -> bool {
    let lvl = state.core_upgrades[up.idx()];
    if lvl >= up.max_level() {
        return false;
    }
    let c = up.cost(lvl);
    if state.dark_matter >= c {
        state.dark_matter -= c;
        state.core_upgrades[up.idx()] += 1;
        true
    } else {
        false
    }
}

/// Престиж: сброс забега с конвертацией руды в Тёмную Материю.
/// Реликвии, Ядро и ТМ сохраняются.
pub fn do_prestige(state: &mut crate::resources::game_state::GameState) -> f64 {
    let gain = state.prestige_gain();
    if gain < 1.0 {
        return 0.0;
    }
    state.dark_matter += gain;
    state.prestiges += 1;
    state.ore = 0.0;
    state.run_ore_earned = 0.0;
    state.galaxy = 0;
    state.biome = 0;
    state.planet = 0;
    state.drones = [0; 6];
    state.click_upgrades = [0; 3];
    state.planet_hp = crate::components::planet::planet_max_hp(0, 0, 0);
    state.shield_hp = crate::components::planet::shield_max_hp(0, 0, 0);
    gain
}
