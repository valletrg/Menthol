use bytes::{Buf, BufMut};
use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;

pub const CODE: u32 = 22;

#[derive(Debug, Clone)]
pub struct MessageUserRequest {
    pub username: String,
    pub message:  String,
}

impl SlskWrite for MessageUserRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.username.write(buf);
        self.message.write(buf);
    }
}

#[derive(Debug, Clone)]
pub struct MessageUserResponse {
    pub id:       u32,
    pub timestamp: u32,
    pub username: String,
    pub message:  String,
    pub new_msg:  bool,
}

impl SlskRead for MessageUserResponse {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self {
            id:        u32::read(buf)?,
            timestamp: u32::read(buf)?,
            username:  String::read(buf)?,
            message:   String::read(buf)?,
            new_msg:   bool::read(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    use crate::codec::SlskRead;

    #[test]
    fn message_user_request_round_trip() {
        let req = MessageUserRequest {
            username: "alice".into(),
            message:  "hi there".into(),
        };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(String::read(&mut buf).unwrap(), "alice");
        assert_eq!(String::read(&mut buf).unwrap(), "hi there");
    }
}
