use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;
use bytes::{Buf, BufMut};

pub const CODE: u32 = 41;

#[derive(Debug, Clone)]
pub struct TransferResponse {
    pub token: u32,
    pub allowed: bool,
    pub reason: Option<String>,
}

impl SlskWrite for TransferResponse {
    fn write(&self, buf: &mut impl BufMut) {
        self.token.write(buf);
        self.allowed.write(buf);
        if let Some(ref reason) = self.reason {
            reason.write(buf);
        }
    }
}

impl SlskRead for TransferResponse {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        let token = u32::read(buf)?;
        let allowed = bool::read(buf)?;
        let reason = if buf.has_remaining() {
            Some(String::read(buf)?)
        } else {
            None
        };
        Ok(Self {
            token,
            allowed,
            reason,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn transfer_response_allowed_round_trip() {
        let resp = TransferResponse {
            token: 12345,
            allowed: true,
            reason: None,
        };
        let mut buf = BytesMut::new();
        resp.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(u32::read(&mut buf).unwrap(), 12345);
        assert!(bool::read(&mut buf).unwrap());
    }

    #[test]
    fn transfer_response_denied_round_trip() {
        let resp = TransferResponse {
            token: 12345,
            allowed: false,
            reason: Some("File not shared.".into()),
        };
        let mut buf = BytesMut::new();
        resp.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(u32::read(&mut buf).unwrap(), 12345);
        assert!(!bool::read(&mut buf).unwrap());
        assert_eq!(
            String::read(&mut buf).unwrap(),
            "File not shared."
        );
    }
}
