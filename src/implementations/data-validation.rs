use actix_web::{HttpResponse, ResponseError};
use validator::ValidationErrors;
use std::fmt;

#[derive(Debug)]
pub struct ValidationError(pub ValidationErrors);

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "validation error: {}", self.0)
    }
}

impl ResponseError for ValidationError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid input",
            "details": self.0.field_errors()
        }))
    }
}