use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;
use crate::types::UserStatus;
use bytes::{Buf, BufMut};

pub const CODE: u32 = 5;

#[derive(Debug, Clone)]
pub struct WatchUserRequest {
    pub username: String,
}

impl SlskWrite for WatchUserRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.username.write(buf);
    }
}

#[derive(Debug, Clone)]
pub struct WatchUserResponse {
    pub username: String,
    pub exists: bool,
    pub status: Option<UserStatus>,
    pub avgspeed: Option<u32>,
    pub uploadnum: Option<u32>,
    pub unknown: Option<u32>,
    pub files: Option<u32>,
    pub dirs: Option<u32>,
    pub country: Option<String>,
}

impl SlskRead for WatchUserResponse {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        let username = String::read(buf)?;
        let exists = bool::read(buf)?;
        if !exists {
            return Ok(Self {
                username,
                exists: false,
                status: None,
                avgspeed: None,
                uploadnum: None,
                unknown: None,
                files: None,
                dirs: None,
                country: None,
            });
        }
        let status = Some(UserStatus::try_from(u32::read(buf)?)?);
        let avgspeed = Some(u32::read(buf)?);
        let uploadnum = Some(u32::read(buf)?);
        let unknown = Some(u32::read(buf)?);
        let files = Some(u32::read(buf)?);
        let dirs = Some(u32::read(buf)?);
        let country = if status == Some(UserStatus::Online) || status == Some(UserStatus::Away) {
            Some(String::read(buf)?)
        } else {
            None
        };
        Ok(Self {
            username,
            exists: true,
            status,
            avgspeed,
            uploadnum,
            unknown,
            files,
            dirs,
            country,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::SlskRead;
    use bytes::BytesMut;

    #[test]
    fn watch_user_request_round_trip() {
        let req = WatchUserRequest {
            username: "alice".into(),
        };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(String::read(&mut buf).unwrap(), "alice");
    }

    #[test]
    fn watch_user_response_exists() {
        // exists=true, status=2 (Online), avgspeed=100, uploadnum=5, unknown=0, files=10, dirs=3, country="US"
        let raw: &[u8] = &[
            0x05, 0x00, 0x00, 0x00, 0x61, 0x6c, 0x69, 0x63, 0x65, // username "alice"
            0x01, // exists=true
            0x02, 0x00, 0x00, 0x00, // status=2 (Online)
            0x64, 0x00, 0x00, 0x00, // avgspeed=100
            0x05, 0x00, 0x00, 0x00, // uploadnum=5
            0x00, 0x00, 0x00, 0x00, // unknown=0
            0x0a, 0x00, 0x00, 0x00, // files=10
            0x03, 0x00, 0x00, 0x00, // dirs=3
            0x02, 0x00, 0x00, 0x00, 0x55, 0x53, // country "US"
        ];
        let mut buf = bytes::Bytes::from_static(raw);
        let resp = WatchUserResponse::read(&mut buf).unwrap();
        assert!(resp.exists);
        assert_eq!(resp.status, Some(UserStatus::Online));
        assert_eq!(resp.avgspeed, Some(100));
        assert_eq!(resp.files, Some(10));
        assert_eq!(resp.country, Some("US".into()));
    }

    #[test]
    fn watch_user_response_not_exists() {
        // username="ghost", exists=false
        let raw: &[u8] = &[
            0x05, 0x00, 0x00, 0x00, 0x67, 0x68, 0x6f, 0x73, 0x74, // "ghost"
            0x00, // exists=false
        ];
        let mut buf = bytes::Bytes::from_static(raw);
        let resp = WatchUserResponse::read(&mut buf).unwrap();
        assert!(!resp.exists);
        assert!(resp.status.is_none());
    }
}
