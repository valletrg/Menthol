use bytes::{Buf, BufMut};
use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;

pub const CODE: u32 = 152;

#[derive(Debug, Clone)]
pub struct GlobalRoomMessageResponse {
    pub room:     String,
    pub username: String,
    pub message:  String,
}

impl SlskRead for GlobalRoomMessageResponse {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self {
            room:     String::read(buf)?,
            username: String::read(buf)?,
            message:  String::read(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_room_message_decode() {
        let raw: &[u8] = &[
            0x05, 0x00, 0x00, 0x00, 0x72, 0x6f, 0x6f, 0x6d, 0x73, // "rooms"
            0x05, 0x00, 0x00, 0x00, 0x61, 0x6c, 0x69, 0x63, 0x65, // "alice"
            0x04, 0x00, 0x00, 0x00, 0x68, 0x69, 0x21, 0x21, // "hi!!"
        ];
        let mut buf = bytes::Bytes::from_static(raw);
        let resp = GlobalRoomMessageResponse::read(&mut buf).unwrap();
        assert_eq!(resp.room, "rooms");
        assert_eq!(resp.username, "alice");
        assert_eq!(resp.message, "hi!!");
    }
}
