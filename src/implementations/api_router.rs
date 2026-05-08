use crate::implementations::load_configurations::load_configs;
use crate::models::load_properties::Properties;
use crate::models::user_model::{
    CheckPaymentIntentQuery, GetAllUsersByLimitParam, HtmlEmailRequest, PayViaQrPhRequestPayload,
    UpdatePlanRequest,
};
use crate::utils::api_key::generate_api_key;
use crate::{
    implementations::user_crud_service::{
        get_all_users_with_limit, get_user, increment_usage_safe,
    },
    models::user_model::{CreateUserRequest, UserDto},
};
use actix_web::{delete, get, post, web, HttpRequest, HttpResponse, Responder, ResponseError};
use log::info;
use sqlx::PgPool;
use std::fmt::{self};
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
    println!("health_check_route called");

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
    info!(
        "inbound_payload: {:?}",
        serde_json::to_string(&body).unwrap_or_else(|_| "Failed to serialize payload".to_string())
    );

    let id = uuid::Uuid::new_v4();
    let api_key = format!("{}{}", generate_api_key(), id);

    let result = sqlx::query(
        r#"
        INSERT INTO email_user_tbl (id, full_name, api_key, plan_limit, used_count)
        VALUES ($1, $2, $3, $4, 0)
        "#,
    )
    .bind(id)
    .bind(&body.full_name)
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
    println!("get_all_users_with_limit_route called");
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
    info!("get_user_route called");
    println!("get_user_route called");
    let api_key = extract_api_key(&req)?;

    let user = sqlx::query_as::<_, UserDto>(
        r#"
        SELECT full_name, plan_limit, used_count
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
    info!("update_plan_route called");
    info!(
        "inbound_payload: {:?}",
        serde_json::to_string(&body).unwrap_or_else(|_| "Failed to serialize payload".to_string())
    );
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
        RETURNING full_name, plan_limit, used_count
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
    info!("delete_user_route called");
    println!("delete_user_route called");
    let result = sqlx::query(r#"DELETE FROM email_user_tbl WHERE id = $1"#)
        .bind(id.into_inner())
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Deleted"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[post("/create-qrph")]
pub async fn create_qrph_payment(
    payload: web::Json<PayViaQrPhRequestPayload>,
) -> Result<HttpResponse, AppValidationError> {
    let client = reqwest::Client::new();
    let cfg: Properties = load_configs().expect("Failed to load configurations");

    info!("create_qrph_payment called");

    let pm_res = client
        .post(format!(
            "{}{}",
            &cfg.paymongo_url, &cfg.paymongo_create_payment_method
        ))
        .header("accept", "application/json")
        .header("content-type", "application/json")
        .header("Authorization", format!("Basic {}", cfg.paymongo_api_key))
        .json(&serde_json::json!({
            "data": {
                "attributes": {
                    "type": "qrph"
                }
            }
        }))
        .send()
        .await
        .map_err(|e| AppValidationError::from_str(e.to_string()))?;

    let text = pm_res.text().await.unwrap();

    let pm_json: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| AppValidationError::from_str(e.to_string()))?;

    let payment_method_id = pm_json["data"]["id"]
        .as_str()
        .ok_or_else(|| AppValidationError::from_str("Missing payment_method_id"))?;

    let pi_res = client
        .post(format!(
            "{}{}",
            &cfg.paymongo_url, &cfg.paymongo_create_payintents
        ))
        .header("accept", "application/json")
        .header("content-type", "application/json")
        .header("Authorization", format!("Basic {}", cfg.paymongo_api_key))
        .json(&serde_json::json!({
            "data": {
                "attributes": {
                    "amount": payload.amount,
                    "payment_method_allowed": ["qrph"],
                    "currency": payload.currency,
                    "capture_type": "automatic"
                }
            }
        }))
        .send()
        .await
        .map_err(|e| AppValidationError::from_str(e.to_string()))?;

    let pi_json: serde_json::Value = pi_res.json().await.unwrap();

    let payment_intent_id = pi_json["data"]["id"]
        .as_str()
        .ok_or_else(|| AppValidationError::from_str("Missing payment_intent_id"))?;

    let client_key = pi_json["data"]["attributes"]["client_key"]
        .as_str()
        .ok_or_else(|| AppValidationError::from_str("Missing client_key"))?;

    let attach_url = format!(
        "{}{}/{}/attach",
        &cfg.paymongo_url, &cfg.paymongo_create_payintents, payment_intent_id
    );

    let attach_res = client
        .post(&attach_url)
        .header("accept", "application/json")
        .header("content-type", "application/json")
        .header("Authorization", format!("Basic {}", cfg.paymongo_api_key))
        .json(&serde_json::json!({
            "data": {
                "attributes": {
                    "payment_method": payment_method_id,
                    "client_key": client_key
                }
            }
        }))
        .send()
        .await
        .map_err(|e| AppValidationError::from_str(e.to_string()))?;

    let attach_json: serde_json::Value = attach_res.json().await.unwrap();

    let qr_url = attach_json["data"]["attributes"]["next_action"]["code"]["image_url"]
        .as_str()
        .ok_or_else(|| AppValidationError::from_str("QR URL not found"))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "qrph_image_url": qr_url,
        "payment_intent_id": payment_intent_id,
        "client_key": client_key
    })))
}

#[get("/payment-intent/status")]
pub async fn get_payment_intent_status(
    query: web::Query<CheckPaymentIntentQuery>,
) -> Result<HttpResponse, AppValidationError> {

    info!("get_payment_intent_status called");

    let cfg: Properties = load_configs().expect("Failed to configuration properties.");
    let client = reqwest::Client::new();

    let url = format!(
        "{}{}/{}?client_key={}",
        &cfg.paymongo_url,
        &cfg.paymongo_create_payintents,
        query.payment_intent_id,
        query.client_key
    );

    let res = client
        .get(&url)
        .header("accept", "application/json")
        .header("Authorization", format!("Basic {}", cfg.paymongo_api_key))
        .send()
        .await
        .map_err(|e| AppValidationError::from_str(e.to_string()))?;

    let json: serde_json::Value = res
        .json()
        .await
        .map_err(|e| AppValidationError::from_str(e.to_string()))?;

    let status = json["data"]["attributes"]["status"]
        .as_str()
        .unwrap_or("unknown");

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "payment_intent_id": query.payment_intent_id,
        "status": status,
        "raw": json
    })))
}

#[post("/send-email-html")]
pub async fn send_email_html_route(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    payload: web::Json<HtmlEmailRequest>,
) -> Result<HttpResponse, AppValidationError> {
    info!("send_email_html_route called");

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
        Err(e) => return Ok(HttpResponse::InternalServerError().body(e.to_string())),
    };

    payload
        .validate()
        .map_err(AppValidationError::from_validator_err)?;

    let cfg: Properties = load_configs().expect("Failed to load configuration");

    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "from": format!("{}{}{}","ZentroMail <", cfg.resend_email,">"),
        "to": payload.to,
        "subject": payload.subject,
        "html": payload.html,
        "reply_to": payload.from
    });

    let response = client
        .post(cfg.resend_url)
        .bearer_auth(cfg.resend_token)
        .json(&body)
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "remaining_limit": updated_user.plan_limit - updated_user.used_count
        }))),
        Ok(resp) => {
            let err_text = resp.text().await.unwrap_or_default();
            Ok(HttpResponse::BadRequest().body(err_text))
        }
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
