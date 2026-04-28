use actix_web::{web, App, HttpServer};
mod implementations;
mod configurations;
mod models;
mod utils;

use std::sync::Arc;
use configurations::db_configurations::connect_db;
use configurations::email_config::build_mailer;
use implementations::load_configurations::load_configs;

use crate::models::load_properties::Properties;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cfg: Properties = load_configs()
        .expect("Failed to load configuration");

    log::info!("Configuration loaded: {:?}", cfg.host);

    let mailer = build_mailer(cfg.clone());

    let pool = connect_db(&cfg.db_url).await;

    let cfg_data = web::Data::new(Arc::new(cfg));

    let pool_data = web::Data::new(pool);

    let host = cfg_data.get_ref().host.clone();
    let port = cfg_data.get_ref().port;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(mailer.clone()))
            .app_data(cfg_data.clone())
            .app_data(pool_data.clone())

            .service(implementations::api_router::send_email_route)
            .service(implementations::api_router::health_check_route)
            .service(implementations::api_router::create_user_route)
            .service(implementations::api_router::get_user_route)
            .service(implementations::api_router::delete_user_route)
            .service(implementations::api_router::update_plan_route)
            .service(implementations::api_router::send_email_html_route)
            .service(implementations::api_router::get_all_users_with_limit_route)
    })
    .bind((host, port))?
    .run()
    .await
}