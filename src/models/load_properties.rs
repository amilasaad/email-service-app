use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Properties {
    pub host: String,
    pub port: u16,
    
    pub resend_url: String,
    pub resend_token: String,
    pub resend_email: String,

    pub db_url: String,
    pub self_check_url: String
}