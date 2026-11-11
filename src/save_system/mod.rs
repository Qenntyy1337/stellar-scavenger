//! Система сохранений: JSON в папке `saves/` рядом с игрой.
//!
//! - save.json      — игровой прогресс
//! - settings.json  — настройки (звук, окно, автокликер...)
//! - save.json.bak  — резервная копия последнего успешного сохранения
//!
//! Запись атомарная: сначала во временный файл, затем rename.
//! Загрузка отказоустойчивая: save.json -> save.json.bak -> дефолты.
//! Все поля serde-структур имеют #[serde(default)] — старые сейвы
//! и частично битые файлы подхватываются без краша.

use crate::resources::game_state::GameState;
use crate::resources::settings::Settings;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;

pub const SAVE_VERSION: u32 = 2;

/// Сериализуемый снимок прогресса (version живёт внутри GameState).
#[derive(Serialize, Deserialize)]
pub struct SaveData {
    #[serde(flatten)]
    pub state: GameState,
}

/// Папка сейвов: ./saves относительно рабочей директории игры.
pub fn saves_dir() -> PathBuf {
    PathBuf::from("saves")
}

fn atomic_write(path: &PathBuf, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
    {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    // Бэкап предыдущей версии
    if path.exists() {
        let bak = path.with_extension("json.bak");
        let _ = std::fs::copy(path, &bak);
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Полное сохранение: прогресс + настройки. Возвращает успех.
pub fn save_all(state: &GameState, settings: &Settings) -> bool {
    let data = SaveData { state: state.clone() };
    let ok_progress = match serde_json::to_vec_pretty(&data) {
        Ok(bytes) => atomic_write(&saves_dir().join("save.json"), &bytes).is_ok(),
        Err(_) => false,
    };
    let ok_settings = match serde_json::to_vec_pretty(settings) {
        Ok(bytes) => atomic_write(&Settings::path(), &bytes).is_ok(),
        Err(_) => false,
    };
    ok_progress && ok_settings
}

/// Результат загрузки.
pub struct LoadResult {
    pub state: GameState,
    pub settings: Settings,
    /// true, если сейв не найден (первый запуск)
    pub fresh: bool,
    /// true, если основной сейв битый и загрузились из бэкапа
    pub from_backup: bool,
}

/// Загрузка прогресса и настроек с фолбэками.
pub fn load_all() -> LoadResult {
    let settings = std::fs::read(Settings::path())
        .ok()
        .and_then(|b| serde_json::from_slice::<Settings>(&b).ok())
        .unwrap_or_default();

    let save_path = saves_dir().join("save.json");
    let bak_path = saves_dir().join("save.json.bak");

    let mut from_backup = false;
    let mut raw = std::fs::read(&save_path).ok();
    if raw.is_none() {
        if let Ok(b) = std::fs::read(&bak_path) {
            raw = Some(b);
            from_backup = true;
        }
    }

    match raw {
        Some(bytes) => match serde_json::from_slice::<SaveData>(&bytes) {
            Ok(data) => {
                let mut state = data.state;
                // Миграция версий: заполняем новые поля дефолтами
                let d = GameState::default();
                if state.version < SAVE_VERSION {
                    if state.version < 2 {
                        state.run_ore_earned = state.total_ore_all_time;
                    }
                    state.version = SAVE_VERSION;
                }
                // Страховка: HP не может быть <= 0 после загрузки
                if state.planet_hp <= 0.0 {
                    state.planet_hp = d.planet_hp;
                }
                LoadResult { state, settings, fresh: false, from_backup }
            }
            // Битый основной файл — пробуем бэкап
            Err(_) => {
                if let Ok(b) = std::fs::read(&bak_path) {
                    if let Ok(data) = serde_json::from_slice::<SaveData>(&b) {
                        return LoadResult {
                            state: data.state,
                            settings,
                            fresh: false,
                            from_backup: true,
                        };
                    }
                }
                LoadResult { state: GameState::default(), settings, fresh: true, from_backup: false }
            }
        },
        None => LoadResult { state: GameState::default(), settings, fresh: true, from_backup: false },
    }
}
