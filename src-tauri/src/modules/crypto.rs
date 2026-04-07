use crate::modules::helpers::argon2::strong_argon2;
use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng as AeadRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use anyhow::{anyhow, Result};
use argon2::password_hash::SaltString;
use argon2::{PasswordHash, PasswordHasher, PasswordVerifier};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use once_cell::sync::Lazy;
use rand::rngs::OsRng;
use std::sync::Mutex;
use zeroize::{Zeroize, Zeroizing};

static MASTER_KEY: Lazy<Mutex<Option<Zeroizing<[u8; 32]>>>> = Lazy::new(|| Mutex::new(None)); //глобальное состоянеие для мастер ключа(хранится только в памяти)

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = strong_argon2();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow!("Hash failed: {}", e))?;
    Ok(hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    let parsed = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    strong_argon2()
        .verify_password(password.as_bytes(), &parsed)
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
pub fn get_master_key() -> Result<Zeroizing<[u8; 32]>> {
    let lock = MASTER_KEY.lock().unwrap();
    match lock.as_ref() {
        Some(key) => Ok(key.clone()),
        None => Err(anyhow!("MASTER_KEY_LOCKED: not set")),
    }
}

//получаем масстер ключ из пароля + соли
pub fn derive_master_key_from_password(password: &str, salt: &[u8]) -> Result<[u8; 32]> {
    let mut output = [0u8; 32];
    let argon2 = strong_argon2();
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut output)
        .map_err(|e| anyhow!("Key derivation failed: {}", e))?;
    Ok(output)
}

//шифрует данные с использованием мастер ключа
pub fn encrypt_with_master(plaintext: &[u8]) -> Result<String> {
    let key = get_master_key()?;
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&*key));
    let nonce = Aes256Gcm::generate_nonce(&mut AeadRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| anyhow!("Encrypt failed: {}", e))?;
    let mut combined = vec![1u8]; // version
    combined.extend_from_slice(&nonce);
    combined.extend(ciphertext);
    Ok(BASE64.encode(&combined))
}
//расшифровывает данные с использованием мастер ключа
pub fn decrypt_with_master(encrypted_b64: &str) -> Result<Vec<u8>> {
    let key = get_master_key()?;
    let combined = BASE64
        .decode(encrypted_b64)
        .map_err(|e| anyhow!("Invalid base64: {}", e))?;
    if combined.is_empty() {
        return Err(anyhow!("Empty encrypted data"));
    }
    let version = combined[0];
    if version != 1 {
        return Err(anyhow!("Unsupported encrypted data version {}", version));
    }
    if combined.len() < 1 + 12 {
        return Err(anyhow!("Encrypted data too short"));
    }
    let nonce = Nonce::from_slice(&combined[1..1 + 12]);
    let ciphertext = &combined[1 + 12..];
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&*key));
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow!("Decrypt failed: {}", e))?;
    Ok(plaintext)
}
