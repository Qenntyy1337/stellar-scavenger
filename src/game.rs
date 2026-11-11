//! Игровой цикл и оркестрация всех систем.
//!
//! Кадр = update (логика) + draw (немедленный UI, ввод обрабатывается тут же)
//! + process_actions (мутации, накопленные UI-слоями).

use crate::admin_panel::{self, AdminAction, AdminState};
use crate::components::drones::DroneKind;
use crate::components::planet;
use crate::resources::assets::Assets;
use crate::resources::game_state::GameState;
use crate::resources::settings::Settings;
use crate::save_system;
use crate::systems::combat;
use crate::systems::particles_system::Fx;
use crate::systems::planet_render::PlanetRenderer;
use crate::ui::background::Background;
use crate::ui::click_area::{self, BeamPulse, PlayField};
use crate::ui::hud::Hud;
use crate::ui::prestige_ui::PrestigeUi;
use crate::ui::relics_ui;
use crate::ui::settings_ui::{self, SettingsAction};
use crate::ui::shop::Shop;
use crate::ui::theme::col;
use crate::ui::widgets::{self, round_rect, round_rect_lines};
use crate::ui::{Modal, UiAction};
use crate::util::{clamp01, smooth, Rect, SmoothBar, Spring};
use macroquad::prelude::*;

/// Баннер смены биома/галактики.
pub struct Transition {
    t: f32,
    dur: f32,
    title: String,
    subtitle: String,
}

/// Всплывающее уведомление внизу экрана.
pub struct Toast {
    text: String,
    t: f32,
    color: Color,
}

pub struct Game {
    pub assets: Assets,
    pub state: GameState,
    pub settings: Settings,
    pub fx: Fx,
    pub background: Background,
    pub planet: PlanetRenderer,
    pub shop: Shop,
    pub hud: Hud,
    pub prestige_ui: PrestigeUi,
    pub admin: AdminState,
    pub modal: Modal,
    pub reset_armed: bool,

    // Визуальное состояние планеты
    pub pulse: Spring,
    pub hp_bar: SmoothBar,
    pub shield_bar: SmoothBar,
    pub beams: Vec<BeamPulse>,
    pub hover_planet: bool,

    // Таймеры
    pub autosave_timer: f32,
    pub save_soon: f32,
    pub autoclick_acc: f32,
    pub drone_acc: [f32; 6],
    pub since_shield_hit: f32,

    // Служебное
    pub toasts: Vec<Toast>,
    pub transition: Option<Transition>,
    pub last_music_vol: f32,
    pub fps_smooth: f32,
    pub ui_regions: Vec<Rect>,
    pub frame_count: u64,
    pub smoke_frames: Option<u64>,
}

impl Game {
    pub async fn new() -> Game {
        let loaded = save_system::load_all();
        let state = loaded.state;
        if loaded.from_backup {
            println!("[save] Основной сейв повреждён, загружен бэкап save.json.bak");
        }
        if loaded.fresh {
            println!("[save] Сейв не найден — новая игра. Файлы: ./saves/");
        }

        let assets = Assets::load().await;
        let mut settings = loaded.settings;
        // Сглаживание шумных значений из старых настроек
        settings.animation_speed = settings.animation_speed.clamp(0.5, 2.0);
        settings.autoclicker_rate = settings.autoclicker_rate.clamp(1.0, 20.0);

        let seed = Self::planet_seed(&state);
        let mut planet_r = PlanetRenderer::new();
        planet_r.generate(planet::biome(state.biome), seed);

        let hp_frac = (state.planet_hp / state.max_hp()) as f32;
        let sh_frac = if state.shield_max() > 0.0 {
            (state.shield_hp / state.shield_max()) as f32
        } else {
            0.0
        };

        let smoke_frames = std::env::var("STELLAR_SMOKE_FRAMES")
            .ok()
            .and_then(|v| v.parse::<u64>().ok());

        let mut game = Game {
            assets,
            state,
            settings,
            fx: Fx::new(),
            background: Background::new(),
            planet: planet_r,
            shop: Shop::new(),
            hud: Hud::new(),
            prestige_ui: PrestigeUi::new(),
            admin: AdminState::default(),
            modal: Modal::None,
            reset_armed: false,
            pulse: Spring::new(0.0, 130.0, 9.0),
            hp_bar: SmoothBar::new(hp_frac),
            shield_bar: SmoothBar::new(sh_frac),
            beams: Vec::new(),
            hover_planet: false,
            autosave_timer: 0.0,
            save_soon: 0.0,
            autoclick_acc: 0.0,
            drone_acc: [0.0; 6],
            since_shield_hit: 0.0,
            toasts: Vec::new(),
            transition: None,
            last_music_vol: -1.0,
            fps_smooth: 60.0,
            ui_regions: Vec::new(),
            frame_count: 0,
            smoke_frames,
        };

        if game.assets.audio_enabled {
            if let Some(music) = &game.assets.sfx.music {
                macroquad::audio::play_sound(
                    music,
                    macroquad::audio::PlaySoundParams { looped: true, volume: 0.0 },
                );
                game.last_music_vol = 0.0; // объём выставится в первом кадре
            }
        }

        game.push_toast("Добро пожаловать, scavenger! Кликай по планете.".to_string(), col::accent());
        game
    }

    /// Детерминированный сид планеты.
    fn planet_seed(state: &GameState) -> u64 {
        (state.biome as u64) * 10_007
            + (state.planet as u64) * 331
            + (state.galaxy as u64) * 999_91
            + 1
    }

    pub fn push_toast(&mut self, text: String, color: Color) {
        if self.toasts.len() > 5 {
            self.toasts.remove(0);
        }
        self.toasts.push(Toast { text, t: 0.0, color });
    }

    /// Перегенерация планеты под текущее состояние (биом/сид/бары).
    pub fn refresh_planet(&mut self) {
        let seed = Self::planet_seed(&self.state);
        let biome = planet::biome(self.state.biome);
        self.planet.generate(biome, seed);
        self.hp_bar.set((self.state.planet_hp / self.state.max_hp()) as f32);
        let smax = self.state.shield_max();
        self.shield_bar.set(if smax > 0.0 { (self.state.shield_hp / smax) as f32 } else { 0.0 });
        self.beams.clear();
    }

    // ======================= UPDATE =======================

    pub fn update(&mut self, dt_raw: f32) {
        let dt = dt_raw.min(0.05);
        let anim = self.settings.animation_speed;
        let game_dt = dt * self.admin.time_scale;

        self.frame_count += 1;
        self.fps_smooth = smooth(self.fps_smooth, 1.0 / dt.max(1e-4), 4.0, dt);
        self.state.stats.playtime += dt as f64;

        // Фон и планета
        self.background.update(dt, anim);
        if self.planet.wants_update() {
            self.planet.update(game_dt, anim);
        }
        self.pulse.update(dt);

        // Частицы/тексты (анимации UI-скорости)
        self.fx.update(dt * anim);

        // Плавные бары
        let hp_target = clamp01((self.state.planet_hp / self.state.max_hp()) as f32);
        self.hp_bar.update(hp_target, dt);
        let smax = self.state.shield_max();
        let sh_target = if smax > 0.0 && self.state.shield_hp > 0.0 {
            clamp01((self.state.shield_hp / smax) as f32)
        } else {
            0.0
        };
        self.shield_bar.update(sh_target, dt);

        // --- Геймплей (с учётом time scale) ---
        self.update_drones(game_dt);
        self.update_autoclicker(game_dt);
        self.since_shield_hit += game_dt;
        combat::shield_regen(&mut self.state, self.since_shield_hit, game_dt);

        // --- Таймеры ---
        self.autosave_timer += dt;
        if self.autosave_timer >= 10.0 {
            self.autosave_timer = 0.0;
            self.do_save(true);
        }
        if self.save_soon > 0.0 {
            self.save_soon -= dt;
            if self.save_soon <= 0.0 {
                self.do_save(false);
            }
        }

        // Тосты
        for t in self.toasts.iter_mut() {
            t.t += dt;
        }
        self.toasts.retain(|t| t.t < 3.4);

        // Баннер перехода
        if let Some(tr) = &mut self.transition {
            tr.t += dt * anim;
            if tr.t >= tr.dur {
                self.transition = None;
            }
        }

        // Мигание лучей дронов
        for b in self.beams.iter_mut() {
            b.t -= dt * 3.5;
        }
        self.beams.retain(|b| b.t > 0.0);

        // Музыка: громкость
        if self.assets.audio_enabled {
            let vol = self.settings.effective_music() * 0.9;
            if (vol - self.last_music_vol).abs() > 0.005 {
                if let Some(m) = &self.assets.sfx.music {
                    macroquad::audio::set_sound_volume(m, vol);
                }
                self.last_music_vol = vol;
            }
        }

        // Магазин: мигание ошибок
        self.shop.update(dt);
    }

    /// Дроны: DPS + визуальные импульсы лучей.
    fn update_drones(&mut self, dt: f32) {
        let b = self.state.bonuses();
        let dps = self.state.drones_dps();
        if dps > 0.0 {
            if self.state.shield_hp > 0.0 {
                // Дроны на 25% эффективны против щита
                self.state.shield_hp -= (dps * 0.25 * dt as f64).min(self.state.shield_hp);
            } else {
                self.state.planet_hp -= dps * dt as f64;
                self.state.stats.total_damage += dps * dt as f64;
                // Руда пассивно
                let ore = dps * dt as f64 * 0.06 * b.ore_mult;
                self.state.ore += ore;
                self.state.run_ore_earned += ore;
                self.state.total_ore_all_time += ore;
            }
            if self.state.planet_hp <= 0.0 && self.state.shield_hp <= 0.0 {
                self.on_planet_destroyed();
            }
        }
        // Лучи
        let speed = b.drone_speed_mult as f32;
        for (ki, &lvl) in self.state.drones.iter().enumerate() {
            if lvl == 0 {
                continue;
            }
            self.drone_acc[ki] += dt * speed;
            if self.drone_acc[ki] >= 1.0 {
                self.drone_acc[ki] -= 1.0;
                if self.beams.len() < 24 {
                    self.beams.push(BeamPulse {
                        kind: ki,
                        angle: rand::gen_range(0.0, std::f32::consts::TAU),
                        t: 1.0,
                    });
                }
            }
        }
    }

    /// Встроенный автокликер: тумблер + частота из настроек.
    fn update_autoclicker(&mut self, dt: f32) {
        if !self.settings.autoclicker_enabled {
            return;
        }
        let b = self.state.bonuses();
        let cps = ((self.settings.autoclicker_rate + b.tempo_cps as f32) * b.drone_speed_mult as f32).min(40.0);
        self.autoclick_acc += dt * cps;
        let field = PlayField::compute();
        let mut clicks = 0u32;
        while self.autoclick_acc >= 1.0 && clicks < 8 {
            self.autoclick_acc -= 1.0;
            clicks += 1;
            let (x, y) = field.random_point();
            self.do_click(x, y, true);
        }
    }

    // ======================= КЛИК =======================

    /// Полный пайплайн клика (мышь или автокликер).
    pub fn do_click(&mut self, x: f32, y: f32, auto: bool) {
        let b = self.state.bonuses();
        let (dmg, crit) = combat::click_damage(&self.state, &b, self.admin.infinite_damage);
        let outcome = combat::apply_click(&mut self.state, &b, dmg, crit, self.admin.god);

        if auto {
            self.state.stats.auto_clicks += 1;
        } else {
            self.state.stats.clicks += 1;
        }
        if outcome.crit && outcome.dmg < 1e300 {
            self.state.stats.best_crit = self.state.stats.best_crit.max(outcome.dmg);
        }

        // Визуал
        let dmg_disp = if outcome.dmg > 1e299 {
            "INF".to_string()
        } else {
            crate::util::fmt_num(outcome.dmg)
        };
        let size = if crit { 26.0 } else if auto { 15.0 } else { 19.0 };
        let tint = if outcome.shield_hit { col::shield() } else if crit { col::crit() } else { Color::new(1.0, 0.95, 0.85, 1.0) };
        let label = if crit { format!("{} КРИТ", dmg_disp) } else { dmg_disp };
        self.fx.push_text(x, y - 10.0, label, tint, size, crit);

        let pc = col::ore();
        self.fx.cone(x, y, -std::f32::consts::FRAC_PI_2, 0.9, if crit { 14 } else { 7 }, if crit { 240.0 } else { 150.0 }, pc, Color::new(pc.r, pc.g, pc.b, 0.0));

        self.pulse.impulse(if crit { 26.0 } else { 12.0 } / 60.0);
        self.fx.add_shake(if crit { 4.5 } else { 1.6 });

        // Звук
        if self.assets.audio_enabled {
            let vol = self.settings.effective_sfx();
            if vol > 0.0 {
                let snd = if outcome.shield_hit {
                    &self.assets.sfx.boss
                } else if crit {
                    &self.assets.sfx.crit
                } else {
                    &self.assets.sfx.click
                };
                if let Some(s) = snd.as_ref() {
                    macroquad::audio::play_sound(
                        s,
                        macroquad::audio::PlaySoundParams { looped: false, volume: vol },
                    );
                }
            }
        }

        // Слом щита — отдельное событие
        if outcome.shield_broken {
            self.fx.add_shake(9.0);
            self.fx.ring(x, y, 20.0, col::shield());
            self.push_toast("Щит босса разрушен!".to_string(), col::shield());
        }

        if outcome.killed {
            self.on_planet_destroyed();
        }
    }

    /// Уничтожение планеты: награды, реликвия, взрыв, переход.
    fn on_planet_destroyed(&mut self) {
        let report = combat::destroy_planet(&mut self.state);

        // Взрыв
        let field = PlayField::compute();
        let c = col::ore();
        self.fx.burst(field.cx, field.cy, 130, 340.0, 3.5, Color::new(1.0, 0.85, 0.4, 1.0), Color::new(1.0, 0.3, 0.05, 0.0), crate::components::particles::ParticleKind::Spark, 0.9);
        self.fx.burst(field.cx, field.cy, 60, 180.0, 2.5, c, Color::new(c.r, c.g, c.b, 0.0), crate::components::particles::ParticleKind::Streak, 1.1);
        self.fx.ring(field.cx, field.cy, 30.0, Color::new(1.0, 0.9, 0.6, 0.9));
        self.fx.add_shake(18.0);
        self.pulse.impulse(2.2);

        // Награда
        self.fx.push_text(
            field.cx,
            field.cy,
            format!("+{} руды", crate::util::fmt_num(report.ore)),
            col::ore(),
            22.0,
            true,
        );

        // Звук взрыва
        if self.assets.audio_enabled {
            let vol = self.settings.effective_sfx();
            if let Some(s) = self.assets.sfx.destroy.as_ref() {
                macroquad::audio::play_sound(s, macroquad::audio::PlaySoundParams { looped: false, volume: vol });
            }
        }

        // Реликвия
        if let Some(def) = report.relic {
            self.push_toast(
                format!("Реликвия найдена: {} ({})", def.name, def.rarity.name()),
                Color::from_rgba(def.rarity.rgb()[0], def.rarity.rgb()[1], def.rarity.rgb()[2], 255),
            );
            self.fx.push_text(field.cx, field.cy - 40.0, def.name.to_string(), Color::from_rgba(def.rarity.rgb()[0], def.rarity.rgb()[1], def.rarity.rgb()[2], 255), 18.0, true);
            if self.assets.audio_enabled {
                if let Some(s) = self.assets.sfx.relic.as_ref() {
                    let vol = self.settings.effective_sfx();
                    macroquad::audio::play_sound(s, macroquad::audio::PlaySoundParams { looped: false, volume: vol });
                }
            }
        }

        // Баннер перехода
        if report.galaxy_cleared {
            self.transition = Some(Transition {
                t: 0.0,
                dur: 2.6,
                title: format!("ГАЛАКТИКА ОЧИЩЕНА! Переход в галактику +{}", self.state.galaxy),
                subtitle: "Враги стали в 20 раз мощнее. Награды - в 8 раз щедрее.".to_string(),
            });
            if self.assets.audio_enabled {
                if let Some(s) = self.assets.sfx.prestige.as_ref() {
                    let vol = self.settings.effective_sfx();
                    macroquad::audio::play_sound(s, macroquad::audio::PlaySoundParams { looped: false, volume: vol });
                }
            }
        } else if report.biome_cleared {
            let nb = planet::biome(report.new_biome);
            self.transition = Some(Transition {
                t: 0.0,
                dur: 2.2,
                title: format!("НОВЫЙ БИОМ: {}", nb.name.to_uppercase()),
                subtitle: nb.desc.to_string(),
            });
        }

        self.refresh_planet();
        self.save_soon = 1.2; // быстрый отложенный сейв после события
    }

    pub fn do_save(&mut self, auto: bool) {
        let ok = save_system::save_all(&self.state, &self.settings);
        if ok {
            if auto {
                // Автосейв не спамим тостами — показываем каждый пятый
                if (self.frame_count / 600) % 2 == 0 {
                    // тихо
                }
            } else {
                self.push_toast("Игра сохранена".to_string(), col::success());
            }
        } else {
            self.push_toast("ОШИБКА сохранения!".to_string(), col::danger());
        }
    }

    // ======================= DRAW =======================

    pub fn draw(&mut self) {
        let (sw, sh) = (screen_width(), screen_height());
        self.ui_regions.clear();

        self.background.draw(sw, sh);

        // Поле планеты (со смещением тряски)
        let mut field = PlayField::compute();
        field.cx += self.fx.shake_x;
        field.cy += self.fx.shake_y;

        // Hover планеты
        let (mx, my) = mouse_position();
        let base_field = PlayField::compute();
        self.hover_planet = base_field.hit(mx, my) && self.modal == Modal::None;

        click_area::draw_scene(
            &self.assets,
            &self.planet,
            &self.fx,
            &self.state,
            &field,
            self.pulse.value,
            &self.beams,
            self.hover_planet,
            self.hp_bar.value,
            self.hp_bar.ghost.max(self.hp_bar.value),
            self.shield_bar.value,
        );

        // --- HUD
        let (top_rect, top_actions) = self.hud.draw_top(&self.assets, &self.state);
        self.ui_regions.push(top_rect);
        let mut actions: Vec<UiAction> = top_actions;

        let (shop_rect, shop_actions) = self.shop.draw(&self.assets, &mut self.state);
        self.ui_regions.push(shop_rect);
        actions.extend(shop_actions);

        // Звук ошибки при неудачной покупке
        if self.shop.buy_failed {
            self.shop.clear_error();
            if self.assets.audio_enabled {
                if let Some(s) = self.assets.sfx.error.as_ref() {
                    let vol = self.settings.effective_sfx() * 0.8;
                    macroquad::audio::play_sound(s, macroquad::audio::PlaySoundParams { looped: false, volume: vol });
                }
            }
        }

        let bottom_rect = self.hud.draw_bottom(&self.assets, &self.state, &mut self.settings, self.fps_smooth);
        self.ui_regions.push(bottom_rect);

        // --- Подсказка первого запуска
        if self.state.stats.clicks < 6 {
            let t = get_time() as f32;
            let alpha = 0.55 + 0.45 * (t * 2.5).sin();
            widgets::text_center(
                &self.assets,
                "Кликай по планете, чтобы добывать руду!",
                base_field.cx,
                base_field.cy - base_field.radius - 64.0,
                15.0,
                Color::new(1.0, 1.0, 1.0, alpha),
            );
        }

        // --- Модальные окна
        match self.modal {
            Modal::Settings => {
                self.ui_regions.push(Rect::new(0.0, 0.0, sw, sh));
                let acts = settings_ui::draw(&self.assets, &mut self.settings, &self.state, &mut self.reset_armed);
                for a in acts {
                    match a {
                        SettingsAction::Ui(u) => actions.push(u),
                        SettingsAction::ResetProgress => {
                            self.state = GameState::default();
                            self.refresh_planet();
                            self.push_toast("Прогресс сброшен".to_string(), col::danger());
                        }
                    }
                }
            }
            Modal::Relics => {
                self.ui_regions.push(Rect::new(0.0, 0.0, sw, sh));
                let acts = relics_ui::draw(&self.assets, &self.state);
                actions.extend(acts);
            }
            Modal::Prestige => {
                self.ui_regions.push(Rect::new(0.0, 0.0, sw, sh));
                let acts = self.prestige_ui.draw(&self.assets, &mut self.state);
                actions.extend(acts);
            }
            Modal::None => {}
        }

        // --- Админ-панель
        if self.admin.open {
            let (arect, admin_actions) = admin_panel::draw(&self.assets, &mut self.admin, &mut self.state);
            self.ui_regions.push(arect);
            for act in admin_actions {
                self.process_admin_action(act);
            }
        }

        // --- Клик по планете (если не перекрыт UI)
        let over_ui = self.ui_regions.iter().any(|r| r.contains(mx, my));
        if !over_ui
            && base_field.hit(mx, my)
            && self.modal == Modal::None
            && is_mouse_button_pressed(MouseButton::Left)
        {
            self.do_click(mx, my, false);
        }

        // --- Тосты
        for (i, t) in self.toasts.iter().enumerate() {
            let alpha = if t.t < 0.25 { t.t / 0.25 } else if t.t > 2.8 { ((3.4 - t.t) / 0.6).clamp(0.0, 1.0) } else { 1.0 };
            let y = sh - crate::ui::theme::BOTTOM_BAR_H - 14.0 - (self.toasts.len() - 1 - i) as f32 * 30.0;
            let size = 13.0;
            let w = widgets::text_w(&self.assets, &t.text, size) + 20.0;
            let x = (sw - w) / 2.0;
            let c = Color::new(t.color.r, t.color.g, t.color.b, alpha);
            round_rect(x, y - 22.0, w, 26.0, 13.0, Color::new(0.03, 0.05, 0.1, 0.85 * alpha));
            round_rect_lines(x, y - 22.0, w, 26.0, 13.0, 1.0, Color::new(c.r, c.g, c.b, 0.5 * alpha));
            widgets::text_center(&self.assets, &t.text, sw / 2.0, y - 16.0, size, c);
        }

        // --- Баннер перехода
        if let Some(tr) = &self.transition {
            let p = tr.t / tr.dur;
            let alpha = if p < 0.15 { p / 0.15 } else if p > 0.8 { ((1.0 - p) / 0.2).clamp(0.0, 1.0) } else { 1.0 };
            draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.55 * alpha));
            widgets::text_center_b(&self.assets, &tr.title, sw / 2.0, sh / 2.0 - 10.0, 30.0, Color::new(col::accent().r, col::accent().g, col::accent().b, alpha));
            widgets::text_center(&self.assets, &tr.subtitle, sw / 2.0, sh / 2.0 + 26.0, 14.0, Color::new(col::text().r, col::text().g, col::text().b, 0.9 * alpha));
        }

        // --- Обработка UI-действий (после отрисовки, чтобы не путать borrow'и)
        for act in actions {
            self.process_ui_action(act);
        }
    }

    fn process_ui_action(&mut self, act: UiAction) {
        match act {
            UiAction::OpenSettings => {
                self.modal = Modal::Settings;
                self.reset_armed = false;
                self.play_ui_sound();
            }
            UiAction::OpenRelics => {
                self.modal = Modal::Relics;
                self.play_ui_sound();
            }
            UiAction::OpenPrestige => {
                self.modal = Modal::Prestige;
                self.play_ui_sound();
            }
            UiAction::CloseModal => {
                self.modal = Modal::None;
                self.reset_armed = false;
                self.play_ui_sound();
            }
            UiAction::SaveNow => self.do_save(false),
            UiAction::ToggleFullscreen => self.apply_fullscreen(),
            UiAction::BuyDrone(_, _) | UiAction::BuyClickUpgrade(_, _) => {
                if self.assets.audio_enabled {
                    if let Some(s) = self.assets.sfx.buy.as_ref() {
                        let vol = self.settings.effective_sfx();
                        macroquad::audio::play_sound(s, macroquad::audio::PlaySoundParams { looped: false, volume: vol });
                    }
                }
            }
            UiAction::BuyCore(_) => {
                self.push_toast("Ядро усилено!".to_string(), col::dark_matter());
                self.play_ui_sound();
            }
            UiAction::DoPrestige => {
                self.refresh_planet();
                self.push_toast("ПРЕСТИЖ! Ядро поглотило галактику".to_string(), col::dark_matter());
                if self.assets.audio_enabled {
                    if let Some(s) = self.assets.sfx.prestige.as_ref() {
                        let vol = self.settings.effective_sfx();
                        macroquad::audio::play_sound(s, macroquad::audio::PlaySoundParams { looped: false, volume: vol });
                    }
                }
            }
            UiAction::ToggleAdmin => admin_panel::toggle(&mut self.admin),
        }
    }

    fn process_admin_action(&mut self, act: AdminAction) {
        match act {
            AdminAction::GiveOre(v) => {
                self.state.ore += v;
                self.state.run_ore_earned += v;
                self.state.total_ore_all_time += v;
                self.push_toast(format!("+{} руды (админ)", crate::util::fmt_num(v)), col::admin_accent());
            }
            AdminAction::GiveDarkMatter(v) => {
                self.state.dark_matter += v;
                self.push_toast(format!("+{} ТМ (админ)", crate::util::fmt_num(v)), col::dark_matter());
            }
            AdminAction::KillPlanet => {
                self.state.shield_hp = 0.0;
                self.state.planet_hp = 0.0;
                self.on_planet_destroyed();
            }
            AdminAction::BreakShield => {
                self.state.shield_hp = 0.0;
                self.push_toast("Щит сломан (админ)".to_string(), col::shield());
            }
            AdminAction::TeleportBiome(i) => {
                self.state.biome = i;
                self.state.planet = 0;
                self.state.planet_hp = planet::planet_max_hp(i, 0, self.state.galaxy);
                self.state.shield_hp = planet::shield_max_hp(i, 0, self.state.galaxy);
                self.refresh_planet();
                self.push_toast(format!("Телепорт: {}", planet::biome(i).name), col::accent());
            }
            AdminAction::SkipPlanets(n) => {
                for _ in 0..n {
                    if self.state.planet + 1 < planet::PLANETS_PER_BIOME {
                        self.state.planet += 1;
                    } else if self.state.biome + 1 < planet::BIOMES.len() {
                        self.state.biome += 1;
                        self.state.planet = 0;
                    } else {
                        self.state.galaxy += 1;
                        self.state.biome = 0;
                        self.state.planet = 0;
                    }
                }
                self.state.planet_hp = planet::planet_max_hp(self.state.biome, self.state.planet, self.state.galaxy);
                self.state.shield_hp = planet::shield_max_hp(self.state.biome, self.state.planet, self.state.galaxy);
                self.refresh_planet();
            }
            AdminAction::SpawnRelic(i) => {
                let add = if is_key_down(KeyCode::LeftShift) { 10 } else { 1 };
                let def = &crate::components::relics::RELICS[i.min(crate::components::relics::RELICS.len() - 1)];
                *self.state.relics.entry(def.id.to_string()).or_insert(0) += add;
                self.state.stats.relics_found += add;
                self.push_toast(format!("+{} реликвия: {}", add, def.name), col::dark_matter());
            }
            AdminAction::SpawnAllRelics => {
                for def in crate::components::relics::RELICS.iter() {
                    *self.state.relics.entry(def.id.to_string()).or_insert(0) += 1;
                }
                self.push_toast("Все реликвии выданы".to_string(), col::dark_matter());
            }
            AdminAction::ClearRelics => {
                self.state.relics.clear();
                self.push_toast("Реликвии очищены".to_string(), col::danger());
            }
        }
    }

    fn play_ui_sound(&mut self) {
        if self.assets.audio_enabled {
            if let Some(s) = self.assets.sfx.ui.as_ref() {
                let vol = self.settings.effective_sfx() * 0.7;
                macroquad::audio::play_sound(s, macroquad::audio::PlaySoundParams { looped: false, volume: vol });
            }
        }
    }

    pub fn apply_fullscreen(&mut self) {
        macroquad::window::set_fullscreen(self.settings.fullscreen);
        if !self.settings.fullscreen {
            macroquad::window::request_new_screen_size(self.settings.res_w as f32, self.settings.res_h as f32);
        }
    }

    // ======================= КЛАВИАТУРА =======================

    pub fn handle_keys(&mut self) {
        // Админ-панель
        if is_key_pressed(KeyCode::F12) || is_key_pressed(KeyCode::GraveAccent) {
            admin_panel::toggle(&mut self.admin);
        }
        // Полный экран
        if is_key_pressed(KeyCode::F11) {
            self.settings.fullscreen = !self.settings.fullscreen;
            self.apply_fullscreen();
        }
        // Mute
        if is_key_pressed(KeyCode::M) {
            self.settings.muted = !self.settings.muted;
        }
        // Ручной сейв
        if is_key_down(KeyCode::LeftControl) && is_key_pressed(KeyCode::S) {
            self.do_save(false);
        }
        // Esc: закрыть слои, иначе выход с сохранением
        if is_key_pressed(KeyCode::Escape) {
            if self.admin.open {
                admin_panel::toggle(&mut self.admin);
            } else if self.modal != Modal::None {
                self.modal = Modal::None;
                self.reset_armed = false;
            } else {
                self.do_save(false);
                println!("[exit] Сохранено, выход.");
                std::process::exit(0);
            }
        }
    }

    // ======================= SMOKE TEST =======================

    /// Скриншот текущего кадра (только для smoke-теста).
    fn screenshot(&self, path: &str) {
        let img = macroquad::prelude::get_screen_data();
        img.export_png(path);
        println!("[smoke] скриншот: {}", path);
    }

    /// Автотест при запуске: STELLAR_SMOKE_FRAMES=300 cargo run
    /// Проходит по ключевым системам и завершает процесс с SMOKE OK.
    pub fn smoke_tick(&mut self) {
        let f = self.frame_count;
        match f {
            10 => {
                self.state.ore += 1000.0;
                println!("[smoke] выдано 1000 руды");
            }
            20 => {
                let r = crate::systems::economy::buy_drones(&mut self.state, DroneKind::Probe, crate::systems::economy::BuyMode::Many(5));
                println!("[smoke] куплено дронов: {}", r.bought);
            }
            30 => {
                let field = PlayField::compute();
                self.do_click(field.cx, field.cy, false);
                println!("[smoke] клик выполнен");
            }
            40 => {
                self.settings.autoclicker_enabled = true;
                println!("[smoke] автокликер включён");
            }
            80 => {
                self.state.shield_hp = 0.0;
                self.state.planet_hp = 0.0;
                self.on_planet_destroyed();
                println!("[smoke] планета уничтожена, новая: биом {} планета {}", self.state.biome, self.state.planet);
            }
            100 => {
                self.admin.god = true;
                self.do_click(PlayField::compute().cx, PlayField::compute().cy, false);
                println!("[smoke] режим бога: биом {} планета {}", self.state.biome, self.state.planet);
            }
            120 => {
                self.admin.god = false;
                self.process_admin_action(AdminAction::TeleportBiome(3));
                println!("[smoke] телепорт в биом 3");
            }
            140 => {
                self.process_admin_action(AdminAction::SpawnAllRelics);
                println!("[smoke] реликвии выданы: {}", self.state.relics.len());
            }
            160 => {
                self.settings.autoclicker_enabled = false;
                let ok = save_system::save_all(&self.state, &self.settings);
                println!("[smoke] сейв: {}", ok);
            }
            170 => {
                self.admin.open = true;
            }
            180 => {
                self.screenshot("/tmp/render/smoke_admin.png");
            }
            190 => {
                self.admin.open = false;
                self.modal = Modal::Settings;
            }
            200 => {
                self.screenshot("/tmp/render/smoke_settings.png");
            }
            210 => {
                self.modal = Modal::Relics;
            }
            220 => {
                self.screenshot("/tmp/render/smoke_relics.png");
            }
            230 => {
                self.modal = Modal::None;
                self.screenshot("/tmp/render/smoke_main.png");
            }
            240 => {
                // Изоляция примитивов: чёрный фон + отдельные вызовы
                clear_background(BLACK);
                draw_rectangle(20.0, 20.0, 60.0, 60.0, WHITE);
                crate::ui::icons::draw_icon(crate::ui::icons::Icon::Drill, 200.0, 60.0, 30.0, WHITE);
                crate::ui::icons::draw_icon(crate::ui::icons::Icon::Relic, 320.0, 60.0, 26.0, WHITE);
                draw_poly(450.0, 60.0, 4, 13.0, 45.0, WHITE);
                draw_poly_lines(450.0, 160.0, 6, 12.0, 30.0, 2.0, WHITE);
                draw_circle(600.0, 60.0, 15.0, WHITE);
                self.planet.draw(200.0, 300.0, 100.0);
                self.screenshot("/tmp/render/smoke_iso.png");
                self.planet.debug_export("/tmp/render/smoke_sphere.png");
            }
            _ => {}
        }
        if let Some(n) = self.smoke_frames {
            if f >= n {
                let ok = save_system::save_all(&self.state, &self.settings);
                println!("SMOKE OK (frames={}, save={})", f, ok);
                std::process::exit(0);
            }
        }
    }
}
