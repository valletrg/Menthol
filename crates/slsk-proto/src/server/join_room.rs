use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;
use bytes::{Buf, BufMut};

pub const CODE: u32 = 14;

#[derive(Debug, Clone)]
pub struct JoinRoomRequest {
    pub room: String,
    pub private: u32,
}

impl SlskWrite for JoinRoomRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.room.write(buf);
        self.private.write(buf);
    }
}

#[derive(Debug, Clone)]
pub struct JoinRoomResponse {
    pub room: String,
    pub num_users: u32,
    pub users: Vec<String>,
    pub num_statuses: u32,
    pub statuses: Vec<u32>,
    pub num_stats: u32,
    pub stats: Vec<(u32, u32, u32, u32, u32)>, // avgspeed, uploadnum, unknown, files, dirs
    pub num_slotsfull: u32,
    pub slotsfull: Vec<u32>,
    pub num_countries: u32,
    pub countries: Vec<String>,
    // Private room fields omitted for now
}

impl SlskRead for JoinRoomResponse {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        let room = String::read(buf)?;
        let num_users = u32::read(buf)?;
        let mut users = Vec::new();
        for _ in 0..num_users {
            users.push(String::read(buf)?);
        }
        let num_statuses = u32::read(buf)?;
        let mut statuses = Vec::new();
        for _ in 0..num_statuses {
            statuses.push(u32::read(buf)?);
        }
        let num_stats = u32::read(buf)?;
        let mut stats = Vec::new();
        for _ in 0..num_stats {
            stats.push((
                u32::read(buf)?,
                u32::read(buf)?,
                u32::read(buf)?,
                u32::read(buf)?,
                u32::read(buf)?,
            ));
        }
        let num_slotsfull = u32::read(buf)?;
        let mut slotsfull = Vec::new();
        for _ in 0..num_slotsfull {
            slotsfull.push(u32::read(buf)?);
        }
        let num_countries = u32::read(buf)?;
        let mut countries = Vec::new();
        for _ in 0..num_countries {
            countries.push(String::read(buf)?);
        }
        Ok(Self {
            room,
            num_users,
            users,
            num_statuses,
            statuses,
            num_stats,
            stats,
            num_slotsfull,
            slotsfull,
            num_countries,
            countries,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::SlskRead;
    use bytes::BytesMut;

    #[test]
    fn join_room_request_round_trip() {
        let req = JoinRoomRequest {
            room: "The Lobby".into(),
            private: 0,
        };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(String::read(&mut buf).unwrap(), "The Lobby");
        assert_eq!(u32::read(&mut buf).unwrap(), 0);
    }
}
