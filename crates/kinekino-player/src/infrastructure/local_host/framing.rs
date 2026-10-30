use std::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub(super) async fn read(reader: &mut (impl AsyncRead + Unpin), max: usize) -> io::Result<Vec<u8>> {
    read_observed(reader, max, |_| {}).await
}
pub(super) async fn read_observed(
    reader: &mut (impl AsyncRead + Unpin),
    max: usize,
    mut phase: impl FnMut(&'static str),
) -> io::Result<Vec<u8>> {
    phase("read_length");
    let length = reader.read_u32_le().await? as usize;
    if length == 0 || length > max {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid local frame size",
        ));
    }
    let mut bytes = vec![0; length];
    phase("read_body");
    reader.read_exact(&mut bytes).await?;
    Ok(bytes)
}
pub(super) async fn write(
    writer: &mut (impl AsyncWrite + Unpin),
    bytes: &[u8],
    max: usize,
) -> io::Result<()> {
    if bytes.is_empty() || bytes.len() > max {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid local frame size",
        ));
    }
    writer.write_u32_le(bytes.len() as u32).await?;
    writer.write_all(bytes).await?;
    writer.flush().await
}
