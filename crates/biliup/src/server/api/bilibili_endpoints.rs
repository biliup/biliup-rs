use crate::client::StatelessClient;
use crate::server::errors::AppResult;
use crate::uploader::credential::login_by_cookies;
use axum::extract::Query;
use axum::{Extension, Json};
use bytes::Bytes;
use std::collections::HashMap;

pub async fn archive_pre_endpoint(
    Query(params): Query<HashMap<String, String>>,
) -> AppResult<Json<serde_json::Value>> {
    let bili = login_by_cookies("cookies.json").await?;
    Ok(Json(bili.archive_pre().await?))
}

pub async fn get_myinfo_endpoint(
    Query(params): Query<HashMap<String, String>>,
) -> AppResult<Json<serde_json::Value>> {
    println!("{}", &params["user"]);
    let bili = login_by_cookies(&params["user"]).await?;
    Ok(Json(bili.my_info().await?))
}

pub async fn get_proxy_endpoint(
    Extension(client): Extension<StatelessClient>,
    Query(params): Query<HashMap<String, String>>,
) -> AppResult<Bytes> {
    println!("{}", &params["url"]);
    // let bili = login_by_cookies(&params["user"]).await?;
    Ok(client
        .client
        .get(&params["url"])
        .send()
        .await?
        .bytes()
        .await?)
}
