use crate::codec::SlskWrite;
use bytes::BufMut;

pub const CODE: u32 = 120;

#[derive(Debug, Clone)]
pub struct RoomSearchRequest {
    pub room: String,
    pub token: u32,
    pub query: String,
}

impl SlskWrite for RoomSearchRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.room.write(buf);
        self.token.write(buf);
        self.query.write(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::SlskRead;
    use bytes::BytesMut;

    #[test]
    fn room_search_round_trip() {
        let req = RoomSearchRequest {
            room: "The Lobby".into(),
            token: 99,
            query: "jazz".into(),
        };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(String::read(&mut buf).unwrap(), "The Lobby");
        assert_eq!(u32::read(&mut buf).unwrap(), 99);
        assert_eq!(String::read(&mut buf).unwrap(), "jazz");
    }
}
