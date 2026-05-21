use bytes::Buf;

pub const CODE: u32 = 130;

// ResetDistributed is receive-only, empty payload
#[derive(Debug, Clone)]
pub struct ResetDistributed;

impl crate::codec::SlskRead for ResetDistributed {
    fn read(_buf: &mut impl Buf) -> Result<Self, crate::error::ProtoError> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    use crate::codec::SlskRead;

    #[test]
    fn reset_distributed_decode() {
        let raw: &[u8] = &[];
        let mut buf = bytes::Bytes::from_static(raw);
        ResetDistributed::read(&mut buf).unwrap();
    }
}
