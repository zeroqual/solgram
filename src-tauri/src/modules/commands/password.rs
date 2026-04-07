use crate::modules::crypto::{
    clear_master_key, decrypt_with_master, derive_master_key_from_password, encrypt_with_master,
    hash_password, set_master_key, verify_password,
};
use crate::modules::helpers::files::{load_config, save_config};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use once_cell::sync::Lazy;
use rand::RngCore;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::AppHandle;
use zeroize::Zeroizing;

struct UnlockState {
    attempts: u32,
    locked_until: Option<Instant>,
}

static UNLOCK_STATE: Lazy<Mutex<UnlockState>> = Lazy::new(|| {
    Mutex::new(UnlockState {
        attempts: 0,
        locked_until: None,
    })
});

#[tauri::command]
pub fn is_initialized(app: AppHandle) -> Result<bool, String> {
    let config = load_config(&app)?;
    Ok(config.initialized)
}

#[tauri::command]
pub async fn setup_password(app: AppHandle, password: String) -> Result<(), String> {
    let password = Zeroizing::new(password);
    let mut config = load_config(&app)?;
    if config.initialized {
        return Err("Password already set".to_string());
    }
    let hash = hash_password(password.as_str()).map_err(|e| e.to_string())?;
    let mut salt = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt);
    let salt_b64 = BASE64.encode(salt);
    let master_key =
        derive_master_key_from_password(password.as_str(), &salt).map_err(|e| e.to_string())?;
    config.version = 1;
    config.initialized = true;
    config.password_hash = Some(hash);
    config.salt = Some(salt_b64);
    set_master_key(master_key);
    save_config(&app, &config).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn unlock(app: AppHandle, password: String) -> Result<bool, String> {
    let password = Zeroizing::new(password);
    {
        let mut state = UNLOCK_STATE.lock().unwrap();
        if let Some(locked_until) = state.locked_until {
            if locked_until > Instant::now() {
                let remaining = locked_until.duration_since(Instant::now());
                return Err(format!(
                    "Rate limit exceeded. Wait {} seconds.",
                    remaining.as_secs()
                ));
            } else {
                state.attempts = 0;
                state.locked_until = None;
            }
        }
    }
    let config = load_config(&app)?;
    if !config.initialized {
        return Err("App not initialized. Set a password first.".to_string());
    }
    let hash = config.password_hash.ok_or("Hash not found")?;
    if !verify_password(password.as_str(), &hash) {
        let mut state = UNLOCK_STATE.lock().unwrap();
        state.attempts += 1;
        if state.attempts >= 5 {
            state.locked_until = Some(Instant::now() + Duration::from_secs(30));
            return Err("Rate limit exceeded. Wait 30 seconds.".to_string());
        } else {
            let delay =
                Duration::from_millis(500 * state.attempts as u64).min(Duration::from_secs(2));
            std::thread::sleep(delay);
        }
        return Ok(false);
    }
    // успех
    {
        let mut state = UNLOCK_STATE.lock().unwrap();
        state.attempts = 0;
        state.locked_until = None;
    }
    let salt_b64 = config.salt.ok_or("salt not found")?;
    let salt = BASE64.decode(&salt_b64).map_err(|e| e.to_string())?;
    let master_key =
        derive_master_key_from_password(password.as_str(), &salt).map_err(|e| e.to_string())?;
    set_master_key(master_key);
    Ok(true)
}

#[tauri::command]
pub fn lock() -> Result<(), String> {
    clear_master_key();
    Ok(())
}

#[tauri::command]
pub fn encrypt_data(plaintext: String) -> Result<String, String> {
    encrypt_with_master(plaintext.as_bytes()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn decrypt_data(ciphertext_b64: String) -> Result<String, String> {
    let bytes = decrypt_with_master(&ciphertext_b64).map_err(|e| e.to_string())?;
    String::from_utf8(bytes).map_err(|e| e.to_string())
}
