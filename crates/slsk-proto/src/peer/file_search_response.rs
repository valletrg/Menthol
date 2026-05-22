use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;
use bytes::{Buf, BufMut};

pub const CODE: u32 = 9;

#[derive(Debug, Clone)]
pub struct FileSearchResponse {
    pub token: u32,
    pub result: FileSearchResult,
}

#[derive(Debug, Clone)]
pub struct FileSearchResult {
    pub filename: String,
    pub size: u64,
    pub checksum: u32,
    pub attributes: Vec<(u32, u32)>,
}

impl SlskRead for FileSearchResult {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        let filename = String::read(buf)?;
        let size = u64::read(buf)?;
        let checksum = u32::read(buf)?;
        let num_attrs = u32::read(buf)?;
        let mut attributes = Vec::new();
        for _ in 0..num_attrs {
            attributes.push((u32::read(buf)?, u32::read(buf)?));
        }
        Ok(Self {
            filename,
            size,
            checksum,
            attributes,
        })
    }
}

impl SlskRead for FileSearchResponse {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        Ok(Self {
            token: u32::read(buf)?,
            result: FileSearchResult::read(buf)?,
        })
    }
}

impl SlskWrite for FileSearchResponse {
    fn write(&self, buf: &mut impl BufMut) {
        self.token.write(buf);
        self.result.filename.write(buf);
        self.result.size.write(buf);
        self.result.checksum.write(buf);
        (self.result.attributes.len() as u32).write(buf);
        for (code, val) in &self.result.attributes {
            code.write(buf);
            val.write(buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::SlskRead;
    use bytes::BytesMut;

    #[test]
    fn file_search_response_round_trip() {
        let resp = FileSearchResponse {
            token: 42,
            result: FileSearchResult {
                filename: "test.mp3".into(),
                size: 1000,
                checksum: 0,
                attributes: vec![],
            },
        };
        let mut buf = BytesMut::new();
        resp.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(u32::read(&mut buf).unwrap(), 42);
        assert_eq!(String::read(&mut buf).unwrap(), "test.mp3");
    }
}
