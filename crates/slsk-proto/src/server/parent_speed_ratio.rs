use bytes::{Buf, BufMut};
use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;

pub const CODE: u32 = 84;

// ParentSpeedRatio is receive-only
#[derive(Debug, Clone)]
pub struct ParentSpeedRatioResponse {
    pub ratio: u32,
}

impl SlskRead for ParentSpeedRatioResponse {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self { ratio: u32::read(buf)? })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_speed_ratio_decode() {
        let raw: &[u8] = &[0x02, 0x00, 0x00, 0x00]; // ratio=2
        let mut buf = bytes::Bytes::from_static(raw);
        let resp = ParentSpeedRatioResponse::read(&mut buf).unwrap();
        assert_eq!(resp.ratio, 2);
    }
}
