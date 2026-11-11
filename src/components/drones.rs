//! Дроны-автокликеры: 6 типов с экспоненциальной стоимостью.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum DroneKind {
    Probe,
    Laser,
    Plasma,
    Quantum,
    Singularity,
    Harvester,
}

impl DroneKind {
    pub const ALL: [DroneKind; 6] = [
        DroneKind::Probe,
        DroneKind::Laser,
        DroneKind::Plasma,
        DroneKind::Quantum,
        DroneKind::Singularity,
        DroneKind::Harvester,
    ];

    pub fn idx(self) -> usize {
        match self {
            DroneKind::Probe => 0,
            DroneKind::Laser => 1,
            DroneKind::Plasma => 2,
            DroneKind::Quantum => 3,
            DroneKind::Singularity => 4,
            DroneKind::Harvester => 5,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            DroneKind::Probe => "Разведочный Бур",
            DroneKind::Laser => "Лазерный Майнер",
            DroneKind::Plasma => "Плазменный Бур",
            DroneKind::Quantum => "Квантовый Экскаватор",
            DroneKind::Singularity => "Сингулярная Установка",
            DroneKind::Harvester => "Звёздный Жнец",
        }
    }

    pub fn desc(self) -> &'static str {
        match self {
            DroneKind::Probe => "Медленно грызёт кору, зато почти бесплатен.",
            DroneKind::Laser => "Выжигает породу сфокусированным лучом.",
            DroneKind::Plasma => "Шарики плазмы пробивают мантию насквозь.",
            DroneKind::Quantum => "Добывает руду из суперпозиции разломов.",
            DroneKind::Singularity => "Микро-чёрная дыра на орбите планеты.",
            DroneKind::Harvester => "Собирает руду с самого звёздного ветра.",
        }
    }

    pub fn base_cost(self) -> f64 {
        match self {
            DroneKind::Probe => 15.0,
            DroneKind::Laser => 260.0,
            DroneKind::Plasma => 5_200.0,
            DroneKind::Quantum => 110_000.0,
            DroneKind::Singularity => 2.8e6,
            DroneKind::Harvester => 9.5e7,
        }
    }

    /// Базовый DPS одной единицы (без бонусов реликвий).
    pub fn base_dps(self) -> f64 {
        match self {
            DroneKind::Probe => 0.6,
            DroneKind::Laser => 7.0,
            DroneKind::Plasma => 62.0,
            DroneKind::Quantum => 700.0,
            DroneKind::Singularity => 8_200.0,
            DroneKind::Harvester => 105_000.0,
        }
    }
}

/// Рост стоимости: base * 1.15^level (классика жанра).
pub const COST_GROWTH: f64 = 1.15;

pub fn drone_cost(kind: DroneKind, current_level: u32) -> f64 {
    kind.base_cost() * COST_GROWTH.powi(current_level as i32)
}

/// Суммарная стоимость покупки n уровней подряд.
pub fn drone_bulk_cost(kind: DroneKind, current_level: u32, n: u32) -> f64 {
    let c0 = kind.base_cost() * COST_GROWTH.powi(current_level as i32);
    // Геометрическая сумма: c0 * (r^n - 1) / (r - 1)
    c0 * (COST_GROWTH.powi(n as i32) - 1.0) / (COST_GROWTH - 1.0)
}

/// DPS всех дронов данного типа с учётом уровня и множителей.
pub fn drone_dps(kind: DroneKind, level: u32, mult: f64) -> f64 {
    kind.base_dps() * level as f64 * mult
}
