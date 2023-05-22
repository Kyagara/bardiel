use crate::protocol;
use anyhow::{anyhow, Result};
use std::io::Cursor;
use tokio::{io::AsyncReadExt, net::TcpStream};

#[allow(unused)]
pub struct HandshakePacket {
    pub buffer: Vec<u8>,
    protocol_version: i32,
    server_address: String,
    server_port: u16,
    pub next_state: NextState,
}

#[derive(PartialEq)]
pub enum NextState {
    Status,
    Login,
}

impl HandshakePacket {
    pub async fn new(stream: &mut TcpStream) -> Result<HandshakePacket> {
        let length = protocol::read_varint(stream).await?;
        let mut buffer = vec![0u8; length as usize];
        stream.read_exact(&mut buffer).await?;

        let mut cursor = Cursor::new(buffer);

        let id = protocol::read_varint(&mut cursor).await?;

        if id != 0x00 {
            return Err(anyhow!("Invalid packet ID."));
        }

        let protocol_version = protocol::read_varint(&mut cursor).await?;
        let server_address = protocol::read_string(&mut cursor).await?;
        let server_port = cursor.read_u16().await?;

        let next_state = if protocol::read_varint(&mut cursor).await? == 1 {
            NextState::Status
        } else {
            NextState::Login
        };

        Ok(HandshakePacket {
            buffer: cursor.into_inner(),
            protocol_version,
            server_address,
            server_port,
            next_state,
        })
    }
}
