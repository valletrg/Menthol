use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;
use bytes::{Buf, BufMut};

pub const CODE: u32 = 50;

#[derive(Debug, Clone)]
pub struct UploadDenied {
    pub filename: String,
    pub reason: String,
}

impl SlskRead for UploadDenied {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self {
            filename: String::read(buf)?,
            reason: String::read(buf)?,
        })
    }
}

impl SlskWrite for UploadDenied {
    fn write(&self, buf: &mut impl BufMut) {
        self.filename.write(buf);
        self.reason.write(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::SlskRead;
    use bytes::BytesMut;

    #[test]
    fn upload_denied_round_trip() {
        let req = UploadDenied {
            filename: "song.mp3".into(),
            reason: "File not shared.".into(),
        };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(String::read(&mut buf).unwrap(), "song.mp3");
        assert_eq!(String::read(&mut buf).unwrap(), "File not shared.");
    }
}
