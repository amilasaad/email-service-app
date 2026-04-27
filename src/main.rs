use actix_web::{web, App, HttpServer};
mod implementations;
mod configurations;
mod models;

use std::sync::Arc;
use configurations::email_config::build_mailer;
use implementations::api_router::send_email;
use implementations::api_router::health_check;
use implementations::load_configurations::load_configs;

use crate::models::load_properties::Properties;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    let cfg: Properties = load_configs().expect("Failed to load configuration");
    log::info!("Configuration loaded: {:?}", cfg.host);
    let mailer = build_mailer(cfg.clone());

    let cfg_data = web::Data::new(Arc::new(cfg.clone()));
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(mailer.clone()))
            .service(send_email)
            .service(health_check)
    })
    .bind((cfg_data.host.clone(), cfg_data.get_ref().port))?
    .run()
    .await
}