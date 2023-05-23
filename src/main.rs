use crate::handshake::{HandshakePacket, NextState};
use anyhow::{anyhow, Result};
use log::{error, info, LevelFilter};
use std::io::{Cursor, ErrorKind};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

mod handshake;
mod logger;
mod protocol;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    log::set_logger(&logger::LOGGER).unwrap();
    log::set_max_level(LevelFilter::Info);

    let proxy_addr = "127.0.0.1:25565";

    info!("Creating listener on {proxy_addr:?}.");
    let listener = TcpListener::bind(proxy_addr).await?;
    info!("Listening to {proxy_addr:?}.");

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                info!("New client: \"{addr}\".");

                tokio::spawn(async move {
                    let ip = addr.ip().to_string();

                    if let Err(err) = handle_connection(stream, ip.clone()).await {
                        error!("Error occurred with client \"{addr}\": {err:?}.");
                    }

                    info!("[{ip}] Disconnected.");
                });
            }
            Err(err) => error!("Error getting client: {err:?}."),
        }
    }
}

async fn handle_connection(mut client_conn: TcpStream, client_addr: String) -> Result<()> {
    let handshake = HandshakePacket::new(&mut client_conn).await?;

    // The minecraft server.
    let server_addr = "127.0.0.1:25566";

    match TcpStream::connect(server_addr).await {
        Ok(mut server_conn) => {
            info!("[{client_addr}] Estabilished new connection to server.");

            if handshake.next_state == NextState::Status {
                info!("[{client_addr}] Handshake with NextState Status received.")
            }

            protocol::write_varint(&mut server_conn, handshake.buffer.len() as i32).await?;
            server_conn.write_all(&handshake.buffer).await?;
            info!("[{client_addr}] Wrote Handshake to server.");

            let mut username = String::new();

            if handshake.next_state == NextState::Login {
                info!("[{client_addr}] Handshake with NextState Login received.");
                username = get_username(&mut client_conn, &mut server_conn).await?;
                info!("[{client_addr}] User {username:?} connected.");
            }

            let (mut server_read, mut server_write) = io::split(server_conn);
            let (mut player_read, mut player_write) = io::split(client_conn);

            let addr_clone = client_addr.clone();
            tokio::spawn(async move {
                // Here we could check the packet and do something with it, like saving chat messages.
                // For now, just throw everything from the server to the client.
                if let Err(err) = io::copy(&mut server_read, &mut player_write).await {
                    if err.kind() != ErrorKind::BrokenPipe {
                        error!("[{addr_clone}] Error copying contents from server to player: {err}")
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
        Err(err) => Err(anyhow!(
            "[{client_addr}] Error connecting to server: {err:?}."
        )),
    }
}

async fn get_username(player_conn: &mut TcpStream, server_conn: &mut TcpStream) -> Result<String> {
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
