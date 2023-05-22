use anyhow::Result;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub async fn read_varint<T>(stream: &mut T) -> Result<i32>
where
    T: AsyncRead + std::marker::Unpin,
{
    let mut buffer = [0];
    let mut result = 0;

    for i in 0..4 {
        stream.read_exact(&mut buffer).await?;

        result |= ((buffer[0] & 0x7F) as i32) << (7 * i);

        if buffer[0] & 0x80 == 0 {
            break;
        }
    }

    Ok(result)
}

pub async fn write_varint<T>(stream: &mut T, mut value: i32) -> Result<()>
where
    T: AsyncWrite + std::marker::Unpin,
{
    let mut buffer = [0];

    while value != 0 {
        buffer[0] = (value & 0x7F) as u8;

        value = (value >> 7) & (i32::MAX >> 6);

        if value != 0 {
            buffer[0] |= 0x80;
        }

        stream.write_all(&buffer).await?;
    }

    Ok(())
}

pub async fn read_string<T>(stream: &mut T) -> Result<String>
where
    T: AsyncRead + std::marker::Unpin,
{
    let length = read_varint(stream).await?;
    let mut buffer = vec![0u8; length as usize];

    stream.read_exact(&mut buffer).await?;

    Ok(String::from_utf8_lossy(&buffer).to_string())
}