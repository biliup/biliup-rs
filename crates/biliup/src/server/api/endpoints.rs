use crate::server::core::live_streamers::{
    AddLiveStreamerDto, DynLiveStreamersService, LiveStreamerDto,
};
use crate::server::errors::AppResult;
use axum::{Extension, Json};
use crate::server::core::upload_streamers::{DynUploadStreamersRepository, StudioEntity};
use crate::uploader::bilibili::Studio;

// basic handler that responds with a static string
pub async fn root() -> &'static str {
    "Hello, World!"
}

pub async fn get_streamers_endpoint(
    Extension(streamers_service): Extension<DynLiveStreamersService>,
) -> AppResult<Json<Vec<LiveStreamerDto>>> {
    Ok(Json(streamers_service.get_streamers().await?))
}

pub async fn add_streamer_endpoint(
    Extension(streamers_service): Extension<DynLiveStreamersService>,
    Json(request): Json<AddLiveStreamerDto>,
) -> AppResult<Json<LiveStreamerDto>> {
    Ok(Json(streamers_service.add_streamer(request).await?))
}

pub async fn add_upload_streamer_endpoint(
    Extension(streamers_service): Extension<DynUploadStreamersRepository>,
    Json(request): Json<Studio>,
) -> AppResult<Json<StudioEntity>> {
    Ok(Json(streamers_service.create_streamer(request).await?))
}

pub async fn get_upload_streamers_endpoint(
    Extension(streamers_service): Extension<DynUploadStreamersRepository>,
) -> AppResult<Json<Vec<StudioEntity>>> {
    Ok(Json(streamers_service.get_streamers().await?))
}
