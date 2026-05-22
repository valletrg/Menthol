use crate::codec::SlskWrite;
use bytes::BufMut;

pub const CODE: u32 = 52;

// RemoveThingILike is send-only
#[derive(Debug, Clone)]
pub struct RemoveThingILikeRequest {
    pub item: String,
}

impl SlskWrite for RemoveThingILikeRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.item.write(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::SlskRead;
    use bytes::BytesMut;

    #[test]
    fn remove_thing_i_like_round_trip() {
        let req = RemoveThingILikeRequest {
            item: "rock".into(),
        };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(String::read(&mut buf).unwrap(), "rock");
    }
}
