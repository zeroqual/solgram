use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    pub version: u8,
    pub initialized: bool,
    pub password_hash: Option<String>, // содержит соль внутри (PHC string)
    pub salt: Option<String>,          // отдельная соль для мастер-ключа (base64)
    pub encrypted_private_key: Option<String>,
    pub hmac: Option<String>, // HMAC-SHA256 от остальных полей (base64)
}
