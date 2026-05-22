// File transfer message structs
// These are sent over the F connection without a message code
// The first message is FileTransferInit, then raw bytes

use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;
use bytes::{Buf, BufMut};

#[derive(Debug, Clone)]
pub struct FileTransferInit {
    pub token: u32,
}

impl SlskWrite for FileTransferInit {
    fn write(&self, buf: &mut impl BufMut) {
        self.token.write(buf);
    }
}

impl SlskRead for FileTransferInit {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self {
            token: u32::read(buf)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FileOffset {
    pub offset: u64,
}

impl SlskWrite for FileOffset {
    fn write(&self, buf: &mut impl BufMut) {
        self.offset.write(buf);
    }
}

impl SlskRead for FileOffset {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self {
            offset: u64::read(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn file_transfer_init_round_trip() {
        let req = FileTransferInit { token: 12345 };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(u32::read(&mut buf).unwrap(), 12345);
    }

    #[test]
    fn file_offset_round_trip() {
        let req = FileOffset { offset: 1_000_000 };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(u64::read(&mut buf).unwrap(), 1_000_000);
    }
}
