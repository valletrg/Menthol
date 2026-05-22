use crate::codec::SlskWrite;
use bytes::BufMut;

pub const CODE: u32 = 51;

// AddThingILike is send-only
#[derive(Debug, Clone)]
pub struct AddThingILikeRequest {
    pub item: String,
}

impl SlskWrite for AddThingILikeRequest {
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
    fn add_thing_i_like_round_trip() {
        let req = AddThingILikeRequest {
            item: "rock".into(),
        };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(String::read(&mut buf).unwrap(), "rock");
    }
}
