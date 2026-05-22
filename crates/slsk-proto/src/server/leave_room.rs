use crate::codec::SlskWrite;
use bytes::BufMut;

pub const CODE: u32 = 15;

#[derive(Debug, Clone)]
pub struct LeaveRoomRequest {
    pub room: String,
}

impl SlskWrite for LeaveRoomRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.room.write(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::SlskRead;
    use bytes::BytesMut;

    #[test]
    fn leave_room_round_trip() {
        let req = LeaveRoomRequest {
            room: "The Lobby".into(),
        };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(String::read(&mut buf).unwrap(), "The Lobby");
    }
}
