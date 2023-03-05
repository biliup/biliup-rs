pub mod errors;

pub mod api {
    pub mod bilibili_endpoints;
    pub mod endpoints;
    pub mod router;
}

pub mod core;

pub mod infrastructure {
    pub mod repositories {
        pub mod download_records_repository;
        pub mod live_streamers_repository;
        pub mod upload_records_repository;
        pub mod upload_streamers_repository;
        pub mod users_repository;
        pub mod videos_repository;
    }

    pub mod connection_pool;
    pub mod live_streamers_service;
    pub mod service_register;
}
