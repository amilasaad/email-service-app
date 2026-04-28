use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, FromRow, Serialize)]
pub struct UserDao {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub api_key: String,
    pub plan_limit: i64,
    pub used_count: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password_hash: String,
    pub plan_limit: i64,
}

#[derive(Debug, serde::Serialize, FromRow)]
pub struct UserDto {
    pub username: Option<String>,
    pub plan_limit: i64,
    pub used_count: i64
}

#[derive(Deserialize)]
pub struct UpdatePlanRequest {
    pub plan_limit: i32,
}

#[derive(Deserialize, validator::Validate)]
pub struct HtmlEmailRequest {
    #[validate(email)]
    pub from: String,

    #[validate(email)]
    pub to: String,

    pub subject: String,

    pub html: String,
}

#[derive(Deserialize)]
pub struct GetAllUsersByLimitParam {
    pub limit: i64
}