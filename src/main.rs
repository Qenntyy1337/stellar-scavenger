//! Точка входа Stellar Scavenger: Core & Relics.
//!
//! Управление:
//!   ЛКМ           — удар по планете
//!   F12 или ~     — админ-панель
//!   F11           — полноэкранный режим
//!   M             — mute
//!   Ctrl+S        — сохранить
//!   Esc           — закрыть окно/меню, повторное — выход с сохранением

mod admin_panel;
mod components;
mod game;
mod resources;
mod save_system;
mod systems;
mod ui;
mod util;

use macroquad::prelude::*;

/// Конфиг окна читает сохранённые настройки ДО создания окна
/// (разрешение и фуллскрин применяются сразу, а не со второго запуска).
fn window_conf() -> Conf {
    let s = resources::settings::preload();
    Conf {
        window_title: "Stellar Scavenger: Core & Relics".to_owned(),
        window_width: s.res_w as i32,
        window_height: s.res_h as i32,
        fullscreen: s.fullscreen,
        high_dpi: true,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    println!("Stellar Scavenger: Core & Relics v1.0.0");
    println!("Управление: ЛКМ - удар | F12/~ - админка | F11 - фуллскрин | M - mute | Ctrl+S - сейв | Esc - выход");
    println!("Аудио: {}", if std::env::var("STELLAR_NO_AUDIO").map(|v| v == "1").unwrap_or(false) { "выключено (STELLAR_NO_AUDIO=1)" } else { "включено (процедурный синтез)" });

    let mut game = game::Game::new().await;

    // Применяем полноэкранный режим, если он не совпал с конфигом окна
    if game.settings.fullscreen {
        macroquad::window::set_fullscreen(true);
    }

    loop {
        game.handle_keys();
        let dt = get_frame_time();
        game.update(dt);
        game.draw();
        game.smoke_tick();

        next_frame().await;
    }
}
