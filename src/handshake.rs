use crate::protocol;
use anyhow::{anyhow, Result};
use std::io::Cursor;
use tokio::net::tcp::OwnedReadHalf;

pub struct HandshakePacket {
    pub buffer: Vec<u8>,
    pub next_state: NextState,
}

#[derive(PartialEq)]
pub enum NextState {
    Status,
    Login,
}

impl HandshakePacket {
    pub async fn decode(read: &mut OwnedReadHalf) -> Result<HandshakePacket> {
        let hs = protocol::decode_packet(read).await?;

        let mut cursor = Cursor::new(hs);

        let id = protocol::read_varint(&mut cursor).await?;

        if id != 0x00 {
            return Err(anyhow!("Invalid packet ID."));
        }

        // Skipping some fields that we don't use for now.
        cursor.set_position(cursor.position() + 14);

        let next_state = if protocol::read_varint(&mut cursor).await? == 1 {
            NextState::Status
        } else {
            NextState::Login
        };

        Ok(HandshakePacket {
            buffer: cursor.into_inner(),
            next_state,
        })
    }
}
