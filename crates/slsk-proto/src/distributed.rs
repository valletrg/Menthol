// Distributed message structs

use bytes::{Buf, BufMut};
use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;

pub const CODE: u8 = 3; // DistribSearch code

#[derive(Debug, Clone)]
pub struct DistribSearch {
    pub token: u32,
    pub query: String,
}

impl SlskWrite for DistribSearch {
    fn write(&self, buf: &mut impl BufMut) {
        self.token.write(buf);
        self.query.write(buf);
    }
}

impl SlskRead for DistribSearch {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self {
            token: u32::read(buf)?,
            query: String::read(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    use crate::codec::SlskRead;

    #[test]
    fn distrib_search_round_trip() {
        let req = DistribSearch {
            token: 42,
            query: "hello".into(),
        };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(u32::read(&mut buf).unwrap(), 42);
        assert_eq!(String::read(&mut buf).unwrap(), "hello");
    }
}
