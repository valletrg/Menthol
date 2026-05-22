use crate::codec::SlskRead;
use crate::error::ProtoError;
use bytes::Buf;

pub const CODE: u32 = 44;

#[derive(Debug, Clone)]
pub struct PlaceInQueueResponse {
    pub filename: String,
    pub place: u32,
}

impl SlskRead for PlaceInQueueResponse {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self {
            filename: String::read(buf)?,
            place: u32::read(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn place_in_queue_response_decode() {
        let raw: &[u8] = &[
            0x04, 0x00, 0x00, 0x00, 0x74, 0x65, 0x73, 0x74, // "test"
            0x03, 0x00, 0x00, 0x00, // place=3
        ];
        let mut buf = bytes::Bytes::from_static(raw);
        let resp = PlaceInQueueResponse::read(&mut buf).unwrap();
        assert_eq!(resp.filename, "test");
        assert_eq!(resp.place, 3);
    }
}
