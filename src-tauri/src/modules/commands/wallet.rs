use tauri::AppHandle;

use crate::{
    modules::{
        crypto::{decrypt_with_master, encrypt_with_master},
        helpers::files::{get_config_path, save_config},
    },
    structs::files::AppConfig,
};

#[tauri::command]
pub fn save_private_key(app: AppHandle, private_key_b58: String) -> Result<(), String> {
    let encrypted = encrypt_with_master(private_key_b58.as_bytes())?;
    let path = get_config_path(&app);
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut config: AppConfig = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    config.encrypted_private_key = Some(encrypted);
    save_config(&app, &config)?;
    Ok(())
}

#[tauri::command]
pub fn get_private_key(app: AppHandle) -> Result<Option<String>, String> {
    let path = get_config_path(&app);
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let config: AppConfig = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    match config.encrypted_private_key {
        Some(enc) => {
            let decrypted_bytes = decrypt_with_master(&enc)?;
            let private_key = String::from_utf8(decrypted_bytes).map_err(|e| e.to_string())?;
            Ok(Some(private_key))
        }
        None => Ok(None),
    }
}

#[tauri::command]
pub fn change_private_key(app: AppHandle, new_private_key_b58: String) -> Result<(), String> {
    let encrypted = encrypt_with_master(new_private_key_b58.as_bytes())?;
    let path = get_config_path(&app);
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut config: AppConfig = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    config.encrypted_private_key = Some(encrypted);
    save_config(&app, &config)?;
    Ok(())
}

#[tauri::command]
pub fn remove_private_key(app: AppHandle) -> Result<(), String> {
    let path = get_config_path(&app);
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut config: AppConfig = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    config.encrypted_private_key = None;
    save_config(&app, &config)?;
    Ok(())
}
