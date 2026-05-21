use bytes::{Buf, BufMut};
use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;

pub const CODE: u32 = 57;

// UserInterests is bidirectional
#[derive(Debug, Clone)]
pub struct UserInterestsRequest {
    pub username: String,
}

impl SlskWrite for UserInterestsRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.username.write(buf);
    }
}

#[derive(Debug, Clone)]
pub struct UserInterestsResponse {
    pub username:        String,
    pub num_likes:       u32,
    pub likes:           Vec<String>,
    pub num_hates:       u32,
    pub hates:           Vec<String>,
}

impl SlskRead for UserInterestsResponse {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        let username = String::read(buf)?;
        let num_likes = u32::read(buf)?;
        let mut likes = Vec::new();
        for _ in 0..num_likes {
            likes.push(String::read(buf)?);
        }
        let num_hates = u32::read(buf)?;
        let mut hates = Vec::new();
        for _ in 0..num_hates {
            hates.push(String::read(buf)?);
        }
        Ok(Self { username, num_likes, likes, num_hates, hates })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_interests_request_round_trip() {
        let req = UserInterestsRequest { username: "alice".into() };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(String::read(&mut buf).unwrap(), "alice");
    }

    use bytes::BytesMut;
}
