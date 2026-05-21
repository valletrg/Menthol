use bytes::BufMut;
use crate::codec::SlskWrite;

pub const CODE: u32 = 100;

#[derive(Debug, Clone)]
pub struct AcceptChildrenRequest {
    pub accept: bool,
}

impl SlskWrite for AcceptChildrenRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.accept.write(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    use crate::codec::SlskRead;

    #[test]
    fn accept_children_round_trip() {
        let req = AcceptChildrenRequest { accept: true };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(bool::read(&mut buf).unwrap(), true);
    }
}
