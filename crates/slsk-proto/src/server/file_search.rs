use bytes::{Buf, BufMut};
use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;

pub const CODE: u32 = 26;

// Outgoing search request
#[derive(Debug, Clone)]
pub struct FileSearchRequest {
    pub token:      u32,
    pub query:      String,
}

impl SlskWrite for FileSearchRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.token.write(buf);
        self.query.write(buf);
    }
}

// Incoming search results from a peer (server relays FileSearch from peers)
#[derive(Debug, Clone)]
pub struct FileSearchResponse {
    pub username: String,
    pub token:    u32,
    pub query:    String,
}

impl SlskRead for FileSearchResponse {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self {
            username: String::read(buf)?,
            token:    u32::read(buf)?,
            query:    String::read(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    use crate::codec::SlskRead;

    #[test]
    fn file_search_request_round_trip() {
        let req = FileSearchRequest {
            token: 42,
            query: "hello world".into(),
        };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(u32::read(&mut buf).unwrap(), 42);
        assert_eq!(String::read(&mut buf).unwrap(), "hello world");
    }

    #[test]
    fn file_search_response_decode() {
        // username="alice" (5 bytes), token=42, query="hello" (5 bytes)
        let raw: &[u8] = &[
            0x05, 0x00, 0x00, 0x00, 0x61, 0x6c, 0x69, 0x63, 0x65, // "alice"
            0x2a, 0x00, 0x00, 0x00, // token=42
            0x05, 0x00, 0x00, 0x00, 0x68, 0x65, 0x6c, 0x6c, 0x6f, // "hello"
        ];
        let mut buf = bytes::Bytes::from_static(raw);
        let resp = FileSearchResponse::read(&mut buf).unwrap();
        assert_eq!(resp.username, "alice");
        assert_eq!(resp.token, 42);
        assert_eq!(resp.query, "hello");
    }
}
