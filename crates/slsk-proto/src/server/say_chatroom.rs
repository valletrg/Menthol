use bytes::{Buf, BufMut};
use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;

pub const CODE: u32 = 13;

#[derive(Debug, Clone)]
pub struct SayChatroomRequest {
    pub room:    String,
    pub message: String,
}

impl SlskWrite for SayChatroomRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.room.write(buf);
        self.message.write(buf);
    }
}

#[derive(Debug, Clone)]
pub struct SayChatroomResponse {
    pub room:    String,
    pub username: String,
    pub message: String,
}

impl SlskRead for SayChatroomResponse {
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
    use bytes::BytesMut;
    use crate::codec::SlskRead;

    #[test]
    fn say_chatroom_request_round_trip() {
        let req = SayChatroomRequest {
            room: "The Lobby".into(),
            message: "hello everyone".into(),
        };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(String::read(&mut buf).unwrap(), "The Lobby");
        assert_eq!(String::read(&mut buf).unwrap(), "hello everyone");
    }
}
