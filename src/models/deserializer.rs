use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Serialize, Validate)]
pub struct EmailRequest {

    #[validate(email, length(min = 3, max = 100))]
    pub from: String,

    #[validate(email, length(min = 3, max = 100))]
    pub to: String,

    #[validate(length(min = 3, max = 200))]
    pub subject: String,

    #[validate(length(min = 3, max = 100))]
    pub body: String,
}
