use std::{fs, path::PathBuf};

use tauri::{AppHandle, Manager};

use crate::structs::files::AppConfig;

pub fn get_config_path(app: &AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_data_dir()
        .expect("Failed to get app_data_dir");
    dir.join("config.json")
}

pub fn check_app_config(app: &AppHandle) {
    let path = get_config_path(app);
    if !path.exists() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("Falied to create dir");
        }
        let default_config = AppConfig {
            initialized: false,
            password_hash: None,
            salt: None,
            encrypted_private_key: None,
        };
        fs::write(
            &path,
            serde_json::to_string_pretty(&default_config).unwrap(),
        )
        .expect("Failed to write config.json");
        println!("Config created at: {:?}", path);
    } else {
        println!("Config already exists at: {:?}", path);
    }
}

pub fn save_config(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let path = get_config_path(app);
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}
