use crate::protocol;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct ServerStatusResponse {
    pub description: String,
    pub players: StatusPlayers,
    pub version: StatusVersion,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<StatusFavicon>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StatusPlayers {
    pub max: i32,
    pub online: i32,
    pub sample: Vec<StatusSample>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StatusSample {
    pub uuid: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StatusVersion {
    pub name: String,
    pub protocol: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StatusFavicon {
    pub icon: String,
}

pub async fn handle_server_status_request(
    mut stream: &mut TcpStream,
    status: Arc<Mutex<ServerStatusResponse>>,
) -> Result<()> {
    // Status request
    let length = protocol::read_varint(&mut stream).await?;
    let mut buffer = vec![0u8; length as usize];
    stream.read_exact(&mut buffer).await?;

    // Status response
    let lock = status.lock().await;

    // Any way of serializing this without cloning it?
    let json = serde_json::to_string(&lock.clone())?;

    let mut packet = vec![0x00, json.len() as u8];
    packet.append(&mut json.as_bytes().to_owned());

    protocol::write_varint(&mut stream, packet.len() as i32).await?;
    stream.write_all(&packet).await?;

    // Ping request
    let length = protocol::read_varint(&mut stream).await?;
    let mut buffer = vec![0u8; length as usize];
    stream.read_exact(&mut buffer).await?;

    // Ping response
    protocol::write_varint(&mut stream, buffer.len() as i32).await?;
    stream.write_all(&buffer).await?;

    Ok(())
}
