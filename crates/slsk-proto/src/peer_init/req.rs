use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;
use bytes::{Buf, BufMut};

pub const CODE: u8 = 1;

// PeerInit is the first message sent on a direct P/F/D connection
// The version MUST match what we sent in Login
#[derive(Debug, Clone)]
pub struct PeerInitRequest {
    pub username: String,
    pub conn_type: String,
    pub version: u32,
}

impl SlskWrite for PeerInitRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.username.write(buf);
        self.conn_type.write(buf);
        self.version.write(buf);
    }
}

impl SlskRead for PeerInitRequest {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self {
            username: String::read(buf)?,
            conn_type: String::read(buf)?,
            version: u32::read(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::SlskRead;
    use bytes::BytesMut;

    #[test]
    fn peer_init_request_round_trip() {
        let req = PeerInitRequest {
            username: "alice".into(),
            conn_type: "P".into(),
            version: 160,
        };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(String::read(&mut buf).unwrap(), "alice");
        assert_eq!(String::read(&mut buf).unwrap(), "P");
        assert_eq!(u32::read(&mut buf).unwrap(), 160);
    }

    #[test]
    fn peer_init_request_decode() {
        let raw: &[u8] = &[
            0x05, 0x00, 0x00, 0x00, 0x61, 0x6c, 0x69, 0x63, 0x65, // "alice"
            0x01, 0x00, 0x00, 0x00, 0x50, // "P"
            0xa0, 0x00, 0x00, 0x00, // version=160
        ];
        let mut buf = bytes::Bytes::from_static(raw);
        let req = PeerInitRequest::read(&mut buf).unwrap();
        assert_eq!(req.username, "alice");
        assert_eq!(req.conn_type, "P");
        assert_eq!(req.version, 160);
    }
}
