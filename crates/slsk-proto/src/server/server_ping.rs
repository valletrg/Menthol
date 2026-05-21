use bytes::BufMut;
use crate::codec::SlskWrite;

pub const CODE: u32 = 32;

// ServerPing is send-only, empty payload
#[derive(Debug, Clone)]
pub struct ServerPingRequest;

impl SlskWrite for ServerPingRequest {
    fn write(&self, _buf: &mut impl BufMut) {
        // empty payload
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_ping_round_trip() {
        let req = ServerPingRequest;
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        assert_eq!(buf.len(), 0);
    }

    use bytes::BytesMut;
}
