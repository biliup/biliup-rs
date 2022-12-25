use crate::server::core::live_streamers::{DynLiveStreamersRepository, DynLiveStreamersService};
use crate::server::core::upload_streamers::DynUploadStreamersRepository;
use crate::server::infrastructure::connection_pool::ConnectionPool;
use crate::server::infrastructure::live_streamers_service::ConduitLiveStreamersService;
use crate::server::infrastructure::repositories::live_streamers_repository::SqliteLiveStreamersRepository;
use crate::server::infrastructure::repositories::upload_streamers_repository::SqliteUploadStreamersRepository;
use std::sync::Arc;
use tracing::info;

#[derive(Clone)]
pub struct ServiceRegister {
    pub streamers_service: DynLiveStreamersService,
    pub upload_streamers_repository: DynUploadStreamersRepository,
}

/// A simple service container responsible for managing the various services our API endpoints will pull from through axum extensions.
impl ServiceRegister {
    pub fn new(pool: ConnectionPool) -> Self {
        info!("initializing utility services...");

        info!("utility services initialized, building feature services...");
        let streamers_repository = Arc::new(SqliteLiveStreamersRepository::new(pool.clone()))
            as DynLiveStreamersRepository;
        let upload_streamers_repository =
            Arc::new(SqliteUploadStreamersRepository::new(pool)) as DynUploadStreamersRepository;

        let streamers_service = Arc::new(ConduitLiveStreamersService::new(
            streamers_repository.clone(),
            upload_streamers_repository.clone(),
        )) as DynLiveStreamersService;
        info!("feature services successfully initialized!");

        ServiceRegister {
            streamers_service,
            upload_streamers_repository,
        }
    }
}
