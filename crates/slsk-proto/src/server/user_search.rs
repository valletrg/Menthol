use bytes::BufMut;
use crate::codec::SlskWrite;

pub const CODE: u32 = 42;

#[derive(Debug, Clone)]
pub struct UserSearchRequest {
    pub username: String,
    pub token:    u32,
    pub query:    String,
}

impl SlskWrite for UserSearchRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.username.write(buf);
        self.token.write(buf);
        self.query.write(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    use crate::codec::SlskRead;

    #[test]
    fn user_search_request_round_trip() {
        let req = UserSearchRequest {
            username: "alice".into(),
            token: 7,
            query: "mp3".into(),
        };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(String::read(&mut buf).unwrap(), "alice");
        assert_eq!(u32::read(&mut buf).unwrap(), 7);
        assert_eq!(String::read(&mut buf).unwrap(), "mp3");
    }
}
