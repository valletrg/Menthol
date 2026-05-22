use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;
use bytes::{Buf, BufMut};

pub const CODE: u32 = 42;

#[derive(Debug, Clone)]
pub struct PlaceInQueueRequest {
    pub filename: String,
}

impl SlskWrite for PlaceInQueueRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.filename.write(buf);
    }
}

impl SlskRead for PlaceInQueueRequest {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self {
            filename: String::read(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn place_in_queue_request_round_trip() {
        let req = PlaceInQueueRequest {
            filename: "test.mp3".into(),
        };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(String::read(&mut buf).unwrap(), "test.mp3");
    }
}
