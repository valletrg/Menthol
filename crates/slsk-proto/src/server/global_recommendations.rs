use crate::codec::SlskRead;
use crate::error::ProtoError;
use bytes::Buf;

pub const CODE: u32 = 56;

// GlobalRecommendations is receive-only
#[derive(Debug, Clone)]
pub struct GlobalRecommendationsResponse {
    pub num_recommendations: u32,
    pub recommendations: Vec<(String, i32)>,
    pub num_unrecommendations: u32,
    pub unrecommendations: Vec<(String, i32)>,
}

impl SlskRead for GlobalRecommendationsResponse {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        let num_recommendations = u32::read(buf)?;
        let mut recommendations = Vec::new();
        for _ in 0..num_recommendations {
            recommendations.push((String::read(buf)?, i32::read(buf)?));
        }
        let num_unrecommendations = u32::read(buf)?;
        let mut unrecommendations = Vec::new();
        for _ in 0..num_unrecommendations {
            unrecommendations.push((String::read(buf)?, i32::read(buf)?));
        }
        Ok(Self {
            num_recommendations,
            recommendations,
            num_unrecommendations,
            unrecommendations,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_recommendations_decode() {
        // empty
        let raw: &[u8] = &[
            0x00, 0x00, 0x00, 0x00, // 0 recs
            0x00, 0x00, 0x00, 0x00, // 0 unrecs
        ];
        let mut buf = bytes::Bytes::from_static(raw);
        let resp = GlobalRecommendationsResponse::read(&mut buf).unwrap();
        assert_eq!(resp.num_recommendations, 0);
    }
}
