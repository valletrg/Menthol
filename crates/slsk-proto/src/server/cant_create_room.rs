use bytes::Buf;

pub const CODE: u32 = 1003;

// CantCreateRoom is receive-only
#[derive(Debug, Clone)]
pub struct CantCreateRoom {
    pub room: String,
}

impl crate::codec::SlskRead for CantCreateRoom {
    fn read(buf: &mut impl Buf) -> Result<Self, crate::error::ProtoError> {
        Ok(Self {
            room: String::read(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::SlskRead;
    use bytes::BytesMut;

    #[test]
    fn cant_create_room_decode() {
        let raw: &[u8] = &[
            0x05, 0x00, 0x00, 0x00, // room len
            0x72, 0x6f, 0x6f, 0x6d, 0x73, // "rooms"
        ];
        let mut buf = bytes::Bytes::from_static(raw);
        let resp = CantCreateRoom::read(&mut buf).unwrap();
        assert_eq!(resp.room, "rooms");
    }
}
