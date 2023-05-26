use crate::{protocol, proxy::Proxy, ONLINE_PLAYERS};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    io::Cursor,
    sync::{atomic::Ordering, Arc},
};
use tokio::{io::AsyncWriteExt, net::TcpStream};

#[derive(Serialize, Deserialize)]
pub struct StatusResponse;

impl StatusResponse {
    pub async fn handle_server_status_request(
        mut stream: &mut TcpStream,
        proxy: Arc<Proxy>,
    ) -> Result<()> {
        // Status request
        let request = protocol::decode_packet(stream).await?;

        if request[0] != 0x00 {
            return Err(anyhow!("Invalid packet ID."));
        }

        // Status response

        let json = json!({
            "version": {
                "name": "bardiel",
                "protocol": 762
              },
              "description": proxy.config.description,
              "players": {
                "max": proxy.config.max_players,
                "online": ONLINE_PLAYERS.load(Ordering::Relaxed),
                "sample": []
              },
              "favicon":  proxy.config.server_icon
        });

        let json = serde_json::to_string(&json)?;

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
        let ping = protocol::decode_packet(stream).await?;

        if ping[0] != 0x01 {
            return Err(anyhow!("Invalid packet ID."));
        }

        // Ping response
        protocol::write_varint(&mut stream, ping.len() as i32).await?;
        stream.write_all(&ping).await?;

        Ok(())
    }
}
