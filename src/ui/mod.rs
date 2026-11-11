//! UI-слой: тема, иконки, виджеты, HUD, магазин, модальные окна, фон.

pub mod background;
pub mod click_area;
pub mod hud;
pub mod icons;
pub mod relics_ui;
pub mod settings_ui;
pub mod prestige_ui;
pub mod shop;
pub mod theme;
pub mod widgets;

use crate::components::drones::DroneKind;
use crate::resources::game_state::CoreUpgrade;
use crate::systems::economy::BuyMode;

/// Действия, запрошенные из UI. Применяются в game.rs после отрисовки.
#[derive(Clone, Copy, PartialEq)]
pub enum UiAction {
    OpenSettings,
    OpenRelics,
    OpenPrestige,
    CloseModal,
    SaveNow,
    ToggleFullscreen,
    BuyDrone(DroneKind, BuyMode),
    BuyClickUpgrade(usize, BuyMode),
    BuyCore(CoreUpgrade),
    DoPrestige,
    /// Открыть админ-панель (иконка в HUD)
    ToggleAdmin,
}

/// Модальные окна.
#[derive(Clone, Copy, PartialEq)]
pub enum Modal {
    None,
    Settings,
    Relics,
    Prestige,
}
