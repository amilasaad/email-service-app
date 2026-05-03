use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Properties {
    pub host: String,
    pub port: u16,

    pub smtp_host: String,
    pub smtp_port: String,
    pub smtp_user: String,
    pub smtp_pass: String,

    pub db_url: String,
    pub self_check_url: String
}