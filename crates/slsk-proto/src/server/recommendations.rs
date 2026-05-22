use crate::codec::SlskRead;
use crate::error::ProtoError;
use bytes::Buf;

pub const CODE: u32 = 54;

// Recommendations is receive-only
#[derive(Debug, Clone)]
pub struct RecommendationsResponse {
    pub num_recommendations: u32,
    pub recommendations: Vec<(String, i32)>,
    pub num_unrecommendations: u32,
    pub unrecommendations: Vec<(String, i32)>,
}

impl SlskRead for RecommendationsResponse {
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
    fn recommendations_decode() {
        // 1 recommendation: "jazz" +5, 0 unrecommendations
        let raw: &[u8] = &[
            0x01, 0x00, 0x00, 0x00, // num=1
            0x04, 0x00, 0x00, 0x00, 0x6a, 0x61, 0x7a, 0x7a, // "jazz"
            0x05, 0x00, 0x00, 0x00, // +5
            0x00, 0x00, 0x00, 0x00, // num_unrecommendations=0
        ];
        let mut buf = bytes::Bytes::from_static(raw);
        let resp = RecommendationsResponse::read(&mut buf).unwrap();
        assert_eq!(resp.num_recommendations, 1);
        assert_eq!(resp.recommendations[0].0, "jazz");
    }
}
