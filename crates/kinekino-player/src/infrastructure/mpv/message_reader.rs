//! Cancel-safe bounded JSON framing for owned mpv IPC.
use std::io;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
const MAX_MESSAGE_BYTES: usize = 128 * 1024;

pub(super) struct MessageReader<R> {
    reader: BufReader<R>,
    pending: Vec<u8>,
}
impl<R: AsyncRead + Unpin> MessageReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
            pending: Vec::new(),
        }
    }
    /// Any error is terminal for this connection. Cancellation preserves partial bytes.
    pub async fn next(&mut self) -> io::Result<Option<serde_json::Value>> {
        loop {
            let available = self.reader.fill_buf().await?;
            if available.is_empty() {
                return if self.pending.is_empty() {
                    Ok(None)
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "Truncated native IPC frame",
                    ))
                };
            }
            let count = available
                .iter()
                .position(|byte| *byte == b'\n')
                .map_or(available.len(), |index| index + 1);
            if count > MAX_MESSAGE_BYTES.saturating_sub(self.pending.len()) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Native IPC frame exceeds byte budget",
                ));
            }
            let complete = available[count - 1] == b'\n';
            self.pending.extend_from_slice(&available[..count]);
            self.reader.consume(count);
            if complete {
                let message = serde_json::from_slice(&self.pending).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "Invalid native IPC JSON frame")
                })?;
                self.pending.clear();
                return Ok(Some(message));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;
    #[test]
    fn cancelled_wait_keeps_consumed_prefix_and_later_message_boundaries() {
        tokio::runtime::Builder::new_current_thread().build().unwrap().block_on(async {
            let (mut writer, reader) = tokio::io::duplex(1024);
            let mut reader = MessageReader::new(reader);
            writer.write_all(b"{").await.unwrap();
            tokio::select! {
                biased;
                result = reader.next() => panic!("Partial frame unexpectedly completed: {result:?}"),
                _ = std::future::ready(()) => {},
            }
            assert_eq!(reader.pending, b"{");
            writer.write_all(b"\"one\":1}\n{\"two\":2}\n").await.unwrap();
            assert_eq!(reader.next().await.unwrap().unwrap()["one"], 1);
            assert_eq!(reader.next().await.unwrap().unwrap()["two"], 2);
            drop(writer);
            assert!(reader.next().await.unwrap().is_none());
        });
    }
    #[test]
    fn over_limit_and_truncated_json_never_become_messages() {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async {
                let (mut writer, reader) = tokio::io::duplex(MAX_MESSAGE_BYTES + 2);
                writer
                    .write_all(&vec![b' '; MAX_MESSAGE_BYTES + 1])
                    .await
                    .unwrap();
                writer.write_all(b"\n").await.unwrap();
                assert_eq!(
                    MessageReader::new(reader).next().await.unwrap_err().kind(),
                    io::ErrorKind::InvalidData
                );
                let (mut writer, reader) = tokio::io::duplex(128);
                writer
                    .write_all(b"{\"valid_json_but_unterminated_frame\":true}")
                    .await
                    .unwrap();
                drop(writer);
                assert_eq!(
                    MessageReader::new(reader).next().await.unwrap_err().kind(),
                    io::ErrorKind::UnexpectedEof
                );
            });
    }
}
