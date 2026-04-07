use crate::modules::crypto::{decrypt_with_master, encrypt_with_master};
use crate::modules::helpers::files::{load_config, save_config};
use tauri::AppHandle;

#[tauri::command]
pub fn save_private_key(app: AppHandle, private_key_b58: String) -> Result<(), String> {
    let encrypted = encrypt_with_master(private_key_b58.as_bytes()).map_err(|e| e.to_string())?;
    let mut config = load_config(&app)?;
    config.encrypted_private_key = Some(encrypted);
    save_config(&app, &config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_private_key(app: AppHandle) -> Result<Option<String>, String> {
    let config = load_config(&app)?;
    match config.encrypted_private_key {
        Some(enc) => {
            let bytes = decrypt_with_master(&enc).map_err(|e| e.to_string())?;
            let private_key = String::from_utf8(bytes).map_err(|e| e.to_string())?;
            Ok(Some(private_key))
        }
        None => Ok(None),
    }
}

#[tauri::command]
pub fn change_private_key(app: AppHandle, new_private_key_b58: String) -> Result<(), String> {
    let encrypted =
        encrypt_with_master(new_private_key_b58.as_bytes()).map_err(|e| e.to_string())?;
    let mut config = load_config(&app)?;
    config.encrypted_private_key = Some(encrypted);
    save_config(&app, &config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_private_key(app: AppHandle) -> Result<(), String> {
    let mut config = load_config(&app)?;
    config.encrypted_private_key = None;
    save_config(&app, &config).map_err(|e| e.to_string())
}
