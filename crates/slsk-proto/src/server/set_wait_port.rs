use bytes::BufMut;
use crate::codec::SlskWrite;

pub const CODE: u32 = 2;

#[derive(Debug, Clone)]
pub struct SetWaitPortRequest {
    pub port: u32,
    // Optional obfuscation fields (not used by us)
}

impl SetWaitPortRequest {
    pub fn new(port: u32) -> Self {
        Self { port }
    }
}

impl SlskWrite for SetWaitPortRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.port.write(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_wait_port_round_trip() {
        let req = SetWaitPortRequest::new(2234);
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(u32::read(&mut buf).unwrap(), 2234);
    }

    use bytes::BytesMut;
    use crate::codec::SlskRead;
}
