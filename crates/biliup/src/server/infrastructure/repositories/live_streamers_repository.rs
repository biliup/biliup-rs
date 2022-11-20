use crate::server::core::live_streamers::{LiveStreamerEntity, LiveStreamersRepository};
use crate::server::infrastructure::connection_pool::ConnectionPool;
use anyhow::Context;
use async_trait::async_trait;
use sqlx::query_as;

#[derive(Clone)]
pub struct SqliteLiveStreamersRepository {
    pool: ConnectionPool,
}

impl SqliteLiveStreamersRepository {
    pub fn new(pool: ConnectionPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LiveStreamersRepository for SqliteLiveStreamersRepository {
    async fn create_streamer(&self, url: &str, remark: &str) -> anyhow::Result<LiveStreamerEntity> {
        query_as!(
            LiveStreamerEntity,
            r#"
        insert into live_streamers (url, remark)
        values ($1 , $2 )
        returning id, url as "url!", remark as "remark!"
            "#,
            url,
            remark
        )
        .fetch_one(&self.pool)
        .await
        .context("an unexpected error occurred while creating the streamer")
    }

    async fn get_streamers(&self) -> anyhow::Result<Vec<LiveStreamerEntity>> {
        query_as!(
            LiveStreamerEntity,
            r#"
       select * from live_streamers
            "#
        )
        .fetch_all(&self.pool)
        .await
        .context("an unexpected error occurred retrieving streamers")
    }
}
