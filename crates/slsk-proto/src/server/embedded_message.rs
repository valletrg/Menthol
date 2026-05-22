use crate::codec::SlskRead;
use crate::error::ProtoError;
use bytes::Buf;

pub const CODE: u32 = 93;

#[derive(Debug, Clone)]
pub struct EmbeddedMessageResponse {
    pub distributed_code: u8,
    pub distributed_data: bytes::Bytes,
}

impl SlskRead for EmbeddedMessageResponse {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self {
            distributed_code: u8::read(buf)?,
            distributed_data: bytes::Bytes::read(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_message_decode() {
        // code=3, data="hello"
        let raw: &[u8] = &[
            0x03, // code=3
            0x05, 0x00, 0x00, 0x00, // data len=5
            0x68, 0x65, 0x6c, 0x6c, 0x6f, // "hello"
        ];
        let mut buf = bytes::Bytes::from_static(raw);
        let resp = EmbeddedMessageResponse::read(&mut buf).unwrap();
        assert_eq!(resp.distributed_code, 3);
        assert_eq!(&resp.distributed_data[..], b"hello");
    }
}
