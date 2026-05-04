use actix_web::{web, App, HttpServer};
mod implementations;
mod configurations;
mod models;
mod utils;

use std::sync::Arc;
use configurations::db_configurations::connect_db;
use implementations::load_configurations::load_configs;
use log::info;
use std::time::Duration;
use crate::models::load_properties::Properties;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cfg: Properties = load_configs()
        .expect("Failed to load configuration");

    info!("Configuration loaded: {:?}", cfg.host);
    println!("Configuration loaded: {:?}", cfg.host);

    let pool = connect_db(&cfg.db_url).await;

    let cfg_data = web::Data::new(Arc::new(cfg));

    let pool_data = web::Data::new(pool);

    let host = cfg_data.get_ref().host.clone();
    let port = cfg_data.get_ref().port;
    let self_check_url = cfg_data.get_ref().self_check_url.clone();

    tokio::spawn(async move {
        let client = reqwest::Client::new();

        loop {
            match client
                .get(&self_check_url)
                .send()
                .await
            {
                Ok(_) => println!("Ping success"),
                Err(e) => println!("Ping failed: {:?}", e)
            }

            tokio::time::sleep(Duration::from_secs(120)).await;
        }
    });


    HttpServer::new(move || {
        App::new()
            .app_data(cfg_data.clone())
            .app_data(pool_data.clone())

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