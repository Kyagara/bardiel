use crate::connection::Client;
use anyhow::Result;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;

pub struct Proxy {
    pub listener: TcpListener,
    pub config: Arc<Config>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub bind: String,
    pub server_address: String,
    pub max_players: i32,
    pub description: String,
    pub server_icon: Option<String>,
}

impl Proxy {
    pub async fn listen(self) -> Result<()> {
        loop {
            match self.listener.accept().await {
                Ok((stream, address)) => {
                    info!("New client: \"{address}\".");

                    let (read, write) = stream.into_split();

                    let proxy_config = Arc::clone(&self.config);

                    let connection = Client {
                        read,
                        write,
                        address,
                        proxy_config,
                    };

                    connection.start().await?;
                }
                Err(err) => error!("Error getting client: {err:?}."),
            }
        }
    }
}
