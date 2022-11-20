use anyhow::{Context, Result};
use axum::routing::get;
use axum::Router;
use biliup::server::api::router::ApplicationController;
use biliup::server::infrastructure::connection_pool::ConnectionManager;
use biliup::server::infrastructure::service_register::ServiceRegister;
use std::net::{IpAddr, SocketAddr, TcpListener, ToSocketAddrs};
use std::str::FromStr;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub async fn run(addr: (&str, u16)) -> Result<()> {
    // let config = Arc::new(AppConfig::parse());

    tracing::info!("environment loaded and configuration parsed, initializing Postgres connection and running migrations...");
    let conn_pool = ConnectionManager::new_pool()
        .await
        .expect("could not initialize the database connection pool");

    let service_register = ServiceRegister::new(conn_pool);

    tracing::info!("migrations successfully ran, initializing axum server...");
    let addr = addr.to_socket_addrs()?.next().unwrap();
    ApplicationController::serve(&addr, service_register)
        .await
        .context("could not initialize application routes")?;
    Ok(())
}
