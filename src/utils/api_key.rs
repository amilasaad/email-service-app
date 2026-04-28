use rand::RngCore;
use base64::{engine::general_purpose, Engine as _};

pub fn generate_api_key() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    general_purpose::STANDARD.encode(bytes)
}