use bytes::{Buf, BufMut};
use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;

pub const CODE: u32 = 3;

#[derive(Debug, Clone)]
pub struct GetPeerAddressRequest {
    pub username: String,
}

impl SlskWrite for GetPeerAddressRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.username.write(buf);
    }
}

#[derive(Debug, Clone)]
pub struct GetPeerAddressResponse {
    pub username:       String,
    pub ip:             u32,
    pub port:           u32,
    pub obfuscation:    u32,
    pub obfuscated_port: u16,
}

impl SlskRead for GetPeerAddressResponse {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self {
            username:        String::read(buf)?,
            ip:              u32::read(buf)?,
            port:            u32::read(buf)?,
            obfuscation:     u32::read(buf)?,
            obfuscated_port: u16::read(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    use crate::codec::SlskRead;

    #[test]
    fn get_peer_address_request_round_trip() {
        let req = GetPeerAddressRequest { username: "testuser".into() };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(String::read(&mut buf).unwrap(), "testuser");
    }

    #[test]
    fn get_peer_address_response_decode() {
        // username="alice", ip=192.168.1.100 (0xc0a80164), port=2234 (0x8ba -> little-endian: ba 08)
        let raw: &[u8] = &[
            0x05, 0x00, 0x00, 0x00, // username len
            0x61, 0x6c, 0x69, 0x63, 0x65, // "alice"
            0x64, 0x01, 0xa8, 0xc0, // ip: 192.168.1.100 (little-endian)
            0xba, 0x08, 0x00, 0x00, // port: 2234 (little-endian)
            0x00, 0x00, 0x00, 0x00, // obfuscation: 0
            0xba, 0x08, // obfuscated_port: 2234 (little-endian u16)
        ];
        let mut buf = bytes::Bytes::from_static(raw);
        let resp = GetPeerAddressResponse::read(&mut buf).unwrap();
        assert_eq!(resp.username, "alice");
        assert_eq!(resp.ip, 0xc0a80164);
        assert_eq!(resp.port, 2234);
        assert_eq!(resp.obfuscation, 0);
        assert_eq!(resp.obfuscated_port, 2234);
    }
}
