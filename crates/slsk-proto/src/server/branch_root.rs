use bytes::BufMut;
use crate::codec::SlskWrite;

pub const CODE: u32 = 127;

#[derive(Debug, Clone)]
pub struct BranchRootRequest {
    pub branch_root: String,
}

impl SlskWrite for BranchRootRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.branch_root.write(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    use crate::codec::SlskRead;

    #[test]
    fn branch_root_round_trip() {
        let req = BranchRootRequest { branch_root: "rootuser".into() };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(String::read(&mut buf).unwrap(), "rootuser");
    }
}
