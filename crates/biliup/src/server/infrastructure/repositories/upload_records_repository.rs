use crate::server::core::upload_streamers::{UploadRecords, UploadRecordsRepository};
use crate::server::core::users::{User, UsersRepository};
use crate::server::infrastructure::connection_pool::ConnectionPool;
use async_trait::async_trait;

#[derive(Clone)]
pub struct SqliteUploadRecordsRepository {
    pool: ConnectionPool,
}

impl SqliteUploadRecordsRepository {
    pub fn new(pool: ConnectionPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UploadRecordsRepository for SqliteUploadRecordsRepository {
    async fn create(&self, entity: UploadRecords) -> anyhow::Result<UploadRecords> {
        todo!()
    }

    async fn delete(&self, id: i64) -> anyhow::Result<()> {
        todo!()
    }

    async fn get_all(&self) -> anyhow::Result<Vec<UploadRecords>> {
        todo!()
    }

    async fn get_by_id(&self, id: i64) -> anyhow::Result<UploadRecords> {
        todo!()
    }
}
