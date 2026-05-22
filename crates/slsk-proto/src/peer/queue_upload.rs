use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;
use bytes::{Buf, BufMut};

pub const CODE: u32 = 43;

#[derive(Debug, Clone)]
pub struct QueueUpload {
    pub username: String,
    pub filename: String,
}

impl SlskWrite for QueueUpload {
    fn write(&self, buf: &mut impl BufMut) {
        self.username.write(buf);
        self.filename.write(buf);
    }
}

impl SlskRead for QueueUpload {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self {
            username: String::read(buf)?,
            filename: String::read(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::SlskRead;
    use bytes::BytesMut;

    #[test]
    fn queue_upload_round_trip() {
        let req = QueueUpload {
            username: "alice".into(),
            filename: "song.mp3".into(),
        };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(String::read(&mut buf).unwrap(), "alice");
        assert_eq!(String::read(&mut buf).unwrap(), "song.mp3");
    }
}
