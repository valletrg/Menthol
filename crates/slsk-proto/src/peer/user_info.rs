use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;
use bytes::{Buf, BufMut};

pub const CODE: u32 = 15;

#[derive(Debug, Clone)]
pub struct UserInfoRequest;

impl SlskWrite for UserInfoRequest {
    fn write(&self, _buf: &mut impl BufMut) {}
}

impl SlskRead for UserInfoRequest {
    fn read(_buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_info_request_decode() {
        let raw: &[u8] = &[];
        let mut buf = bytes::Bytes::from_static(raw);
        UserInfoRequest::read(&mut buf).unwrap();
    }
}
