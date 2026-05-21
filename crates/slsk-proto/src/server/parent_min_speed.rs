use bytes::{Buf, BufMut};
use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;

pub const CODE: u32 = 83;

// ParentMinSpeed is receive-only
#[derive(Debug, Clone)]
pub struct ParentMinSpeedResponse {
    pub speed: u32,
}

impl SlskRead for ParentMinSpeedResponse {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self { speed: u32::read(buf)? })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_min_speed_decode() {
        // speed=7500 = 0x1D4C, little-endian: 0x4C 0x1D 0x00 0x00
        let raw: &[u8] = &[0x4C, 0x1D, 0x00, 0x00];
        let mut buf = bytes::Bytes::from_static(raw);
        let resp = ParentMinSpeedResponse::read(&mut buf).unwrap();
        assert_eq!(resp.speed, 7500);
    }
}
