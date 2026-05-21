use bytes::{Buf, BufMut};
use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;

pub const CODE: u32 = 64;

#[derive(Debug, Clone)]
pub struct RoomListResponse {
    pub num_rooms:         u32,
    pub rooms:             Vec<String>,
    pub num_counts:         u32,
    pub user_counts:        Vec<u32>,
    pub num_owned:          u32,
    pub owned_rooms:        Vec<String>,
    pub num_owned_counts:   u32,
    pub owned_user_counts:  Vec<u32>,
    pub num_private:        u32,
    pub private_rooms:      Vec<String>,
    pub num_private_counts: u32,
    pub private_user_counts: Vec<u32>,
    pub num_operated:       u32,
    pub operated_rooms:     Vec<String>,
}

impl SlskRead for RoomListResponse {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        let num_rooms = u32::read(buf)?;
        let mut rooms = Vec::new();
        for _ in 0..num_rooms {
            rooms.push(String::read(buf)?);
        }
        let num_counts = u32::read(buf)?;
        let mut user_counts = Vec::new();
        for _ in 0..num_counts {
            user_counts.push(u32::read(buf)?);
        }
        let num_owned = u32::read(buf)?;
        let mut owned_rooms = Vec::new();
        for _ in 0..num_owned {
            owned_rooms.push(String::read(buf)?);
        }
        let num_owned_counts = u32::read(buf)?;
        let mut owned_user_counts = Vec::new();
        for _ in 0..num_owned_counts {
            owned_user_counts.push(u32::read(buf)?);
        }
        let num_private = u32::read(buf)?;
        let mut private_rooms = Vec::new();
        for _ in 0..num_private {
            private_rooms.push(String::read(buf)?);
        }
        let num_private_counts = u32::read(buf)?;
        let mut private_user_counts = Vec::new();
        for _ in 0..num_private_counts {
            private_user_counts.push(u32::read(buf)?);
        }
        let num_operated = u32::read(buf)?;
        let mut operated_rooms = Vec::new();
        for _ in 0..num_operated {
            operated_rooms.push(String::read(buf)?);
        }
        Ok(Self {
            num_rooms,
            rooms,
            num_counts,
            user_counts,
            num_owned,
            owned_rooms,
            num_owned_counts,
            owned_user_counts,
            num_private,
            private_rooms,
            num_private_counts,
            private_user_counts,
            num_operated,
            operated_rooms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn room_list_decode_minimal() {
        // num_rooms=1, room="Lobby", count=5, no private/owned rooms
        let raw: &[u8] = &[
            0x01, 0x00, 0x00, 0x00, // num_rooms=1
            0x05, 0x00, 0x00, 0x00, 0x4c, 0x6f, 0x62, 0x62, 0x79, // "Lobby"
            0x01, 0x00, 0x00, 0x00, // num_counts=1
            0x05, 0x00, 0x00, 0x00, // user_count=5
            0x00, 0x00, 0x00, 0x00, // num_owned=0
            0x00, 0x00, 0x00, 0x00, // num_owned_counts=0
            0x00, 0x00, 0x00, 0x00, // num_private=0
            0x00, 0x00, 0x00, 0x00, // num_private_counts=0
            0x00, 0x00, 0x00, 0x00, // num_operated=0
        ];
        let mut buf = bytes::Bytes::from_static(raw);
        let resp = RoomListResponse::read(&mut buf).unwrap();
        assert_eq!(resp.num_rooms, 1);
        assert_eq!(resp.rooms[0], "Lobby");
        assert_eq!(resp.user_counts[0], 5);
    }
}
