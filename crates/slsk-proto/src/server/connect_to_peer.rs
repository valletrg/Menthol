use bytes::{Buf, BufMut};
use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;
use crate::types::ConnectionType;

pub const CODE: u32 = 18;

#[derive(Debug, Clone)]
pub struct ConnectToPeerRequest {
    pub token:    u32,
    pub username: String,
    pub conn_type: ConnectionType,
}

impl SlskWrite for ConnectToPeerRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.token.write(buf);
        self.username.write(buf);
        self.conn_type.write(buf);
    }
}

#[derive(Debug, Clone)]
pub struct ConnectToPeerResponse {
    pub username:        String,
    pub conn_type:       String,
    pub ip:             u32,
    pub port:           u32,
    pub token:          u32,
    pub privileged:     bool,
    pub obfuscation:    u32,
    pub obfuscated_port: u32,
}

impl SlskRead for ConnectToPeerResponse {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self {
            username:        String::read(buf)?,
            conn_type:       String::read(buf)?,
            ip:              u32::read(buf)?,
            port:            u32::read(buf)?,
            token:           u32::read(buf)?,
            privileged:      bool::read(buf)?,
            obfuscation:     u32::read(buf)?,
            obfuscated_port: u32::read(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    use crate::codec::SlskRead;

    #[test]
    fn connect_to_peer_request_round_trip() {
        let req = ConnectToPeerRequest {
            token: 12345,
            username: "alice".into(),
            conn_type: ConnectionType::PeerToPeer,
        };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(u32::read(&mut buf).unwrap(), 12345);
        assert_eq!(String::read(&mut buf).unwrap(), "alice");
        assert_eq!(u8::read(&mut buf).unwrap(), b'P');
    }
}
