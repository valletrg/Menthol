use bytes::{Buf, BufMut};
use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;

pub const CODE: u32 = 46;

#[derive(Debug, Clone)]
pub struct UploadFailed {
    pub filename: String,
}

impl SlskRead for UploadFailed {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self { filename: String::read(buf)? })
    }
}

impl SlskWrite for UploadFailed {
    fn write(&self, buf: &mut impl BufMut) {
        self.filename.write(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    use crate::codec::SlskRead;

    #[test]
    fn upload_failed_round_trip() {
        let req = UploadFailed { filename: "song.mp3".into() };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(String::read(&mut buf).unwrap(), "song.mp3");
    }
}
