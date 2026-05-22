use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;
use bytes::{Buf, BufMut};

pub const CODE: u32 = 92;

// CheckPrivileges is send-only (empty payload)
#[derive(Debug, Clone)]
pub struct CheckPrivilegesRequest;

impl SlskWrite for CheckPrivilegesRequest {
    fn write(&self, _buf: &mut impl BufMut) {}
}

#[derive(Debug, Clone)]
pub struct CheckPrivilegesResponse {
    pub time_left: u32,
}

impl SlskRead for CheckPrivilegesResponse {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self {
            time_left: u32::read(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_privileges_round_trip() {
        let req = CheckPrivilegesRequest;
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn check_privileges_response_decode() {
        // 100000 = 0x186A0 -> little-endian: 0xa0 0x86 0x01 0x00
        let raw: &[u8] = &[0xa0, 0x86, 0x01, 0x00];
        let mut buf = bytes::Bytes::from_static(raw);
        let resp = CheckPrivilegesResponse::read(&mut buf).unwrap();
        assert_eq!(resp.time_left, 100000);
    }

    use bytes::BytesMut;
}
