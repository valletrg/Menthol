use crate::codec::SlskWrite;
use bytes::BufMut;

pub const CODE: u32 = 6;

#[derive(Debug, Clone)]
pub struct UnwatchUserRequest {
    pub username: String,
}

impl SlskWrite for UnwatchUserRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.username.write(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::SlskRead;
    use bytes::BytesMut;

    #[test]
    fn unwatch_user_round_trip() {
        let req = UnwatchUserRequest {
            username: "alice".into(),
        };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(String::read(&mut buf).unwrap(), "alice");
    }
}
