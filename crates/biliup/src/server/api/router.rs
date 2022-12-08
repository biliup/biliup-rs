use crate::client::StatelessClient;

use crate::server::api::endpoints::{add_streamer_endpoint, get_streamers_endpoint, root};
use crate::server::core::download_actor::DownloadActorHandle;

use crate::server::infrastructure::service_register::ServiceRegister;
use anyhow::Context;

use axum::routing::{get, post};
use axum::{Extension, Router};

use std::net::SocketAddr;
use std::time::Duration;
use tracing::info;

pub struct ApplicationController;

impl ApplicationController {
    pub async fn serve(addr: &SocketAddr, service_register: ServiceRegister) -> anyhow::Result<()> {
        // build our application with a route
        let app = Router::new()
            // `GET /` goes to `root`
            .route("/streamers", get(get_streamers_endpoint))
            .route("/streamers", post(add_streamer_endpoint))
            .route("/", get(root))
            .layer(Extension(service_register.streamers_service.clone()));
        // `POST /users` goes to `create_user`
        // .route("/users", post(create_user));
        // let mut test = Cycle::new(indexmap![
        //     "1".to_string()  => StreamStatus::Idle,
        //     "2".to_string() => StreamStatus::Idle,
        //     "3".to_string() => StreamStatus::Idle,
        //     "4".to_string() => StreamStatus::Idle,
        // ]);
        // let testget = test.clone();
        let client = StatelessClient::default();
        let vec = service_register.streamers_service.get_streamers().await?;
        let actor_handle = DownloadActorHandle::new(vec, client);
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(31)).await;
            actor_handle.remove_streamer("https://www.huya.com/superrabbit");
            println!("yesssss")
        });
        // let mut n = 0;
        // loop {
        //     let string = testget.get(&mut n);
        //     println!("yoooo {string:?}");
        //     tokio::time::sleep(Duration::from_secs(2)).await;
        // }
        // extensions.get().and_then(|actor| {
        //    actor
        // });
        // println!("nonono");
        // Ok::<_, anyhow::Error>(())

        // tokio::spawn(async move {
        //     tokio::time::sleep(Duration::from_secs(10)).await;
        //     test.replace(indexmap!["10".to_string()  => StreamStatus::Idle]);
        // });
        // run our app with hyper
        // `axum::Server` is a re-export of `hyper::Server`
        info!("routes initialized, listening on {}", addr);
        axum::Server::bind(addr)
            .serve(app.into_make_service())
            .await
            .context("error while starting API server")?;

        Ok(())
    }
}
