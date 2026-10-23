//! View settings (zoom, font and theme), owned by the backend so the View menu and
//! the frontend agree. Saved as JSON in the app's config directory.

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::menu::{CheckMenuItem, MenuItemBuilder, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Emitter, Manager, Wry};

/// Version of DISCLAIMER.md that users must accept before using the app.
/// Increase it whenever the disclaimer text changes, so everyone is asked
/// again.
pub const DISCLAIMER_VERSION: u32 = 3;

const ZOOM_STEPS: [f64; 9] = [0.8, 0.9, 1.0, 1.1, 1.25, 1.5, 1.75, 2.0, 2.5];

/// How many files File → Open recent lists.
const MAX_RECENT_FILES: usize = 8;

/// UI font. The serialized names are also the frontend's `data-font` values.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Font {
    Fira,
    JetbrainsMono,
    Hack,
    System,
}

impl Font {
    /// Every font, in menu order, with its display name.
    pub const ALL: [(Font, &'static str); 4] = [
        (Font::Fira, "Fira Code"),
        (Font::JetbrainsMono, "JetBrains Mono"),
        (Font::Hack, "Hack"),
        (Font::System, "System font"),
    ];
}

/// Colour theme. `System` follows the desktop's light or dark preference.
/// The serialized names are also the frontend's `data-theme` values.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Theme {
    System,
    Dracula,
    CatppuccinMocha,
    CatppuccinLatte,
    TokyoNight,
    GruvboxDark,
    OneDarkPro,
    RosePine,
    Kanagawa,
    EverforestDark,
    SolarizedDark,
}

impl Theme {
    /// Every theme, in menu order, with its display name.
    pub const ALL: [(Theme, &'static str); 11] = [
        (Theme::System, "System (light or dark)"),
        (Theme::Dracula, "Dracula"),
        (Theme::CatppuccinMocha, "Catppuccin Mocha"),
        (Theme::CatppuccinLatte, "Catppuccin Latte"),
        (Theme::TokyoNight, "Tokyo Night"),
        (Theme::GruvboxDark, "Gruvbox Dark"),
        (Theme::OneDarkPro, "One Dark Pro"),
        (Theme::RosePine, "Rosé Pine"),
        (Theme::Kanagawa, "Kanagawa"),
        (Theme::EverforestDark, "Everforest Dark"),
        (Theme::SolarizedDark, "Solarized Dark"),
    ];
}

/// What auditioning a patch does besides loading it into the edit buffer.
#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct Audition {
    /// Play a note after loading the patch.
    pub play_note: bool,
    /// MIDI note number; 60 is C3, the Prophet 12's middle C.
    pub note: u8,
    pub length_ms: u16,
    /// 1-16, or 0 to use the channel set on the synth.
    pub channel: u8,
    /// Audition each patch the arrow keys move to.
    pub on_move: bool,
}

impl Default for Audition {
    fn default() -> Self {
        Self {
            play_note: true,
            note: 60,
            length_ms: 1000,
            channel: 0,
            on_move: false,
        }
    }
}

impl Audition {
    /// Keeps values in the ranges the app offers.
    fn clamped(self) -> Self {
        Self {
            note: self.note.min(127),
            length_ms: self.length_ms.clamp(100, 10_000),
            channel: self.channel.min(16),
            ..self
        }
    }
}

/// What a drop within the Prophet 12 pane does, as last chosen in its dialog.
#[derive(Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Rearrange {
    #[default]
    Swap,
    Move,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub zoom: f64,
    pub font: Font,
    pub theme: Theme,
    pub audition: Audition,
    /// The disclaimer version the user accepted; 0 if none.
    pub disclaimer_accepted: u32,
    /// The Prophet 12 pane's share of the window width, 0.25-0.75.
    pub split: f64,
    /// `.syx` files opened or saved in the File pane, newest first.
    pub recent_files: Vec<String>,
    pub rearrange: Rearrange,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            font: Font::Fira,
            theme: Theme::System,
            audition: Audition::default(),
            disclaimer_accepted: 0,
            split: 0.5,
            recent_files: Vec::new(),
            rearrange: Rearrange::Swap,
        }
    }
}

/// The View menu's font and theme items, kept so their check marks can be
/// updated.
pub struct AppearanceItems {
    pub fonts: Vec<(Font, CheckMenuItem<Wry>)>,
    pub themes: Vec<(Theme, CheckMenuItem<Wry>)>,
}

/// File → Open recent, rebuilt whenever the list changes.
pub struct RecentMenu(pub Submenu<Wry>);

/// The menu label for a recent file: its name, then its folder.
fn recent_label(path: &str) -> String {
    let path = std::path::Path::new(path);
    let name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
    match path.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => format!("{name}    ({})", dir.display()),
        _ => name.into_owned(),
    }
}

fn rebuild_recent_menu(app: &AppHandle, menu: &Submenu<Wry>, files: &[String]) -> tauri::Result<()> {
    while menu.remove_at(0)?.is_some() {}
    if files.is_empty() {
        let none = MenuItemBuilder::with_id("recent_none", "No recent files")
            .enabled(false)
            .build(app)?;
        return menu.append(&none);
    }
    for (i, path) in files.iter().enumerate() {
        menu.append(&MenuItemBuilder::with_id(format!("recent:{i}"), recent_label(path)).build(app)?)?;
    }
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItemBuilder::with_id("recent_clear", "Clear list").build(app)?)
}

/// Puts `path` first in the recent files.
pub fn remember_file(settings: &mut Settings, path: &str) {
    settings.recent_files.retain(|p| p != path);
    settings.recent_files.insert(0, path.to_owned());
    settings.recent_files.truncate(MAX_RECENT_FILES);
}

#[derive(Default)]
pub struct SettingsState(pub Mutex<Settings>);

fn path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|dir| dir.join("settings.json"))
}

pub fn load(app: &AppHandle) -> Settings {
    path(app)
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

fn save(app: &AppHandle, settings: &Settings) {
    let Some(path) = path(app) else { return };
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = fs::write(path, json);
    }
}

/// Applies settings to the window and menu, saves them and tells the frontend.
pub fn apply(app: &AppHandle, settings: &Settings) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_zoom(settings.zoom);
    }
    if let Some(items) = app.try_state::<AppearanceItems>() {
        for (font, item) in &items.fonts {
            let _ = item.set_checked(settings.font == *font);
        }
        for (theme, item) in &items.themes {
            let _ = item.set_checked(settings.theme == *theme);
        }
    }
    if let Some(recent) = app.try_state::<RecentMenu>() {
        let _ = rebuild_recent_menu(app, &recent.0, &settings.recent_files);
    }
    save(app, settings);
    let _ = app.emit("settings", settings.clone());
}

/// Changes the settings with `change` and applies the result.
pub fn update(app: &AppHandle, change: impl FnOnce(&mut Settings)) {
    let settings = {
        let state = app.state::<SettingsState>();
        let mut settings = state.0.lock().unwrap_or_else(|p| p.into_inner());
        change(&mut settings);
        settings.clone()
    };
    apply(app, &settings);
}

/// The next zoom step in `direction` (+1 bigger, -1 smaller, 0 actual size).
pub fn step_zoom(current: f64, direction: i32) -> f64 {
    match direction {
        0 => 1.0,
        d if d > 0 => ZOOM_STEPS
            .iter()
            .copied()
            .find(|&z| z > current + 0.001)
            .unwrap_or(current),
        _ => ZOOM_STEPS
            .iter()
            .rev()
            .copied()
            .find(|&z| z < current - 0.001)
            .unwrap_or(current),
    }
}

#[tauri::command]
pub fn get_settings(state: tauri::State<'_, SettingsState>) -> Settings {
    state.0.lock().unwrap_or_else(|p| p.into_inner()).clone()
}

#[derive(Serialize)]
pub struct Choice<T> {
    id: T,
    name: &'static str,
}

#[derive(Serialize)]
pub struct AppearanceOptions {
    themes: Vec<Choice<Theme>>,
    fonts: Vec<Choice<Font>>,
}

/// The themes and fonts to choose from, for the Appearance dialog.
#[tauri::command]
pub fn appearance_options() -> AppearanceOptions {
    AppearanceOptions {
        themes: Theme::ALL.map(|(id, name)| Choice { id, name }).into(),
        fonts: Font::ALL.map(|(id, name)| Choice { id, name }).into(),
    }
}

/// Sets the theme and/or the font.
#[tauri::command]
pub fn set_appearance(app: AppHandle, theme: Option<Theme>, font: Option<Font>) {
    update(&app, |s| {
        s.theme = theme.unwrap_or(s.theme);
        s.font = font.unwrap_or(s.font);
    });
}

/// Changes the audition settings.
#[tauri::command]
pub fn set_audition(app: AppHandle, audition: Audition) {
    update(&app, |s| s.audition = audition.clamped());
}

/// Whether the user has accepted the current disclaimer.
#[tauri::command]
pub fn disclaimer_accepted(state: tauri::State<'_, SettingsState>) -> bool {
    state
        .0
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .disclaimer_accepted
        >= DISCLAIMER_VERSION
}

/// Records that the user accepted the current disclaimer.
#[tauri::command]
pub fn accept_disclaimer(app: AppHandle) {
    update(&app, |s| s.disclaimer_accepted = DISCLAIMER_VERSION);
}

/// Exits the app, for a user who declines the disclaimer.
#[tauri::command]
pub fn quit(app: AppHandle) {
    app.exit(0);
}

/// Sets how much of the window width the Prophet 12 pane takes.
#[tauri::command]
pub fn set_split(app: AppHandle, split: f64) {
    if split.is_finite() {
        update(&app, |s| s.split = split.clamp(0.25, 0.75));
    }
}

/// Remembers whether drops within the Prophet 12 pane swap or move.
#[tauri::command]
pub fn set_rearrange(app: AppHandle, rearrange: Rearrange) {
    update(&app, |s| s.rearrange = rearrange);
}

/// Adds a file to File → Open recent, or removes it (e.g. when it's gone).
#[tauri::command]
pub fn recent_file(app: AppHandle, path: String, remove: bool) {
    update(&app, |s| {
        if remove {
            s.recent_files.retain(|p| *p != path);
        } else {
            remember_file(s, &path);
        }
    });
}

/// Zooms in (+1), out (-1) or back to actual size (0).
#[tauri::command]
pub fn zoom(app: AppHandle, direction: i32) {
    update(&app, |s| s.zoom = step_zoom(s.zoom, direction));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_steps() {
        assert_eq!(step_zoom(1.0, 1), 1.1);
        assert_eq!(step_zoom(1.0, -1), 0.9);
        assert_eq!(step_zoom(2.5, 1), 2.5);
        assert_eq!(step_zoom(0.8, -1), 0.8);
        assert_eq!(step_zoom(1.3, -1), 1.25); // off-step values snap to a step
        assert_eq!(step_zoom(1.75, 0), 1.0);
    }

    #[test]
    fn recent_files_newest_first_without_repeats() {
        let mut settings = Settings::default();
        for i in 0..10 {
            remember_file(&mut settings, &format!("/syx/{i}.syx"));
        }
        remember_file(&mut settings, "/syx/5.syx");
        assert_eq!(settings.recent_files.len(), MAX_RECENT_FILES);
        assert_eq!(settings.recent_files[0], "/syx/5.syx");
        assert_eq!(settings.recent_files[1], "/syx/9.syx");
        assert_eq!(settings.recent_files.iter().filter(|p| *p == "/syx/5.syx").count(), 1);
    }
}
