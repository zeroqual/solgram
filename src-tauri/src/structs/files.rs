use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub initialized: bool,
    pub password_hash: Option<String>,
    pub salt: Option<String>,
    pub encrypted_private_key: Option<String>,
}
