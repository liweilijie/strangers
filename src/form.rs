use serde::Deserialize;

#[derive(Deserialize)]
pub struct AdminLogin {
    pub username: String,
    pub password: String,
    pub hcaptcha_response: String,
}

#[derive(Deserialize, Debug)]
pub struct CreateMedicinal {
    pub name: String,
    pub category: String,
    pub batch_number: String,
    pub count: String,
    pub validity: chrono::NaiveDate,
}

#[derive(Deserialize, Debug)]
pub struct UpdateMedicinal {
    pub id: i32,
    pub name: String,
    pub category: String,
    pub batch_number: String,
    pub count: String,
    pub validity: chrono::NaiveDate,
}
