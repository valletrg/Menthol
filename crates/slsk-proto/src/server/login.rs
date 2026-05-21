use bytes::{Buf, BufMut};
use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;

pub const CODE: u32 = 1;

#[derive(Debug, Clone)]
pub struct LoginRequest {
    pub username:      String,
    pub password:      String,
    pub major_version: u32,
    pub hash:          String,
    pub minor_version: u32,
}

impl SlskWrite for LoginRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.username.write(buf);
        self.password.write(buf);
        self.major_version.write(buf);
        self.hash.write(buf);
        self.minor_version.write(buf);
    }
}

#[derive(Debug, Clone)]
pub enum LoginResponse {
    Success {
        greet:        String,
        own_ip:       u32,
        hash:         String,
        is_supporter: bool,
    },
    Failure {
        reason: String,
        detail: Option<String>,
    },
}

impl SlskRead for LoginResponse {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        let success = bool::read(buf)?;
        if success {
            Ok(Self::Success {
                greet:        String::read(buf)?,
                own_ip:       u32::read(buf)?,
                hash:         String::read(buf)?,
                is_supporter: bool::read(buf)?,
            })
        } else {
            let reason = String::read(buf)?;
            let detail = if reason == "INVALIDUSERNAME" {
                Some(String::read(buf)?)
            } else {
                None
            };
            Ok(Self::Failure { reason, detail })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_request_round_trip() {
        let req = LoginRequest {
            username:      "testuser".into(),
            password:      "testpass".into(),
            major_version: 160,
            hash:          "d51c9a7e9353746a6020f9602d452929".into(),
            minor_version: 2,
        };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(String::read(&mut buf).unwrap(), "testuser");
        assert_eq!(String::read(&mut buf).unwrap(), "testpass");
        assert_eq!(u32::read(&mut buf).unwrap(), 160);
        assert_eq!(String::read(&mut buf).unwrap(), "d51c9a7e9353746a6020f9602d452929");
        assert_eq!(u32::read(&mut buf).unwrap(), 2);
    }

    #[test]
    fn login_response_success_decode() {
        // success=true (1), greet="hello", own_ip=0x01020304, hash="abc123", is_supporter=true (1)
        let raw: &[u8] = &[
            0x01, // success
            0x05, 0x00, 0x00, 0x00, // greet len
            0x68, 0x65, 0x6c, 0x6c, 0x6f, // "hello"
            0x04, 0x03, 0x02, 0x01, // own_ip little-endian
            0x06, 0x00, 0x00, 0x00, // hash len
            0x61, 0x62, 0x63, 0x31, 0x32, 0x33, // "abc123"
            0x01, // is_supporter
        ];
        let mut buf = bytes::Bytes::from_static(raw);
        let resp = LoginResponse::read(&mut buf).unwrap();
        match resp {
            LoginResponse::Success { greet, own_ip, hash, is_supporter } => {
                assert_eq!(greet, "hello");
                assert_eq!(own_ip, 0x01020304);
                assert_eq!(hash, "abc123");
                assert!(is_supporter);
            }
            _ => panic!("expected Success"),
        }
    }

    #[test]
    fn login_response_failure_decode() {
        // success=false (0), reason="INVALIDPASS"
        let raw: &[u8] = &[
            0x00, // success
            0x0b, 0x00, 0x00, 0x00, // reason len
            0x49, 0x4e, 0x56, 0x41, 0x4c, 0x49, 0x44, 0x50, 0x41, 0x53, 0x53, // "INVALIDPASS"
        ];
        let mut buf = bytes::Bytes::from_static(raw);
        let resp = LoginResponse::read(&mut buf).unwrap();
        match resp {
            LoginResponse::Failure { reason, detail } => {
                assert_eq!(reason, "INVALIDPASS");
                assert!(detail.is_none());
            }
            _ => panic!("expected Failure"),
        }
    }

    use bytes::BytesMut;
}
