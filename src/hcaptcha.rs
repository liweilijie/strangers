use crate::error::{AppError, AppErrorType};
use crate::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

#[derive(Serialize)]
pub struct VerifyRequest {
    pub secret: String,
    pub response: String,
}

#[derive(Deserialize)]
pub struct VerifyResponse {
    pub success: bool,
}

pub async fn verify(response: String, secret: String) -> Result<bool> {
    let req = VerifyRequest { secret, response };
    let client = reqwest::Client::new();
    let res = client
        .post("https://hcaptcha.com/siteverify")
        .form(&req)
        .send()
        .await
        .map_err(|err| {
            error!(" POST {:?}", err);
            AppError::from_err(err, AppErrorType::HttpError)
        })?;
    let res = res.text().await.map_err(|err| {
        error!(" GET TEXT error {:?}", err);
        AppError::from_err(err, AppErrorType::HttpError)
    })?;
    debug!("hcaptch res: {:?}", res);
    let res: VerifyResponse = serde_json::from_str(&res).map_err(|err| {
        error!(" serde_json Deserialize error {:?}", err);
        AppError::from_err(err, AppErrorType::HttpError)
    })?;
    Ok(res.success)
}
