use crate::error::{Error, Result};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};

const PAYLOAD_SIZE_INFO_LENGTH: usize = 8;

/// Converts the bytes from our stream into a string(The payload).
/// The first byte should be the hexadecimal representation
/// of the size of the message.
pub async fn get_payload_content<T>(stream: &mut T) -> Result<String>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let mut payload_size_buffer = Vec::with_capacity(PAYLOAD_SIZE_INFO_LENGTH);
    if let Err(e) = stream.read_buf(&mut payload_size_buffer).await {
        return Err(Error::ErrorReadingMessage(format!("{} : {}", e, e.kind())));
    }
    let buf_size = match String::from_utf8(payload_size_buffer) {
        Ok(v) => match usize::from_str_radix(&v, 16) {
            Ok(n) => n,
            Err(_) => {
                return Err(Error::InvalidPayload(format!(
                    "First {PAYLOAD_SIZE_INFO_LENGTH} bytes weren't a correct integer"
                )));
            }
        },
        Err(_) => {
            return Err(Error::InvalidPayload(format!(
                "First {PAYLOAD_SIZE_INFO_LENGTH} bytes weren't a valid UTF-8 string",
            )));
        }
    };
    let mut payload_buf: Vec<u8> = Vec::with_capacity(buf_size);
    if let Err(e) = stream.read_buf(&mut payload_buf).await {
        return Err(Error::ErrorReadingMessage(format!("{} : {}", e, e.kind())));
    }
    match String::from_utf8(payload_buf) {
        Ok(s) => Ok(s),
        Err(_) => Err(Error::InvalidPayload(
            "Message not a valid UTF-8 string".to_string(),
        )),
    }
}

/// Primarily converts our message into bytes, the first
/// byte is the hexadecimal representation of the size
/// of the message followed by the message(in bytes).
pub fn payload_to_bytes(message: &str) -> Result<Vec<u8>> {
    let msg_len = format!("{:x}", message.len());
    let mut payload = "00000000"[0..PAYLOAD_SIZE_INFO_LENGTH - msg_len.len()].to_owned();
    payload.push_str(&msg_len);
    payload.push_str(message);
    Ok(Vec::from(payload.as_bytes()))
}
#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use tokio::io::BufStream;

    use super::*;

    #[test]
    fn test_payload_to_bytes() {
        let payload = r#"{"action": "hello"}"#.to_string();
        let payload_size = payload.len();
        let bytes = payload_to_bytes(&payload).unwrap();
        let size_info_bytes = &bytes[0..PAYLOAD_SIZE_INFO_LENGTH];
        let size = String::from_utf8(size_info_bytes.to_vec())
            .unwrap()
            .parse::<u16>()
            .unwrap();
        assert_eq!(size, payload_size as u16);
        let content_bytes = &bytes[PAYLOAD_SIZE_INFO_LENGTH..bytes.len()];
        let content = String::from_utf8(content_bytes.to_vec()).unwrap();
        assert_eq!(content.len(), payload_size);
        assert!(content.eq(&payload));
    }

    #[tokio::test]
    async fn test_get_payload_content_no_size_info() {
        let payload_bytes = r#"{"action": "hello"}"#.as_bytes().to_vec();
        let cursor = Cursor::new(payload_bytes);
        let mut bytes_stream = BufStream::new(cursor);
        let res = get_payload_content(&mut bytes_stream).await;
        assert!(res.is_err())
    }

    #[tokio::test]
    async fn test_get_payload_content_valid_payload_bytes() {
        let payload_bytes = r#"0019{"action": "hello"}"#.as_bytes().to_vec();
        let cursor = Cursor::new(payload_bytes);
        let mut bytes_stream = BufStream::new(cursor);
        let res = get_payload_content(&mut bytes_stream).await.unwrap();
        assert!(res.eq(r#"{"action": "hello"}"#));
    }
}
