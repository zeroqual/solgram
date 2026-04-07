use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng as AeadRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use once_cell::sync::Lazy;
use rand::rngs::OsRng;
use std::sync::Mutex;
use zeroize::{Zeroize, Zeroizing};

static MASTER_KEY: Lazy<Mutex<Option<Zeroizing<[u8; 32]>>>> = Lazy::new(|| Mutex::new(None)); //глобальное состоянеие для мастер ключа(хранится только в памяти)

pub fn hash_password(password: &str) -> Result<String, anyhow::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(password_hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

//устанавливает мастер ключ в память
pub fn set_master_key(key: [u8; 32]) {
    let mut lock = MASTER_KEY.lock().unwrap();
    *lock = Some(Zeroizing::new(key));
}
//удаляет мастер ключ из памяти
pub fn clear_master_key() {
    let mut lock = MASTER_KEY.lock().unwrap();
    if let Some(mut key) = lock.take() {
        key.zeroize();
    }
}
//возвращает мастер ключ из памяти
pub fn get_master_key() -> Result<Zeroizing<[u8; 32]>, String> {
    let lock = MASTER_KEY.lock().unwrap();
    match lock.as_ref() {
        Some(key) => Ok(key.clone()),
        None => Err("master key not set".to_string()),
    }
}

//получаем масстер ключ из пароля + соли
pub fn derive_master_key_from_password(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut output = [0u8; 32];
    let argon2 = Argon2::default();
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut output)
        .unwrap();
    output
}

//шифрует данные с использованием мастер ключа
pub fn encrypt_with_master(plaintext: &[u8]) -> Result<String, String> {
    let key = get_master_key()?;
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&*key));
    let nonce = Aes256Gcm::generate_nonce(&mut AeadRng); // 12 байт
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| format!("Encrypt failed: {}", e))?;
    let mut combined = nonce.to_vec();
    combined.extend(ciphertext);
    Ok(BASE64.encode(&combined))
}
//расшифровывает данные с использованием мастер ключа
pub fn decrypt_with_master(encrypted_b64: &str) -> Result<Vec<u8>, String> {
    let key = get_master_key()?;
    let combined = BASE64
        .decode(encrypted_b64)
        .map_err(|e| format!("Invalid base64: {}", e))?;
    if combined.len() < 12 {
        return Err("Too short".to_string());
    }
    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&*key));
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decrypt failed: {}", e))?;
    Ok(plaintext)
}
