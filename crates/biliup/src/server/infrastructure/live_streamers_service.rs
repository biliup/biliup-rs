use crate::server::core::live_streamers::{
    AddLiveStreamerDto, DynLiveStreamersRepository, LiveStreamerDto, LiveStreamersService,
};
use async_trait::async_trait;

#[derive(Clone)]
pub struct ConduitLiveStreamersService {
    repository: DynLiveStreamersRepository,
}

impl ConduitLiveStreamersService {
    pub fn new(repository: DynLiveStreamersRepository) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl LiveStreamersService for ConduitLiveStreamersService {
    async fn add_streamer(&self, request: AddLiveStreamerDto) -> anyhow::Result<LiveStreamerDto> {
        Ok(self
            .repository
            .create_streamer(&request.url, &request.remark)
            .await?
            .into_dto())
    }

    async fn get_streamers(&self) -> anyhow::Result<Vec<LiveStreamerDto>> {
        Ok(self
            .repository
            .get_streamers()
            .await?
            .into_iter()
            .map(|s| s.into_dto())
            .collect())
    }
}
