mod modules;
mod structs;

use modules::commands::password::{is_initialized, lock, setup_password, unlock};
use modules::commands::wallet::{
    change_private_key, get_private_key, remove_private_key, save_private_key,
};
use modules::helpers::files::check_app_config;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            check_app_config(&app.handle());
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            is_initialized,
            setup_password,
            unlock,
            lock,
            save_private_key,
            get_private_key,
            change_private_key,
            remove_private_key
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
