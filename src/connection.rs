use crate::{
    handshake::{HandshakePacket, NextState},
    login::LoginStart,
    protocol,
    proxy::Config,
    status::StatusResponse,
    ONLINE_PLAYERS,
};
use anyhow::Result;
use log::{error, info};
use serde_json::json;
use std::{
    io::ErrorKind,
    net::SocketAddr,
    sync::{atomic::Ordering, Arc},
};
use tokio::{
    io::AsyncWriteExt,
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};

pub struct Client {
    pub read: OwnedReadHalf,
    pub write: OwnedWriteHalf,
    pub address: SocketAddr,
    pub proxy_config: Arc<Config>,
}

impl Client {
    pub async fn start(self) -> Result<()> {
        tokio::spawn(async move {
            let ip = self.address;

            if let Err(err) = self.handle_connection().await {
                error!("Error occurred with client \"{ip}\": {err:?}");
            }

            info!("[{ip}] Disconnected.");
        });

        Ok(())
    }

    pub async fn handle_connection(mut self) -> Result<()> {
        let handshake = HandshakePacket::decode(&mut self.read).await?;

        let address = self.address.to_string();

        if handshake.next_state == NextState::Status {
            info!("[{address}] Handshake with NextState Status received.");

            return StatusResponse::handle_server_status_request(
                &mut self.read,
                &mut self.write,
                self.proxy_config,
            )
            .await;
        }

        // TODO: Handle more packets from the Login flow.

        match TcpStream::connect(&self.proxy_config.server_address).await {
            Ok(server_connection) => {
                info!("[{address}] Connected to server.");

                let (mut server_read, mut server_write) = server_connection.into_split();

                protocol::write_varint(&mut server_write, handshake.buffer.len() as i32).await?;
                server_write.write_all(&handshake.buffer).await?;
                info!("[{address}] Wrote Handshake to server.");

                let mut username = String::new();

                if handshake.next_state == NextState::Login {
                    info!("[{address}] Handshake with NextState Login received.");
                    username =
                        LoginStart::get_client_username(&mut self.read, &mut server_write).await?;
                    info!("[{address}] User {username:?} connected.");
                }

                ONLINE_PLAYERS.fetch_add(1, Ordering::Relaxed);

                let address_clone = address.clone();
                tokio::spawn(async move {
                    // Here we could check the packet and do something with it, like saving chat messages.
                    // For now, just throw everything from the server to the client.
                    if let Err(err) = tokio::io::copy(&mut server_read, &mut self.write).await {
                        if err.kind() != ErrorKind::BrokenPipe {
                            error!(
                                "[{address_clone}] Error copying contents from server to player: {err}"
                            )
                        }
                    }
                });

                if let Err(err) = tokio::io::copy(&mut self.read, &mut server_write).await {
                    if err.kind() != ErrorKind::BrokenPipe {
                        error!("[{address}] Error copying contents from player to server: {err}")
                    }
                }

                if !username.is_empty() {
                    info!("[{address}] User {username:?} disconnecting.");

                    ONLINE_PLAYERS.fetch_sub(1, Ordering::Relaxed);
                }

                Ok(())
            }
            Err(err) => {
                error!("[{address}] Error connecting to server: {err:?}.");

                let json = json!({
                    "text": err.to_string(),
                });

                let mut buffer = vec![0x00];
                let data = &mut json.to_string().as_bytes().to_owned();
                buffer.push(data.len() as u8);
                buffer.append(data);

                protocol::write_varint(&mut self.write, buffer.len() as i32).await?;
                self.write.write_all(&buffer).await?;

                Ok(())
            }
        }
    }
}
