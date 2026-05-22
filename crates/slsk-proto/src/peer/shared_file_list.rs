use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;
use bytes::{Buf, BufMut};

pub const CODE: u32 = 4;

#[derive(Debug, Clone)]
pub struct SharedFileListRequest;

impl SlskWrite for SharedFileListRequest {
    fn write(&self, _buf: &mut impl BufMut) {}
}

impl SlskRead for SharedFileListRequest {
    fn read(_buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_file_list_request_decode() {
        let raw: &[u8] = &[];
        let mut buf = bytes::Bytes::from_static(raw);
        SharedFileListRequest::read(&mut buf).unwrap();
    }
}
