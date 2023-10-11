use axum::Server;
use serde_json::Value;
use tokio::sync::mpsc::Sender;

use crate::{app::routes, client::OpenAiClient, config::Config, error::Error};
use log::{error, info};

pub async fn run_service<'a>(
    tx_neo4j: Sender<Value>,
    client: OpenAiClient,
    config: Config,
) -> Result<(), anyhow::Error> {
    let mut bind = true;
    let socket_address = config.socket_address;
    let server = Server::try_bind(&socket_address)
        .or_else(|_| {
            bind = false;
            error!("Failed to bind to socket address: {}", socket_address);
            axum::Server::try_bind(&"127.0.0.1:0".parse().unwrap())
        })
        .map_err(|_| Error::FailedToStartService)?;
    let server = server.serve(routes(tx_neo4j, client).into_make_service());

    let bind_addr = if bind {
        socket_address
    } else {
        "127.0.0.1:0".parse().unwrap()
    };

    info!("Started JSON RPC service at {:?}", bind_addr);

    server.await.map_err(|_| Error::FailedToStartService)?;

    Ok(())
}