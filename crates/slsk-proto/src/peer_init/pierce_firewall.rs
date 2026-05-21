use bytes::{Buf, BufMut};
use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;

pub const CODE: u8 = 0;

// PierceFirewall is sent by the peer that initiated the indirect connection
// The token comes from the ConnectToPeer message we received from the server
#[derive(Debug, Clone)]
pub struct PierceFirewallRequest {
    pub token: u32,
}

impl SlskWrite for PierceFirewallRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.token.write(buf);
    }
}

impl SlskRead for PierceFirewallRequest {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self { token: u32::read(buf)? })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    use crate::codec::SlskRead;

    #[test]
    fn pierce_firewall_request_round_trip() {
        let req = PierceFirewallRequest { token: 12345 };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(u32::read(&mut buf).unwrap(), 12345);
    }

    #[test]
    fn pierce_firewall_request_decode() {
        let raw: &[u8] = &[0x39, 0x30, 0x00, 0x00]; // token=12345
        let mut buf = bytes::Bytes::from_static(raw);
        let req = PierceFirewallRequest::read(&mut buf).unwrap();
        assert_eq!(req.token, 12345);
    }
}
