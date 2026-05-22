use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;
use bytes::{Buf, BufMut};

pub const CODE: u32 = 36;

#[derive(Debug, Clone)]
pub struct GetUserStatsRequest {
    pub username: String,
}

impl SlskWrite for GetUserStatsRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.username.write(buf);
    }
}

#[derive(Debug, Clone)]
pub struct GetUserStatsResponse {
    pub username: String,
    pub avgspeed: u32,
    pub uploadnum: u32,
    pub unknown: u32,
    pub files: u32,
    pub dirs: u32,
}

impl SlskRead for GetUserStatsResponse {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self {
            username: String::read(buf)?,
            avgspeed: u32::read(buf)?,
            uploadnum: u32::read(buf)?,
            unknown: u32::read(buf)?,
            files: u32::read(buf)?,
            dirs: u32::read(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::SlskRead;
    use bytes::BytesMut;

    #[test]
    fn get_user_stats_request_round_trip() {
        let req = GetUserStatsRequest {
            username: "alice".into(),
        };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(String::read(&mut buf).unwrap(), "alice");
    }

    #[test]
    fn get_user_stats_response_decode() {
        let raw: &[u8] = &[
            0x05, 0x00, 0x00, 0x00, 0x61, 0x6c, 0x69, 0x63, 0x65, // "alice"
            0x64, 0x00, 0x00, 0x00, // avgspeed=100
            0x0a, 0x00, 0x00, 0x00, // uploadnum=10
            0x00, 0x00, 0x00, 0x00, // unknown=0
            0x96, 0x00, 0x00, 0x00, // files=150
            0x05, 0x00, 0x00, 0x00, // dirs=5
        ];
        let mut buf = bytes::Bytes::from_static(raw);
        let resp = GetUserStatsResponse::read(&mut buf).unwrap();
        assert_eq!(resp.username, "alice");
        assert_eq!(resp.avgspeed, 100);
        assert_eq!(resp.files, 150);
    }
}
