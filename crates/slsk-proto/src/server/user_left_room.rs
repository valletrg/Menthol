use bytes::{Buf, BufMut};
use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;

pub const CODE: u32 = 17;

#[derive(Debug, Clone)]
pub struct UserLeftRoom {
    pub room:     String,
    pub username: String,
}

impl SlskRead for UserLeftRoom {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self {
            room:     String::read(buf)?,
            username: String::read(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_left_room_decode() {
        let raw: &[u8] = &[
            0x04, 0x00, 0x00, 0x00, 0x72, 0x6f, 0x6f, 0x6d, // "room"
            0x05, 0x00, 0x00, 0x00, 0x61, 0x6c, 0x69, 0x63, 0x65, // "alice"
        ];
        let mut buf = bytes::Bytes::from_static(raw);
        let resp = UserLeftRoom::read(&mut buf).unwrap();
        assert_eq!(resp.room, "room");
        assert_eq!(resp.username, "alice");
    }
}
