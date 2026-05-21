// File transfer message structs
// These are sent over the F connection without a message code
// The first message is FileTransferInit, then raw bytes

pub const FILE_TRANSFER_INIT_CODE: u32 = 0; // placeholder, not really a code

use bytes::{Buf, BufMut};
use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;

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
        Ok(Self { token: u32::read(buf)? })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    use crate::codec::SlskRead;

    #[test]
    fn file_transfer_init_round_trip() {
        let req = FileTransferInit { token: 12345 };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(u32::read(&mut buf).unwrap(), 12345);
    }
}
