use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;
use crate::types::TransferDirection;
use bytes::{Buf, BufMut};

pub const CODE: u32 = 40;

#[derive(Debug, Clone)]
pub struct TransferRequest {
    pub direction: TransferDirection,
    pub token: u32,
    pub username: String,
    pub filename: String,
    pub file_size: Option<u64>,
}

impl SlskWrite for TransferRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.direction.write(buf);
        self.token.write(buf);
        self.username.write(buf);
        self.filename.write(buf);
        if self.direction == TransferDirection::Upload {
            if let Some(size) = self.file_size {
                size.write(buf);
            }
        }
    }
}

impl SlskRead for TransferRequest {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        let direction = TransferDirection::read(buf)?;
        let token = u32::read(buf)?;
        let username = String::read(buf)?;
        let filename = String::read(buf)?;
        let file_size = if direction == TransferDirection::Upload && buf.has_remaining() {
            Some(u64::read(buf)?)
        } else {
            None
        };
        Ok(Self {
            direction,
            token,
            username,
            filename,
            file_size,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::SlskRead;
    use bytes::BytesMut;

    #[test]
    fn transfer_request_download_round_trip() {
        let req = TransferRequest {
            direction: TransferDirection::Download,
            token: 123,
            username: "alice".into(),
            filename: "song.mp3".into(),
            file_size: None,
        };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(
            TransferDirection::read(&mut buf).unwrap(),
            TransferDirection::Download
        );
        assert_eq!(u32::read(&mut buf).unwrap(), 123);
        assert_eq!(String::read(&mut buf).unwrap(), "alice");
        assert_eq!(String::read(&mut buf).unwrap(), "song.mp3");
    }
}
