use actix_web::{post, get, web, HttpResponse, Responder, ResponseError};
use lettre::{Message, Transport};
use crate::models::deserializer::EmailRequest;
use validator::{Validate, ValidationErrors};
use std::fmt;

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

#[post("/send-email")]
pub async fn send_email(
    mailer: web::Data<lettre::SmtpTransport>,
    payload: web::Json<EmailRequest>,
) -> Result<HttpResponse, AppValidationError> {
    log::info!(">> Send email");
    payload
        .validate()
        .map_err(AppValidationError::from_validator_err)?;

    let email = Message::builder()
        .from(
            payload
                .from
                .parse()
                .map_err(|e| AppValidationError::from_str(format!("Invalid sender address: {}", e)))?,
        )
        .to(
            payload
                .to
                .parse()
                .map_err(|e| AppValidationError::from_str(format!("Invalid recipient address: {}", e)))?,
        )
        .subject(&payload.subject)
        .body(payload.body.clone())
        .map_err(|e| AppValidationError::from_str(format!("Malformed email fields: {}", e)))?;

    match mailer.send(&email) {
        Ok(_) => Ok(HttpResponse::Ok()
            .json(serde_json::json!({ "status": "Success" }))),
        Err(e) => Ok(HttpResponse::InternalServerError()
            .body(format!("Failed to send email: {}", e))),
    }
}

#[get("/health-check")]
pub async fn health_check() -> impl Responder {
    log::info!(">> Health Check");
    HttpResponse::Ok()
        .content_type("application/json")
        .json(serde_json::json!({ "status": "Im good!"}))
}