use crate::server::errors::AppResult;
use crate::uploader::credential::login_by_cookies;
use axum::Json;

pub async fn archive_pre_endpoint() -> AppResult<Json<serde_json::Value>> {
    let bili = login_by_cookies("cookies.json").await?;

    Ok(Json(bili.archive_pre().await?))
}
