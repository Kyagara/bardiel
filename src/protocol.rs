use anyhow::Result;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub async fn read_varint<T: AsyncRead + AsyncWrite + Unpin>(stream: &mut T) -> Result<i32> {
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

pub async fn write_varint<T: AsyncWrite + Unpin>(stream: &mut T, mut value: i32) -> Result<usize> {
    let mut buffer = [0];
    let mut written = 0;

    loop {
        buffer[0] = (value & 0x7F) as u8;

        value = (value >> 7) & (i32::MAX >> 6);

        if value != 0 {
            buffer[0] |= 0x80;
        }

        written += stream.write(&buffer).await?;

        if value == 0 {
            break Ok(written);
        }
    }
}

pub async fn read_string<T: AsyncRead + AsyncWrite + Unpin>(stream: &mut T) -> Result<String> {
    let length = read_varint(stream).await?;
    let mut buffer = vec![0u8; length as usize];
    stream.read_exact(&mut buffer).await?;
    Ok(String::from_utf8(buffer)?)
}

pub async fn write_string<T: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut T,
    value: String,
) -> Result<()> {
    write_varint(stream, value.len() as i32).await?;
    stream.write_all(value.as_bytes()).await?;
    Ok(())
}
