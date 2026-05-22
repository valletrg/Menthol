use crate::codec::SlskRead;
use crate::error::ProtoError;
use bytes::Buf;

pub const CODE: u32 = 69;

#[derive(Debug, Clone)]
pub struct PrivilegedUsersResponse {
    pub num_users: u32,
    pub users: Vec<String>,
}

impl SlskRead for PrivilegedUsersResponse {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        let num_users = u32::read(buf)?;
        let mut users = Vec::new();
        for _ in 0..num_users {
            users.push(String::read(buf)?);
        }
        Ok(Self { num_users, users })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn privileged_users_decode() {
        // 2 users: "alice", "bob"
        let raw: &[u8] = &[
            0x02, 0x00, 0x00, 0x00, // num_users=2
            0x05, 0x00, 0x00, 0x00, 0x61, 0x6c, 0x69, 0x63, 0x65, // "alice"
            0x03, 0x00, 0x00, 0x00, 0x62, 0x6f, 0x62, // "bob"
        ];
        let mut buf = bytes::Bytes::from_static(raw);
        let resp = PrivilegedUsersResponse::read(&mut buf).unwrap();
        assert_eq!(resp.num_users, 2);
        assert_eq!(resp.users[0], "alice");
        assert_eq!(resp.users[1], "bob");
    }
}
