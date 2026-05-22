use crate::codec::SlskRead;
use crate::error::ProtoError;
use bytes::Buf;

pub const CODE: u32 = 110;

// SimilarUsers is receive-only
#[derive(Debug, Clone)]
pub struct SimilarUsersResponse {
    pub num_users: u32,
    pub users: Vec<(String, u32)>,
}

impl SlskRead for SimilarUsersResponse {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        let num_users = u32::read(buf)?;
        let mut users = Vec::new();
        for _ in 0..num_users {
            users.push((String::read(buf)?, u32::read(buf)?));
        }
        Ok(Self { num_users, users })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn similar_users_decode() {
        // 1 user: "rockfan" (7 bytes), rating=8
        let raw: &[u8] = &[
            0x01, 0x00, 0x00, 0x00, // num=1
            0x07, 0x00, 0x00, 0x00, 0x72, 0x6f, 0x63, 0x6b, 0x66, 0x61, 0x6e, // "rockfan"
            0x08, 0x00, 0x00, 0x00, // rating=8
        ];
        let mut buf = bytes::Bytes::from_static(raw);
        let resp = SimilarUsersResponse::read(&mut buf).unwrap();
        assert_eq!(resp.num_users, 1);
        assert_eq!(resp.users[0].0, "rockfan");
    }
}
