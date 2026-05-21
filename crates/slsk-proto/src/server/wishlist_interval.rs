use bytes::Buf;

pub const CODE: u32 = 104;

// WishlistInterval is receive-only
#[derive(Debug, Clone)]
pub struct WishlistInterval {
    pub interval: u32,
}

impl crate::codec::SlskRead for WishlistInterval {
    fn read(buf: &mut impl Buf) -> Result<Self, crate::error::ProtoError> {
        Ok(Self { interval: u32::read(buf)? })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    use crate::codec::SlskRead;

    #[test]
    fn wishlist_interval_decode() {
        // 12 minutes = 720 seconds
        let raw: &[u8] = &[0xD0, 0x02, 0x00, 0x00];
        let mut buf = bytes::Bytes::from_static(raw);
        let resp = WishlistInterval::read(&mut buf).unwrap();
        assert_eq!(resp.interval, 720);
    }
}
