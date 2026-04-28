use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn connect_db(database_url: &str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .expect("Failed to connect to database")
}