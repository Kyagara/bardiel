use crate::{
    proxy::ProxyConfig,
    status::{ServerStatusResponse, StatusPlayers, StatusVersion},
};
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use log::{error, info, LevelFilter};
use std::{
    fs::File,
    io::{BufReader, Read},
    sync::Arc,
};
use tokio::{net::TcpListener, sync::Mutex};

mod handshake;
mod logger;
mod login;
mod protocol;
mod proxy;
mod status;

#[tokio::main]
async fn main() -> Result<()> {
    log::set_logger(&logger::LOGGER).unwrap();
    log::set_max_level(LevelFilter::Info);

    info!("Loading configuration file.");
    let file = File::open("./config.json")?;
    let reader = BufReader::new(file);
    let config: ProxyConfig = serde_json::from_reader(reader)?;

    let favicon = match config.server_icon.clone() {
        Some(path) => {
            if !path.is_empty() {
                let mut image = File::open(path)?;
                let mut buffer = Vec::new();
                image.read_to_end(&mut buffer)?;
                let encoded: String = general_purpose::STANDARD_NO_PAD.encode(buffer);
                let base64 = format!("data:image/png;base64,{}", encoded);
                Some(base64)
            } else {
                None
            }
        }
        None => None,
    };

    let status = ServerStatusResponse {
        description: config.description.clone(),
        favicon,
        players: StatusPlayers {
            max: config.max_players,
            online: 0,
            sample: vec![],
        },
        version: StatusVersion {
            name: "bardiel".to_string(),
            protocol: 762,
        },
    };

    info!("Creating listener on {:?}.", config.bind);
    let listener = TcpListener::bind(&config.bind).await?;
    info!("Listening on {:?}.", config.bind);

    let status = Arc::new(Mutex::new(status));

    loop {
        match listener.accept().await {
            Ok((stream, client_addr)) => {
                info!("New client: \"{client_addr}\".");

                let server_status = Arc::clone(&status);
                let proxy_config = config.clone();
                stream.set_nodelay(true)?;

                tokio::spawn(async move {
                    let ip = client_addr.ip();

                    if let Err(err) =
                        proxy::handle_connection(stream, ip, proxy_config, server_status).await
                    {
                        error!("Error occurred with client \"{client_addr}\": {err:?}");
                    }

                    info!("[{ip}] Disconnected.");
                });
            }
            Err(err) => error!("Error getting client: {err:?}."),
        }
    }
}
