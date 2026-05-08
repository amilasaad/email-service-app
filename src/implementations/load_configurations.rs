use crate::models::load_properties::Properties;

pub fn load_configs() -> Result<Properties, std::env::VarError> {
    dotenv::dotenv().expect("Failed to load .env");

    let cfg = Properties {
        host: std::env::var("APP_HOST")?,
        port: std::env::var("APP_PORT")?
            .parse()
            .expect("APP_PORT must be a number"),

        resend_url: std::env::var("RESEND_URL")?,
        resend_token: std::env::var("RESEND_TOKEN")?,
        resend_email: std::env::var("RESEND_ONBOARDING_EMAIL")?,
        
        db_url: std::env::var("APP_DATABASE_URL")?,
        self_check_url: std::env::var("APP_SELF_CHECK_URL")?,

        paymongo_url: std::env::var("PAYMONGO_URL")?,
        paymongo_create_payment_method: std::env::var("PAYMONGO_CREATE_PAYMENT")?,
        paymongo_create_payintents: std::env::var("PAYMONGO_CREATE_PAYINTENTS")?,
        paymongo_api_key: std::env::var("PAYMONGO_API_KEY")?    
    };

    Ok(cfg)
}