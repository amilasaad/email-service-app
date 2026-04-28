use crate::models::user_model::{GetAllUsersByLimitParam, HtmlEmailRequest, UpdatePlanRequest};
use crate::utils::api_key::generate_api_key;
use crate::{
    implementations::user_crud_service::{get_user, increment_usage_safe, get_all_users_with_limit},
    models::{
        deserializer::EmailRequest,
        user_model::{CreateUserRequest, UserDto},
    },
};
use actix_web::{HttpRequest, HttpResponse, Responder, ResponseError, delete, get, post, web};
use lettre::message::{SinglePart, header::ContentType};
use lettre::{Message, Transport};
use log::{info};
use sqlx::PgPool;
use std::fmt;
use validator::{Validate, ValidationErrors};

#[derive(Debug)]
pub struct AppValidationError {
    pub message: String,
}

impl fmt::Display for AppValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "validation error: {}", self.message)
    }
}

impl ResponseError for AppValidationError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid input",
            "details": self.message
        }))
    }
}

impl AppValidationError {
    fn from_validator_err(err: ValidationErrors) -> Self {
        Self {
            message: format!("{:?}", err),
        }
    }

    fn from_str(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

#[get("/health-check")]
pub async fn health_check_route() -> impl Responder {
    info!("health_check_route called");

    HttpResponse::Ok()
        .content_type("application/json")
        .json(serde_json::json!({ "status": "Im good!" }))
}

#[post("/users")]
pub async fn create_user_route(
    pool: web::Data<PgPool>,
    body: web::Json<CreateUserRequest>,
) -> HttpResponse {
    info!("create_user_route called");

    let id = uuid::Uuid::new_v4();
    let api_key = format!("{}{}", generate_api_key(), id);

    let result = sqlx::query(
        r#"
        INSERT INTO email_user_tbl (id, username, password_hash, api_key, plan_limit, used_count)
        VALUES ($1, $2, $3, $4, $5, 0)
        "#,
    )
    .bind(id)
    .bind(&body.username)
    .bind(&body.password_hash)
    .bind(&api_key)
    .bind(body.plan_limit)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "User created",
            "api_key": api_key
        })),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("/users/all")]
pub async fn get_all_users_with_limit_route(
    pool: web::Data<PgPool>,
    query: web::Query<GetAllUsersByLimitParam>,
) -> Result<HttpResponse, AppValidationError> {
    info!("get_all_users_with_limit_route called");
    match get_all_users_with_limit(pool.get_ref(), &query.limit).await {
        Ok(d) => Ok(HttpResponse::Ok().json(d)),
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

#[get("/user")]
pub async fn get_user_route(
    req: HttpRequest,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppValidationError> {
    let api_key = extract_api_key(&req)?;

    let user = sqlx::query_as::<_, UserDto>(
        r#"
        SELECT username, plan_limit, used_count
        FROM email_user_tbl
        WHERE api_key = $1
        "#,
    )
    .bind(api_key)
    .fetch_one(pool.get_ref())
    .await;

    match user {
        Ok(u) => Ok(HttpResponse::Ok().json(u)),
        Err(_) => Ok(HttpResponse::NotFound().body("User not found")),
    }
}

#[post("/users/update-plan")]
pub async fn update_plan_route(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<UpdatePlanRequest>,
) -> Result<HttpResponse, AppValidationError> {
    let api_key = req
        .headers()
        .get("x-api-key")
        .ok_or_else(|| AppValidationError::from_str("Missing API key"))?
        .to_str()
        .map_err(|_| AppValidationError::from_str("Invalid API key"))?;

    let result = sqlx::query_as::<_, UserDto>(
        r#"
        UPDATE email_user_tbl
        SET plan_limit = $1
        WHERE api_key = $2
        RETURNING username, plan_limit, used_count
        "#,
    )
    .bind(body.plan_limit)
    .bind(api_key)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(user) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Plan updated",
            "user": user
        }))),
        Err(sqlx::Error::RowNotFound) => Ok(HttpResponse::NotFound().body("User not found")),
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

#[delete("/users/{id}")]
pub async fn delete_user_route(pool: web::Data<PgPool>, id: web::Path<uuid::Uuid>) -> HttpResponse {
    let result = sqlx::query(r#"DELETE FROM email_user_tbl WHERE id = $1"#)
        .bind(id.into_inner())
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Deleted"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[post("/send-email")]
pub async fn send_email_route(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    mailer: web::Data<lettre::SmtpTransport>,
    payload: web::Json<EmailRequest>,
) -> Result<HttpResponse, AppValidationError> {
    let api_key = extract_api_key(&req)?;

    let _user = match get_user(pool.get_ref(), &api_key).await {
        Ok(u) => u,
        Err(_) => {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "User not found"
            })));
        }
    };

    let updated_user = match increment_usage_safe(pool.get_ref(), &api_key).await {
        Ok(u) => u,
        Err(sqlx::Error::RowNotFound) => {
            return Ok(HttpResponse::Forbidden().json(serde_json::json!({
                "error": "User has reached the limit"
            })));
        }
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().body(e.to_string()));
        }
    };

    payload
        .validate()
        .map_err(AppValidationError::from_validator_err)?;

    let email =
        Message::builder()
            .from(payload.from.parse().map_err(|e| {
                AppValidationError::from_str(format!("Invalid sender address: {}", e))
            })?)
            .to(payload.to.parse().map_err(|e| {
                AppValidationError::from_str(format!("Invalid recipient address: {}", e))
            })?)
            .subject(&payload.subject)
            .body(payload.body.clone())
            .map_err(|e| AppValidationError::from_str(format!("Malformed email fields: {}", e)))?;

    match mailer.send(&email) {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "remaining_limit": updated_user.plan_limit - updated_user.used_count
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

#[post("/send-email-html")]
pub async fn send_email_html_route(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    mailer: web::Data<lettre::SmtpTransport>,
    payload: web::Json<HtmlEmailRequest>,
) -> Result<HttpResponse, AppValidationError> {
    let api_key = extract_api_key(&req)?;

    let _user = match get_user(pool.get_ref(), &api_key).await {
        Ok(u) => u,
        Err(_) => {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "User not found"
            })));
        }
    };

    let updated_user = match increment_usage_safe(pool.get_ref(), &api_key).await {
        Ok(u) => u,
        Err(sqlx::Error::RowNotFound) => {
            return Ok(HttpResponse::Forbidden().json(serde_json::json!({
                "error": "User has reached the limit"
            })));
        }
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().body(e.to_string()));
        }
    };

    payload
        .validate()
        .map_err(AppValidationError::from_validator_err)?;

    let email =
        Message::builder()
            .from(payload.from.parse().map_err(|e| {
                AppValidationError::from_str(format!("Invalid sender address: {}", e))
            })?)
            .to(payload.to.parse().map_err(|e| {
                AppValidationError::from_str(format!("Invalid recipient address: {}", e))
            })?)
            .subject(&payload.subject)
            .singlepart(
                SinglePart::builder()
                    .header(ContentType::TEXT_HTML)
                    .body(payload.html.clone()),
            )
            .map_err(|e| AppValidationError::from_str(format!("Malformed email: {}", e)))?;

    match mailer.send(&email) {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "remaining_limit": updated_user.plan_limit - updated_user.used_count
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

fn extract_api_key(req: &HttpRequest) -> Result<String, AppValidationError> {
    req.headers()
        .get("x-api-key")
        .ok_or_else(|| AppValidationError::from_str("Missing API key"))?
        .to_str()
        .map(|s| s.to_string())
        .map_err(|_| AppValidationError::from_str("Invalid API key"))
}
