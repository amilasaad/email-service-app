use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, FromRow, Serialize)]
pub struct UserDao {
    pub id: Uuid,
    pub full_name: String,
    pub api_key: String,
    pub plan_limit: i64,
    pub used_count: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize, Serialize)]
pub struct CreateUserRequest {
    pub full_name: String,
    pub plan_limit: i64
}

#[derive(Debug, serde::Serialize, FromRow)]
pub struct UserDto {
    pub full_name: Option<String>,
    pub plan_limit: i64,
    pub used_count: i64
}

#[derive(Deserialize, Serialize)]
pub struct UpdatePlanRequest {
    pub plan_limit: i32,
}

#[derive(Deserialize, Serialize, validator::Validate)]
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

#[derive(serde::Deserialize)]
pub struct CheckPaymentIntentQuery {
    pub payment_intent_id: String,
    pub client_key: String,
}

#[derive(serde::Deserialize)]
pub struct PayViaQrPhRequestPayload {
    pub amount: i64,
    pub currency: String
}