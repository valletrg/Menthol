use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;
use bytes::{Buf, BufMut};

pub const CODE: u32 = 1001;

#[derive(Debug, Clone)]
pub struct CantConnectToPeerRequest {
    pub token: u32,
    pub username: String,
}

impl SlskWrite for CantConnectToPeerRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.token.write(buf);
        self.username.write(buf);
    }
}

#[derive(Debug, Clone)]
pub struct CantConnectToPeerResponse {
    pub token: u32,
}

impl SlskRead for CantConnectToPeerResponse {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self {
            token: u32::read(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::SlskRead;
    use bytes::BytesMut;

    #[test]
    fn cant_connect_to_peer_request_round_trip() {
        let req = CantConnectToPeerRequest {
            token: 999,
            username: "alice".into(),
        };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(u32::read(&mut buf).unwrap(), 999);
        assert_eq!(String::read(&mut buf).unwrap(), "alice");
    }

    #[test]
    fn cant_connect_to_peer_response_decode() {
        // 4999 = 0x1387, little-endian: 0x87 0x13 0x00 0x00
        let raw: &[u8] = &[0x87, 0x13, 0x00, 0x00];
        let mut buf = bytes::Bytes::from_static(raw);
        let resp = CantConnectToPeerResponse::read(&mut buf).unwrap();
        assert_eq!(resp.token, 4999);
    }
}
