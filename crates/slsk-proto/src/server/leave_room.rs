use bytes::BufMut;
use crate::codec::SlskWrite;

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
    use bytes::BytesMut;
    use crate::codec::SlskRead;

    #[test]
    fn leave_room_round_trip() {
        let req = LeaveRoomRequest { room: "The Lobby".into() };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(String::read(&mut buf).unwrap(), "The Lobby");
    }
}
