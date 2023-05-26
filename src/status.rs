use crate::protocol;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{io::Cursor, sync::Arc};
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
    pub favicon: Option<String>,
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

impl ServerStatusResponse {
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

        // Buffer with total length of the string
        let mut total_length = [0u8; 2];
        // Need to pass a cursor to write_varint
        let mut cursor = Cursor::new(&mut total_length[..]);

        // We need to find how big the json length is before writing the packet since the json length can be bigger than one byte.
        // Total length for this packet is: packet id byte + json length (can be bigger than one byte) + json bytes
        let length =
            1 + json.len() + protocol::write_varint(&mut cursor, json.len() as i32).await?;

        protocol::write_varint(&mut stream, length as i32).await?;
        protocol::write_varint(&mut stream, 0x00).await?;
        protocol::write_string(&mut stream, json).await?;

        // Ping request
        let length = protocol::read_varint(&mut stream).await?;
        let mut buffer = vec![0u8; length as usize];
        stream.read_exact(&mut buffer).await?;

        // Ping response
        protocol::write_varint(&mut stream, buffer.len() as i32).await?;
        stream.write_all(&buffer).await?;

        Ok(())
    }
}
