use crate::{
    handshake::{HandshakePacket, NextState},
    login::LoginStart,
    protocol,
    status::StatusResponse,
    ONLINE_PLAYERS,
};
use anyhow::Result;
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    io::ErrorKind,
    net::IpAddr,
    sync::{atomic::Ordering, Arc},
};
use tokio::{io::AsyncWriteExt, net::TcpStream};

pub struct Proxy {
    pub config: Config,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub bind: String,
    pub server_address: String,
    pub max_players: i32,
    pub description: String,
    pub server_icon: Option<String>,
}

impl Proxy {
    pub async fn handle_connection(
        client_connection: TcpStream,
        address: IpAddr,
        proxy: Arc<Proxy>,
    ) -> Result<()> {
        let (mut client_read, mut client_write) = client_connection.into_split();

        let handshake = HandshakePacket::decode(&mut client_read).await?;

        if handshake.next_state == NextState::Status {
            info!("[{address}] Handshake with NextState Status received.");

            return StatusResponse::handle_server_status_request(
                &mut client_read,
                &mut client_write,
                proxy,
            )
            .await;
        }

        // TODO: Handle more packets from the Login flow.

        match TcpStream::connect(&proxy.config.server_address).await {
            Ok(server_connection) => {
                info!("[{address}] Connected to server.");

                let (mut server_read, mut server_write) = server_connection.into_split();

                protocol::write_varint(&mut server_write, handshake.buffer.len() as i32).await?;
                server_write.write_all(&handshake.buffer).await?;
                info!("[{address}] Wrote Handshake to server.");

                let mut username = String::new();

                if handshake.next_state == NextState::Login {
                    info!("[{address}] Handshake with NextState Login received.");
                    username = LoginStart::get_client_username(&mut client_read, &mut server_write)
                        .await?;
                    info!("[{address}] User {username:?} connected.");
                }

                ONLINE_PLAYERS.fetch_add(1, Ordering::Relaxed);

                tokio::spawn(async move {
                    // Here we could check the packet and do something with it, like saving chat messages.
                    // For now, just throw everything from the server to the client.
                    if let Err(err) = tokio::io::copy(&mut server_read, &mut client_write).await {
                        if err.kind() != ErrorKind::BrokenPipe {
                            error!(
                                "[{address}] Error copying contents from server to player: {err}"
                            )
                        }
                    }
                });

                if let Err(err) = tokio::io::copy(&mut client_read, &mut server_write).await {
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

                protocol::write_varint(&mut client_write, buffer.len() as i32).await?;
                client_write.write_all(&buffer).await?;

                Ok(())
            }
        }
    }
}
