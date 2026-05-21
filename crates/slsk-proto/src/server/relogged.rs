use bytes::Buf;

pub const CODE: u32 = 41;

// Relogged is receive-only, empty payload — server sends this then disconnects us
#[derive(Debug, Clone)]
pub struct Relogged;

impl crate::codec::SlskRead for Relogged {
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
    fn relogged_decode() {
        let raw: &[u8] = &[];
        let mut buf = bytes::Bytes::from_static(raw);
        Relogged::read(&mut buf).unwrap();
    }
}
