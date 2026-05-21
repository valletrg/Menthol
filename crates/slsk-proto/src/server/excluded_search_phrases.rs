use bytes::{Buf, BufMut};
use crate::codec::{SlskRead, SlskWrite};
use crate::error::ProtoError;

pub const CODE: u32 = 160;

#[derive(Debug, Clone)]
pub struct ExcludedSearchPhrasesResponse {
    pub num_phrases: u32,
    pub phrases:     Vec<String>,
}

impl SlskRead for ExcludedSearchPhrasesResponse {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        let num_phrases = u32::read(buf)?;
        let mut phrases = Vec::new();
        for _ in 0..num_phrases {
            phrases.push(String::read(buf)?);
        }
        Ok(Self { num_phrases, phrases })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excluded_search_phrases_decode() {
        let raw: &[u8] = &[
            0x01, 0x00, 0x00, 0x00, // num_phrases=1
            0x04, 0x00, 0x00, 0x00, 0x62, 0x61, 0x64, 0x73, // "bads"
        ];
        let mut buf = bytes::Bytes::from_static(raw);
        let resp = ExcludedSearchPhrasesResponse::read(&mut buf).unwrap();
        assert_eq!(resp.num_phrases, 1);
        assert_eq!(resp.phrases[0], "bads");
    }
}
