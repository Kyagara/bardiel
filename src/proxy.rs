use crate::{
    handshake::{HandshakePacket, NextState},
    protocol,
    status::{self, ServerStatusResponse},
};
use anyhow::{anyhow, Result};
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    io::{Cursor, ErrorKind},
    net::IpAddr,
    sync::Arc,
};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProxyConfig {
    pub bind: String,
    pub server_address: String,
    pub max_players: i32,
    pub description: String,
    pub server_icon: Option<String>,
}

pub async fn handle_connection(
    mut client_conn: TcpStream,
    client_addr: IpAddr,
    proxy_config: ProxyConfig,
    server_status: Arc<Mutex<ServerStatusResponse>>,
) -> Result<()> {
    let handshake = HandshakePacket::new(&mut client_conn).await?;

    if handshake.next_state == NextState::Status {
        info!("[{client_addr}] Handshake with NextState Status received.");
        status::handle_server_status_request(&mut client_conn, server_status).await?;
        return Ok(());
    }

    // TODO: Handle more packets from the Login flow.

    match TcpStream::connect(proxy_config.server_address).await {
        Ok(mut server_conn) => {
            info!("[{client_addr}] Connected to server.");

            protocol::write_varint(&mut server_conn, handshake.buffer.len() as i32).await?;
            server_conn.write_all(&handshake.buffer).await?;
            info!("[{client_addr}] Wrote Handshake to server.");

            let mut username = String::new();

            if handshake.next_state == NextState::Login {
                info!("[{client_addr}] Handshake with NextState Login received.");
                username = get_client_username(&mut client_conn, &mut server_conn).await?;
                info!("[{client_addr}] User {username:?} connected.");
            }

            let (mut server_read, mut server_write) = io::split(server_conn);
            let (mut player_read, mut player_write) = io::split(client_conn);

            tokio::spawn(async move {
                // Here we could check the packet and do something with it, like saving chat messages.
                // For now, just throw everything from the server to the client.
                if let Err(err) = io::copy(&mut server_read, &mut player_write).await {
                    if err.kind() != ErrorKind::BrokenPipe {
                        error!(
                            "[{client_addr}] Error copying contents from server to player: {err}"
                        )
                    }
                }
            });

            if let Err(err) = io::copy(&mut player_read, &mut server_write).await {
                if err.kind() != ErrorKind::BrokenPipe {
                    error!("[{client_addr}] Error copying contents from player to server: {err}")
                }
            }

            if !username.is_empty() {
                info!("[{client_addr}] User {username:?} disconnecting.");
            }

            Ok(())
        }
        Err(err) => {
            error!("[{client_addr}] Error connecting to server: {err:?}.");

            // Really generic message, return the actual error or be intentionally vague?
            let json = json!({
                "text": "Server offline.",
            });

            let mut buffer = vec![0x00];
            let data = &mut json.to_string().as_bytes().to_owned();
            buffer.push(data.len() as u8);
            buffer.append(data);

            protocol::write_varint(&mut client_conn, buffer.len() as i32).await?;
            client_conn.write_all(&buffer).await?;

            Ok(())
        }
    }
}

async fn get_client_username(
    player_conn: &mut TcpStream,
    server_conn: &mut TcpStream,
) -> Result<String> {
    let length = protocol::read_varint(player_conn).await?;
    let mut buffer = vec![0u8; length as usize];
    player_conn.read_exact(&mut buffer).await?;

    protocol::write_varint(server_conn, buffer.len() as i32).await?;
    server_conn.write_all(&buffer).await?;

    let mut cursor = Cursor::new(buffer);
    let id = protocol::read_varint(&mut cursor).await?;

    if id != 0x00 {
        return Err(anyhow!("Invalid packet ID."));
    }

    let name = protocol::read_string(&mut cursor).await?;

    Ok(name)
}