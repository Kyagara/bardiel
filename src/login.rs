use crate::protocol;
use anyhow::{anyhow, Result};
use std::io::Cursor;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub async fn get_client_username(
    player_conn: &mut TcpStream,
    server_conn: &mut TcpStream,
) -> Result<String> {
    let length = protocol::read_varint(player_conn).await?;
    let mut buffer = vec![0u8; length as usize];
    player_conn.read_exact(&mut buffer).await?;

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

    Ok(name)
}
