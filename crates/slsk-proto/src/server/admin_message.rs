use bytes::{Buf, BufMut};
use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;

pub const CODE: u32 = 66;

// AdminMessage is receive-only
#[derive(Debug, Clone)]
pub struct AdminMessageResponse {
    pub message: String,
}

impl SlskRead for AdminMessageResponse {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self { message: String::read(buf)? })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_message_decode() {
        let raw: &[u8] = &[
            0x0d, 0x00, 0x00, 0x00, // len=13
            0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x55, 0x73, 0x65, 0x72, 0x21, 0x21, 0x21, // "Hello User!!!"
        ];
        let mut buf = bytes::Bytes::from_static(raw);
        let resp = AdminMessageResponse::read(&mut buf).unwrap();
        assert_eq!(resp.message, "Hello User!!!");
    }
}
