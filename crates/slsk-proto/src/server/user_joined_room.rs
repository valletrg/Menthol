use bytes::{Buf, BufMut};
use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;

pub const CODE: u32 = 16;

#[derive(Debug, Clone)]
pub struct UserJoinedRoom {
    pub room:      String,
    pub username:  String,
    pub status:    u32,
    pub avgspeed:  u32,
    pub uploadnum:  u32,
    pub unknown:   u32,
    pub files:     u32,
    pub dirs:      u32,
    pub slotsfull: u32,
    pub country:   String,
}

impl SlskRead for UserJoinedRoom {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self {
            room:      String::read(buf)?,
            username:  String::read(buf)?,
            status:    u32::read(buf)?,
            avgspeed:  u32::read(buf)?,
            uploadnum: u32::read(buf)?,
            unknown:   u32::read(buf)?,
            files:     u32::read(buf)?,
            dirs:      u32::read(buf)?,
            slotsfull: u32::read(buf)?,
            country:   String::read(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_joined_room_decode() {
        // username="alice", status=2, avgspeed=100, others=0, country="US"
        let raw: &[u8] = &[
            0x04, 0x00, 0x00, 0x00, 0x72, 0x6f, 0x6f, 0x6d, // room="room"
            0x05, 0x00, 0x00, 0x00, 0x61, 0x6c, 0x69, 0x63, 0x65, // "alice"
            0x02, 0x00, 0x00, 0x00, // status=2
            0x64, 0x00, 0x00, 0x00, // avgspeed=100
            0x00, 0x00, 0x00, 0x00, // uploadnum=0
            0x00, 0x00, 0x00, 0x00, // unknown=0
            0x00, 0x00, 0x00, 0x00, // files=0
            0x00, 0x00, 0x00, 0x00, // dirs=0
            0x00, 0x00, 0x00, 0x00, // slotsfull=0
            0x02, 0x00, 0x00, 0x00, 0x55, 0x53, // country="US"
        ];
        let mut buf = bytes::Bytes::from_static(raw);
        let resp = UserJoinedRoom::read(&mut buf).unwrap();
        assert_eq!(resp.username, "alice");
        assert_eq!(resp.country, "US");
    }
}
