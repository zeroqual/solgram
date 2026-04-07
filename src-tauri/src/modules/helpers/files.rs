use crate::modules::crypto::get_master_key;
use crate::structs::files::AppConfig;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use hmac_sha256::HMAC;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub fn get_config_path(app: &AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_data_dir()
        .expect("Failed to get app_data_dir");
    dir.join("config.json")
}

pub fn save_config(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let path = get_config_path(app);

    let mut config_to_save = config.clone();
    config_to_save.hmac = None;
    let json = serde_json::to_string_pretty(&config_to_save).map_err(|e| e.to_string())?;
    let hmac_value = if let Ok(master_key) = crate::modules::crypto::get_master_key() {
        let mac = HMAC::mac(json.as_bytes(), &master_key[..]);
        Some(BASE64.encode(mac))
    } else {
        None
    };

    let mut final_config = config_to_save;
    final_config.hmac = hmac_value;
    let final_json = serde_json::to_string_pretty(&final_config).map_err(|e| e.to_string())?;
    fs::write(path, final_json).map_err(|e| e.to_string())
}

pub fn load_config(app: &AppHandle) -> Result<AppConfig, String> {
    let path = get_config_path(app);
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let config: AppConfig = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    if let Ok(master_key) = get_master_key() {
        if let Some(saved_hmac) = &config.hmac {
            let mut config_without_hmac = config.clone();
            config_without_hmac.hmac = None;
            let json =
                serde_json::to_string_pretty(&config_without_hmac).map_err(|e| e.to_string())?;
            let computed_mac = HMAC::mac(json.as_bytes(), &master_key[..]);
            let computed_b64 = BASE64.encode(computed_mac);
            if computed_b64 != *saved_hmac {
                return Err("hmac mismatch".to_string());
            }
        }
    }
    Ok(config)
}

pub fn check_app_config(app: &AppHandle) {
    let path = get_config_path(app);
    if !path.exists() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let default_config = AppConfig {
            version: 1,
            initialized: false,
            password_hash: None,
            salt: None,
            encrypted_private_key: None,
            hmac: None,
        };
        let json = serde_json::to_string_pretty(&default_config).unwrap();
        fs::write(&path, json).unwrap();
        println!("Config created at: {:?}", path);
    }
}
