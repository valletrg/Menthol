use crate::codec::SlskWrite;
use bytes::BufMut;

pub const CODE: u32 = 28;

#[derive(Debug, Clone)]
pub struct SetStatusRequest {
    pub status: i32,
}

impl SlskWrite for SetStatusRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.status.write(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::SlskRead;
    use bytes::BytesMut;

    #[test]
    fn set_status_round_trip() {
        let req = SetStatusRequest { status: 1 }; // Away
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(i32::read(&mut buf).unwrap(), 1);
    }
}
