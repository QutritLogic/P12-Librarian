#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod menu;
mod settings;
mod tags;

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(commands::AppState::default())
        .manage(settings::SettingsState::default())
        .manage(tags::TagState::default())
        .menu(menu::build)
        .setup(|app| {
            let handle = app.handle();
            let saved = settings::load(handle);
            *handle.state::<settings::SettingsState>().0.lock().unwrap() = saved.clone();
            settings::apply(handle, &saved);
            Ok(())
        })
        .on_menu_event(menu::handle)
        .invoke_handler(tauri::generate_handler![
            commands::list_ports,
            commands::connect,
            commands::disconnect,
            commands::ctrl_held,
            commands::read_programs,
            commands::cancel,
            commands::open_syx,
            commands::save_syx,
            commands::save_syx_split,
            commands::audition,
            commands::write_programs,
            commands::cleanify_names,
            commands::rename_programs,
            commands::has_init_template,
            commands::capture_init_template,
            commands::init_programs,
            commands::copy_to_layer_b,
            commands::swap_programs,
            commands::backup_dir,
            commands::list_restore_points,
            commands::restore,
            commands::delete_restore_point,
            settings::get_settings,
            settings::zoom,
            settings::appearance_options,
            settings::set_appearance,
            settings::set_audition,
            settings::disclaimer_accepted,
            settings::accept_disclaimer,
            settings::quit,
            tags::get_tags,
            tags::update_tags,
        ])
        .run(tauri::generate_context!())
        .expect("error while running the application");
}
