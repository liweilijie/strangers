use serde::Deserialize;

#[derive(Deserialize)]
pub struct AdminLogin {
    pub username: String,
    pub password: String,
    pub hcaptcha_response: String,
}
