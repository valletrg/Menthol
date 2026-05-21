use bytes::{Buf, BufMut, Bytes, BytesMut};
use slsk_proto::error::ProtoError;

/// Reads a server message frame: [u32 length][u32 code][payload]
/// Returns the code and payload bytes (zero-copy slice into receive buffer)
pub fn read_frame(buf: &mut impl Buf) -> Result<Option<(u32, Bytes)>, ProtoError> {
    if buf.remaining() < 4 {
        return Ok(None);
    }
    // Peek at length
    let len = buf.get_u32_le() as usize;
    if buf.remaining() < len {
        // Put back the length we peeked
        return Ok(None);
    }
    let code = buf.get_u32_le();
    let payload = buf.copy_to_bytes(len - 4);
    Ok(Some((code, payload)))
}

/// Writes a server message frame
pub fn write_frame(code: u32, payload: &[u8]) -> Bytes {
    let total_len = 4 + payload.len(); // code + payload
    let mut buf = BytesMut::with_capacity(4 + total_len);
    buf.put_u32_le(total_len as u32);
    buf.put_u32_le(code);
    buf.put_slice(payload);
    buf.freeze()
}

/// Encode a message using SlskWrite and frame it
pub fn encode_message<T: slsk_proto::codec::SlskWrite>(code: u32, msg: &T) -> Bytes {
    let mut payload = BytesMut::new();
    msg.write(&mut payload);
    write_frame(code, &payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_frame_verify_length() {
        // empty payload with code=1: 4 bytes length + 4 bytes code + 0 = 8
        let frame = write_frame(1, &[]);
        assert_eq!(frame.len(), 8);
        let mut buf = frame.clone();
        assert_eq!(buf.get_u32_le(), 4); // total_len = 4
        assert_eq!(buf.get_u32_le(), 1); // code
    }

    #[test]
    fn read_frame_simple() {
        let frame = write_frame(42, &[1, 2, 3]);
        let mut buf = frame;
        let result = read_frame(&mut buf).unwrap();
        let (code, payload) = result.unwrap();
        assert_eq!(code, 42);
        assert_eq!(payload.len(), 3);
        assert_eq!(&payload[..], &[1, 2, 3]);
    }

    #[test]
    fn read_frame_incomplete() {
        // Only 2 bytes available, not enough for full frame
        let raw: &[u8] = &[0x04, 0x00, 0x00, 0x00]; // length=4 but no code/payload
        let mut buf = bytes::Bytes::from_static(raw);
        let result = read_frame(&mut buf).unwrap();
        assert!(result.is_none());
    }
}
