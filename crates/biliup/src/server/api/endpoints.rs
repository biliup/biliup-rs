use crate::server::core::download_actor::DownloadActorHandle;
use crate::server::core::live_streamers::{
    AddLiveStreamerDto, DynLiveStreamersService, LiveStreamerDto,
};
use crate::server::core::upload_streamers::{DynUploadStreamersRepository, StudioEntity};
use crate::server::errors::AppResult;
use crate::uploader::bilibili::Studio;
use axum::{Extension, Json};

// basic handler that responds with a static string
pub async fn root() -> &'static str {
    "Hello, World!"
}

pub async fn get_streamers_endpoint(
    Extension(streamers_service): Extension<DynLiveStreamersService>,
    Extension(download_actor_handle): Extension<DownloadActorHandle>,
) -> AppResult<Json<Vec<LiveStreamerDto>>> {
    let map = download_actor_handle.get_streamers();
    let mut vec = streamers_service.get_streamers().await?;
    for live in vec.iter_mut() {
        live.status = map.get(&live.url).copied().unwrap_or_default()
    }
    Ok(Json(vec))
}

pub async fn add_streamer_endpoint(
    Extension(streamers_service): Extension<DynLiveStreamersService>,
    Extension(download_actor_handle): Extension<DownloadActorHandle>,
    Json(request): Json<AddLiveStreamerDto>,
) -> AppResult<Json<LiveStreamerDto>> {
    download_actor_handle.add_streamer(&request.url);
    Ok(Json(streamers_service.add_streamer(request).await?))
}

pub async fn add_upload_streamer_endpoint(
    Extension(streamers_service): Extension<DynUploadStreamersRepository>,
    Json(request): Json<StudioEntity>,
) -> AppResult<Json<StudioEntity>> {
    Ok(Json(streamers_service.create_streamer(request).await?))
}

pub async fn get_upload_streamers_endpoint(
    Extension(streamers_service): Extension<DynUploadStreamersRepository>,
) -> AppResult<Json<Vec<StudioEntity>>> {
    Ok(Json(streamers_service.get_streamers().await?))
}
