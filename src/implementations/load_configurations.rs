use crate::models::load_properties::Properties;

pub fn load_configs() -> Result<Properties, std::env::VarError> {
    dotenv::dotenv().expect("Failed to load .env");

    let cfg = Properties {
        host: std::env::var("APP_HOST")?,
        port: std::env::var("APP_PORT")?
            .parse()
            .expect("APP_PORT must be a number"),

        smtp_host: std::env::var("APP_SMTP_HOST")?,
        smtp_user: std::env::var("APP_SMTP_USER")?,
        smtp_pass: std::env::var("APP_SMTP_PASS")?,
        
        db_url: std::env::var("APP_DATABASE_URL")?
    };

    Ok(cfg)
}