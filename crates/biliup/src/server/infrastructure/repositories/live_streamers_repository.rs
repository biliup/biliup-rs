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
        returning id as "id: u32", url as "url!", remark as "remark!", upload_id as "upload_id: u32"
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
       select id as "id: u32", url as "url!", remark as "remark!", upload_id as "upload_id: u32" from live_streamers
            "#
        )
        .fetch_all(&self.pool)
        .await
        .context("an unexpected error occurred retrieving streamers")
    }

    async fn get_streamer_by_url(&self, url: &str) -> anyhow::Result<LiveStreamerEntity> {
        query_as!(
            LiveStreamerEntity,
            r#"
        select
            id as "id: u32", url as "url!", remark as "remark!", upload_id as "upload_id: u32"
        from
            live_streamers
        where
            url=$1
            "#,
            url
        )
        .fetch_one(&self.pool)
        .await
        .context("an unexpected error occurred while creating the streamer")
    }
}
