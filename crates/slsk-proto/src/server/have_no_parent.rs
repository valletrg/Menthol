use bytes::BufMut;
use crate::codec::SlskWrite;

pub const CODE: u32 = 71;

#[derive(Debug, Clone)]
pub struct HaveNoParentRequest {
    pub no_parent: bool,
}

impl SlskWrite for HaveNoParentRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.no_parent.write(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    use crate::codec::SlskRead;

    #[test]
    fn have_no_parent_round_trip() {
        let req = HaveNoParentRequest { no_parent: true };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(bool::read(&mut buf).unwrap(), true);
    }
}
