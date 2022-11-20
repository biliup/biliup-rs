use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;

/// Similar to above, we want to keep a reference count across threads so we can manage our connection pool.
pub type DynLiveStreamersRepository = Arc<dyn LiveStreamersRepository + Send + Sync>;

#[async_trait]
pub trait LiveStreamersRepository {
    async fn create_streamer(&self, url: &str, remark: &str) -> anyhow::Result<LiveStreamerEntity>;
    async fn get_streamers(&self) -> anyhow::Result<Vec<LiveStreamerEntity>>;
}

#[derive(FromRow)]
pub struct LiveStreamerEntity {
    pub id: i64,
    pub url: String,
    pub remark: String,
}

impl LiveStreamerEntity {
    pub fn into_dto(self) -> LiveStreamerDto {
        LiveStreamerDto {
            id: self.id,
            url: self.url,
            remark: self.remark,
        }
    }

    // pub fn into_profile(self, following: bool) -> ProfileDto {
    //     ProfileDto {
    //         username: self.username,
    //         bio: self.bio,
    //         image: self.image,
    //         following,
    //     }
    // }
}

/// A reference counter for our user service allows us safely pass instances user utils
/// around which themselves depend on the user repostiory, and ultimately, our Posgres connection pool.
pub type DynLiveStreamersService = Arc<dyn LiveStreamersService + Send + Sync>;

#[async_trait]
pub trait LiveStreamersService {
    async fn add_streamer(&self, request: AddLiveStreamerDto) -> anyhow::Result<LiveStreamerDto>;
    async fn get_streamers(&self) -> anyhow::Result<Vec<LiveStreamerDto>>;
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct AddLiveStreamerDto {
    pub url: String,
    pub remark: String,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct LiveStreamerDto {
    pub id: i64,
    pub url: String,
    pub remark: String,
}
