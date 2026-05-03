use crate::models::load_properties::Properties;

use lettre::{SmtpTransport};
use lettre::transport::smtp::authentication::Credentials;

pub fn build_mailer(prop: Properties) -> SmtpTransport {
    let smtp_host = prop.smtp_host;
    let smpt_port = prop.smtp_port.parse().unwrap();
    let smtp_user = prop.smtp_user;
    let smtp_pass = prop.smtp_pass;

    let creds = Credentials::new(smtp_user, smtp_pass);

    SmtpTransport::relay(&smtp_host)
        .expect("Failed to create SMTP relay")
        .credentials(creds)
        .port(smpt_port)
        .build()
}