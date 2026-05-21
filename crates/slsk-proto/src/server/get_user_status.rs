use bytes::{Buf, BufMut};
use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;

pub const CODE: u32 = 7;

#[derive(Debug, Clone)]
pub struct GetUserStatusRequest {
    pub username: String,
}

impl SlskWrite for GetUserStatusRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.username.write(buf);
    }
}

#[derive(Debug, Clone)]
pub struct GetUserStatusResponse {
    pub username:  String,
    pub status:    u32,
    pub privileged: bool,
}

impl SlskRead for GetUserStatusResponse {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self {
            username:  String::read(buf)?,
            status:    u32::read(buf)?,
            privileged: bool::read(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    use crate::codec::SlskRead;

    #[test]
    fn get_user_status_request_round_trip() {
        let req = GetUserStatusRequest { username: "alice".into() };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(String::read(&mut buf).unwrap(), "alice");
    }

    #[test]
    fn get_user_status_response_decode() {
        // username="alice", status=2 (Online), privileged=true
        let raw: &[u8] = &[
            0x05, 0x00, 0x00, 0x00, 0x61, 0x6c, 0x69, 0x63, 0x65, // "alice"
            0x02, 0x00, 0x00, 0x00, // status=2
            0x01, // privileged=true
        ];
        let mut buf = bytes::Bytes::from_static(raw);
        let resp = GetUserStatusResponse::read(&mut buf).unwrap();
        assert_eq!(resp.username, "alice");
        assert_eq!(resp.status, 2);
        assert!(resp.privileged);
    }
}
