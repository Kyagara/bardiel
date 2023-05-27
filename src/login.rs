use crate::protocol;
use anyhow::{anyhow, Result};
use std::io::Cursor;
use tokio::{
    io::AsyncWriteExt,
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
};

pub struct LoginStart;

impl LoginStart {
    pub async fn get_client_username(
        mut client_read: &mut OwnedReadHalf,
        server_write: &mut OwnedWriteHalf,
    ) -> Result<String> {
        let buffer = protocol::decode_packet(&mut client_read).await?;

        if buffer[0] != 0x00 {
            return Err(anyhow!("Invalid packet ID."));
        }

        protocol::write_varint(server_write, buffer.len() as i32).await?;
        server_write.write_all(&buffer).await?;

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
