use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;
use bytes::{Buf, BufMut};

pub const CODE: u32 = 36;

#[derive(Debug, Clone)]
pub struct FolderContentsRequest {
    pub token: u32,
    pub folder: String,
}

impl SlskWrite for FolderContentsRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.token.write(buf);
        self.folder.write(buf);
    }
}

impl SlskRead for FolderContentsRequest {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self {
            token: u32::read(buf)?,
            folder: String::read(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::SlskRead;
    use bytes::BytesMut;

    #[test]
    fn folder_contents_request_round_trip() {
        let req = FolderContentsRequest {
            token: 42,
            folder: "Music".into(),
        };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(u32::read(&mut buf).unwrap(), 42);
        assert_eq!(String::read(&mut buf).unwrap(), "Music");
    }
}
