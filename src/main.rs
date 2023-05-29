use crate::proxy::{Config, Proxy};
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use log::{info, LevelFilter};
use std::{
    fs::File,
    io::{BufReader, Read},
    sync::{atomic::AtomicUsize, Arc},
};
use tokio::net::TcpListener;

mod connection;
mod handshake;
mod logger;
mod login;
mod protocol;
mod proxy;
mod status;

pub static ONLINE_PLAYERS: AtomicUsize = AtomicUsize::new(0);

#[tokio::main]
async fn main() -> Result<()> {
    log::set_logger(&logger::LOGGER).unwrap();
    log::set_max_level(LevelFilter::Info);

    info!("Loading configuration file.");
    let file = File::open("./config.json")?;
    let reader = BufReader::new(file);
    let mut config: Config = serde_json::from_reader(reader)?;

    config.server_icon = match config.server_icon {
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

    info!("Creating listener on {:?}.", config.bind);
    let listener = TcpListener::bind(&config.bind).await?;
    info!("Listening on {:?}.", config.bind);

    let proxy = Proxy {
        listener,
        config: Arc::new(config),
    };

    proxy.listen().await?;

    Ok(())
}
