use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;
use bytes::{Buf, BufMut};

pub const CODE: u32 = 134;

// AddRoomMember is bidirectional
#[derive(Debug, Clone)]
pub struct AddRoomMemberRequest {
    pub room: String,
    pub username: String,
}

impl SlskWrite for AddRoomMemberRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.room.write(buf);
        self.username.write(buf);
    }
}

#[derive(Debug, Clone)]
pub struct AddRoomMemberResponse {
    pub room: String,
    pub username: String,
}

impl SlskRead for AddRoomMemberResponse {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self {
            room: String::read(buf)?,
            username: String::read(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_room_member_request_round_trip() {
        let req = AddRoomMemberRequest {
            room: "myroom".into(),
            username: "alice".into(),
        };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(String::read(&mut buf).unwrap(), "myroom");
        assert_eq!(String::read(&mut buf).unwrap(), "alice");
    }

    use bytes::BytesMut;
}
