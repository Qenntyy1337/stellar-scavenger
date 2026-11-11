//! Ассеты: TTF-шрифты (вкомпилированы в бинарь) + процедурные звуки.

use macroquad::audio::Sound;
use macroquad::text::{load_ttf_font_from_bytes, Font};

/// Банк SFX. Все звуки синтезируются кодом при старте (без внешних файлов).
pub struct SfxBank {
    pub click: Option<Sound>,
    pub crit: Option<Sound>,
    pub buy: Option<Sound>,
    pub relic: Option<Sound>,
    pub destroy: Option<Sound>,
    pub boss: Option<Sound>,
    pub ui: Option<Sound>,
    pub error: Option<Sound>,
    pub prestige: Option<Sound>,
    /// Зацикленная эмбиент-музыка
    pub music: Option<Sound>,
}

/// Все статические ассеты игры.
pub struct Assets {
    pub font_regular: Font,
    pub font_bold: Font,
    pub sfx: SfxBank,
    /// Аудио выключено (переменная окружения STELLAR_NO_AUDIO=1)
    pub audio_enabled: bool,
}

impl Assets {
    pub async fn load() -> Assets {
        // Шрифты вкомпилированы include_bytes! — бинарь самодостаточен
        // и корректно собирается из чистого клона репозитория.
        let font_regular = load_ttf_font_from_bytes(include_bytes!(
            "../../assets/fonts/Inter-Regular.ttf"
        ))
        .expect("assets/fonts/Inter-Regular.ttf должен вкомпилироваться в бинарь");
        let font_bold =
            load_ttf_font_from_bytes(include_bytes!("../../assets/fonts/Inter-Bold.ttf"))
                .expect("assets/fonts/Inter-Bold.ttf должен вкомпилироваться в бинарь");
        let audio_enabled = std::env::var("STELLAR_NO_AUDIO").map(|v| v != "1").unwrap_or(true);

        let sfx = if audio_enabled {
            crate::systems::audio_system::synth_all().await
        } else {
            SfxBank {
                click: None, crit: None, buy: None, relic: None, destroy: None,
                boss: None, ui: None, error: None, prestige: None, music: None,
            }
        };

        Assets { font_regular, font_bold, sfx, audio_enabled }
    }
}
