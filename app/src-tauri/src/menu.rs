//! The native menu bar. View items are handled here; Quit exits; the rest
//! are forwarded to the frontend as `menu` events carrying the item id.

use tauri::menu::{
    CheckMenuItemBuilder, Menu, MenuBuilder, MenuEvent, MenuItemBuilder, SubmenuBuilder,
};
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Emitter, Manager, Wry};
use tauri_plugin_opener::OpenerExt;

use crate::settings::{self, AppearanceItems, Font, Theme};

pub fn build(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let item = |id: &str, text: &str, shortcut: Option<&str>| {
        let builder = MenuItemBuilder::with_id(id, text);
        match shortcut {
            Some(keys) => builder.accelerator(keys).build(app),
            None => builder.build(app),
        }
    };

    let file = SubmenuBuilder::new(app, "File")
        .item(&item("open", "Open .syx file…", Some("CmdOrCtrl+O"))?)
        .item(&item(
            "backup",
            "Back up all User banks…",
            Some("CmdOrCtrl+B"),
        )?)
        .separator()
        .item(&item("quit", "Quit", Some("CmdOrCtrl+Q"))?)
        .build()?;

    // Zoom shortcuts are handled by the frontend from the typed character,
    // because menu accelerators follow the US key positions and Ctrl + would
    // land on the wrong key on e.g. a Swedish keyboard.
    // Check items with ids like "font:2", indexing Font::ALL / Theme::ALL.
    let checks = |prefix: &str, names: &[&str]| {
        names
            .iter()
            .enumerate()
            .map(|(i, name)| CheckMenuItemBuilder::with_id(format!("{prefix}:{i}"), *name).build(app))
            .collect::<tauri::Result<Vec<_>>>()
    };
    let fonts = checks("font", &Font::ALL.map(|(_, name)| name))?;
    let themes = checks("theme", &Theme::ALL.map(|(_, name)| name))?;
    let mut font_menu = SubmenuBuilder::new(app, "Font");
    for item in &fonts {
        font_menu = font_menu.item(item);
    }
    let mut theme_menu = SubmenuBuilder::new(app, "Theme");
    for item in &themes {
        theme_menu = theme_menu.item(item);
    }
    let (font_menu, theme_menu) = (font_menu.build()?, theme_menu.build()?);

    let view = SubmenuBuilder::new(app, "View")
        .item(&item("zoom_in", "Zoom in    (Ctrl +)", None)?)
        .item(&item("zoom_out", "Zoom out    (Ctrl −)", None)?)
        .item(&item("zoom_reset", "Actual size    (Ctrl 0)", None)?)
        .separator()
        .item(&item("appearance", "Appearance…", Some("CmdOrCtrl+Shift+A"))?)
        .item(&theme_menu)
        .item(&font_menu)
        .build()?;
    app.manage(AppearanceItems {
        fonts: Font::ALL.iter().map(|(font, _)| *font).zip(fonts).collect(),
        themes: Theme::ALL.iter().map(|(theme, _)| *theme).zip(themes).collect(),
    });

    let help = SubmenuBuilder::new(app, "Help")
        .item(&item("help", "User guide (PDF)", Some("F1"))?)
        .item(&item("disclaimer", "Disclaimer", None)?)
        .separator()
        .item(&item("about", "About P12 Librarian", None)?)
        .build()?;

    MenuBuilder::new(app).items(&[&file, &view, &help]).build()
}

/// Opens the bundled PDF user guide in the system's default PDF viewer.
fn open_user_guide(app: &AppHandle) -> Result<(), String> {
    let path = app
        .path()
        .resolve("resources/user-guide.pdf", BaseDirectory::Resource)
        .map_err(|e| e.to_string())?;
    if !path.exists() {
        return Err(format!("The user guide is missing: {}", path.display()));
    }
    app.opener()
        .open_path(path.to_string_lossy(), None::<&str>)
        .map_err(|e| format!("Could not open the user guide: {e}"))
}

pub fn handle(app: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        "quit" => app.exit(0),
        "help" => {
            if let Err(message) = open_user_guide(app) {
                let _ = app.emit("app-error", message);
            }
        }
        "zoom_in" => settings::update(app, |s| s.zoom = settings::step_zoom(s.zoom, 1)),
        "zoom_out" => settings::update(app, |s| s.zoom = settings::step_zoom(s.zoom, -1)),
        "zoom_reset" => settings::update(app, |s| s.zoom = 1.0),
        // Re-applying also restores the check marks GTK toggled on click.
        id if id.starts_with("font:") || id.starts_with("theme:") => {
            let (kind, index) = id.split_once(':').unwrap_or_default();
            let Ok(index) = index.parse::<usize>() else { return };
            match kind {
                "font" => {
                    if let Some((font, _)) = Font::ALL.get(index) {
                        settings::update(app, |s| s.font = *font);
                    }
                }
                _ => {
                    if let Some((theme, _)) = Theme::ALL.get(index) {
                        settings::update(app, |s| s.theme = *theme);
                    }
                }
            }
        }
        id => {
            let _ = app.emit("menu", id);
        }
    }
}
