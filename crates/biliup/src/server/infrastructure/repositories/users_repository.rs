use crate::server::core::users::{User, UsersRepository};
use crate::server::infrastructure::connection_pool::ConnectionPool;
use async_trait::async_trait;

#[derive(Clone)]
pub struct SqliteUsersStreamersRepository {
    pool: ConnectionPool,
}

impl SqliteUsersStreamersRepository {
    pub fn new(pool: ConnectionPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UsersRepository for SqliteUsersStreamersRepository {
    async fn create_user(&self, user: User) -> anyhow::Result<User> {
        todo!()
    }

    async fn get_users(&self) -> anyhow::Result<Vec<User>> {
        todo!()
    }

    async fn delete_user(&self, id: i64) -> anyhow::Result<()> {
        todo!()
    }

    async fn update_user(&self, user: User) -> anyhow::Result<User> {
        todo!()
    }

    async fn get_user_by_id(&self, id: i64) -> anyhow::Result<User> {
        todo!()
    }
}