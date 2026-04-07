use crate::modules::crypto::{
    clear_master_key, decrypt_with_master, derive_master_key_from_password, encrypt_with_master,
    set_master_key, verify_password,
};
use crate::modules::helpers::files::save_config;
use crate::{
    modules::{crypto::hash_password, helpers::files::get_config_path},
    structs::files::AppConfig,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand::RngCore;
use tauri::AppHandle;
//проверяет инициализирован ли пароль
#[tauri::command]
pub fn is_initialized(app: AppHandle) -> Result<bool, String> {
    let path = get_config_path(&app);
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let config: AppConfig = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(config.initialized)
}

//устанавливаем мастер пароль(только если он еще не установлен)(initialized - false)
#[tauri::command]
pub fn setup_password(app: AppHandle, password: String) -> Result<(), String> {
    let path = get_config_path(&app);
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut config: AppConfig = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    if config.initialized {
        return Err("Password already set".to_string());
    }
    let hash = hash_password(&password).map_err(|e| e.to_string())?;

    //генеррирем salt для мастер ключа
    let mut salt = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt);

    let salt_b64 = BASE64.encode(salt);

    //вычисляем мастер-ключ
    let master_key = derive_master_key_from_password(&password, &salt);
    config.initialized = true;
    config.password_hash = Some(hash);
    config.salt = Some(salt_b64);
    save_config(&app, &config)?;
    set_master_key(master_key);
    Ok(())
}

//восстанавливает доступ, проверяя пароль
#[tauri::command]
pub async fn unlock(app: AppHandle, password: String) -> Result<bool, String> {
    let path = get_config_path(&app);
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let config: AppConfig = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    if !config.initialized {
        return Err("not initialized".to_string());
    }
    let hash = config.password_hash.ok_or("hash not found")?;
    if !verify_password(&password, &hash) {
        return Ok(false);
    }
    //восстанавливаем мастер-ключ
    let salt_b64 = config.salt.ok_or("salt not found")?;
    let salt = BASE64.decode(&salt_b64).map_err(|e| e.to_string())?;
    let master_key = derive_master_key_from_password(&password, &salt);
    set_master_key(master_key);
    Ok(true)
}

//блокирует доступ, очищая мастер-ключ
#[tauri::command]
pub fn lock() -> Result<(), String> {
    clear_master_key();
    Ok(())
}

// шифрование данных (использует мастер-ключ)
#[tauri::command]
pub fn encrypt_data(plaintext: String) -> Result<String, String> {
    encrypt_with_master(plaintext.as_bytes())
}

// расшифровка данных
#[tauri::command]
pub fn decrypt_data(ciphertext_b64: String) -> Result<String, String> {
    let bytes = decrypt_with_master(&ciphertext_b64)?;
    String::from_utf8(bytes).map_err(|e| e.to_string())
}
