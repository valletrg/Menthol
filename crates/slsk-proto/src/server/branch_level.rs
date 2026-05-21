use bytes::BufMut;
use crate::codec::SlskWrite;

pub const CODE: u32 = 126;

#[derive(Debug, Clone)]
pub struct BranchLevelRequest {
    pub branch_level: u32,
}

impl SlskWrite for BranchLevelRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.branch_level.write(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    use crate::codec::SlskRead;

    #[test]
    fn branch_level_round_trip() {
        let req = BranchLevelRequest { branch_level: 3 };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(u32::read(&mut buf).unwrap(), 3);
    }
}
