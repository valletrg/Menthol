use crate::codec::SlskWrite;
use bytes::BufMut;

pub const CODE: u32 = 23;

#[derive(Debug, Clone)]
pub struct MessageAckedRequest {
    pub id: u32,
}

impl SlskWrite for MessageAckedRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.id.write(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::SlskRead;
    use bytes::BytesMut;

    #[test]
    fn message_acked_round_trip() {
        let req = MessageAckedRequest { id: 12345 };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(u32::read(&mut buf).unwrap(), 12345);
    }
}
