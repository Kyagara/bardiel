use crate::protocol;
use anyhow::{anyhow, Result};
use std::io::Cursor;
use tokio::{io::AsyncWriteExt, net::TcpStream};

pub struct LoginStart;

impl LoginStart {
    pub async fn get_client_username(
        player_conn: &mut TcpStream,
        server_conn: &mut TcpStream,
    ) -> Result<String> {
        let buffer = protocol::decode_packet(player_conn).await?;

        if buffer[0] != 0x00 {
            return Err(anyhow!("Invalid packet ID."));
        }

        protocol::write_varint(server_conn, buffer.len() as i32).await?;
        server_conn.write_all(&buffer).await?;

        let mut cursor = Cursor::new(buffer);
        let id = protocol::read_varint(&mut cursor).await?;

        if id != 0x00 {
            return Err(anyhow!("Invalid packet ID."));
        }

        let name = protocol::read_string(&mut cursor).await?;

        if name.is_empty() {
            return Err(anyhow!("Username empty."));
        }

        if name.len() > 16 {
            return Err(anyhow!("Username too long."));
        }

        Ok(name)
    }
}
