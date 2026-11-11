//! Настройки игры: звук, окно, анимации, автокликер.
//! Хранятся в saves/settings.json (serde JSON с дефолтами).

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    /// Общая громкость 0..1
    pub master_volume: f32,
    /// Громкость музыки 0..1
    pub music_volume: f32,
    /// Громкость эффектов 0..1
    pub sfx_volume: f32,
    /// Полноэкранный режим
    pub fullscreen: bool,
    /// Разрешение окна
    pub res_w: u32,
    pub res_h: u32,
    /// Показывать FPS в углу
    pub show_fps: bool,
    /// Множитель скорости анимаций (0.5x .. 2x)
    pub animation_speed: f32,
    /// Встроенный автокликер (тумблер в UI)
    pub autoclicker_enabled: bool,
    /// Частота встроенного автокликера, кликов/сек
    pub autoclicker_rate: f32,
    /// Быстрое переключение mute (клавиша M), сохраняется
    pub muted: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            master_volume: 0.8,
            music_volume: 0.5,
            sfx_volume: 0.9,
            fullscreen: false,
            res_w: 1280,
            res_h: 720,
            show_fps: false,
            animation_speed: 1.0,
            autoclicker_enabled: false,
            autoclicker_rate: 5.0,
            muted: false,
        }
    }
}

impl Settings {
    /// Итоговая громкость эффектов с учётом mute.
    pub fn effective_sfx(&self) -> f32 {
        if self.muted { 0.0 } else { (self.master_volume * self.sfx_volume).clamp(0.0, 1.0) }
    }

    /// Итоговая громкость музыки с учётом mute.
    pub fn effective_music(&self) -> f32 {
        if self.muted { 0.0 } else { (self.master_volume * self.music_volume).clamp(0.0, 1.0) }
    }

    /// Путь к файлу настроек.
    pub fn path() -> std::path::PathBuf {
        crate::save_system::saves_dir().join("settings.json")
    }
}

static PRELOADED: OnceLock<Settings> = OnceLock::new();

/// Синхронная предзагрузка настроек для window_conf()
/// (нужно ДО создания окна, чтобы применить разрешение/фуллскрин).
/// При любой ошибке чтения возвращаются дефолты.
pub fn preload() -> &'static Settings {
    PRELOADED.get_or_init(|| {
        match std::fs::read(Settings::path()) {
            Ok(bytes) => serde_json::from_slice::<Settings>(&bytes).unwrap_or_default(),
            Err(_) => Settings::default(),
        }
    })
}
