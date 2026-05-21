use bytes::{Buf, BufMut};
use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;

pub const CODE: u32 = 133;

#[derive(Debug, Clone)]
pub struct RoomMembersResponse {
    pub room:         String,
    pub num_members:  u32,
    pub members:      Vec<String>,
}

impl SlskRead for RoomMembersResponse {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        let room = String::read(buf)?;
        let num_members = u32::read(buf)?;
        let mut members = Vec::new();
        for _ in 0..num_members {
            members.push(String::read(buf)?);
        }
        Ok(Self { room, num_members, members })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn room_members_decode() {
        let raw: &[u8] = &[
            0x05, 0x00, 0x00, 0x00, 0x72, 0x6f, 0x6f, 0x6d, 0x73, // "rooms"
            0x02, 0x00, 0x00, 0x00, // num_members=2
            0x05, 0x00, 0x00, 0x00, 0x61, 0x6c, 0x69, 0x63, 0x65, // "alice"
            0x03, 0x00, 0x00, 0x00, 0x62, 0x6f, 0x62, // "bob"
        ];
        let mut buf = bytes::Bytes::from_static(raw);
        let resp = RoomMembersResponse::read(&mut buf).unwrap();
        assert_eq!(resp.room, "rooms");
        assert_eq!(resp.num_members, 2);
    }
}
