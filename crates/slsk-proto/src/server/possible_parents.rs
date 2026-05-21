use bytes::{Buf, BufMut};
use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;

pub const CODE: u32 = 102;

#[derive(Debug, Clone)]
pub struct PossibleParentsResponse {
    pub num_parents: u32,
    pub parents:     Vec<Parent>,
}

#[derive(Debug, Clone)]
pub struct Parent {
    pub username: String,
    pub ip:       u32,
    pub port:     u32,
}

impl SlskRead for PossibleParentsResponse {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        let num_parents = u32::read(buf)?;
        let mut parents = Vec::new();
        for _ in 0..num_parents {
            parents.push(Parent {
                username: String::read(buf)?,
                ip:       u32::read(buf)?,
                port:     u32::read(buf)?,
            });
        }
        Ok(Self { num_parents, parents })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn possible_parents_decode() {
        // 1 parent: "rootnode", ip=10.0.0.1, port=2234
        let raw: &[u8] = &[
            0x01, 0x00, 0x00, 0x00, // num_parents=1
            0x08, 0x00, 0x00, 0x00, 0x72, 0x6f, 0x6f, 0x74, 0x6e, 0x6f, 0x64, 0x65, // "rootnode"
            0x01, 0x00, 0x00, 0x0a, // ip=10.0.0.1
            0xca, 0x08, 0x00, 0x00, // port=2234
        ];
        let mut buf = bytes::Bytes::from_static(raw);
        let resp = PossibleParentsResponse::read(&mut buf).unwrap();
        assert_eq!(resp.num_parents, 1);
        assert_eq!(resp.parents[0].username, "rootnode");
    }
}
