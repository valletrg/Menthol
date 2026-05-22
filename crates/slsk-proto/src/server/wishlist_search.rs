use crate::codec::SlskWrite;
use bytes::BufMut;

pub const CODE: u32 = 103;

#[derive(Debug, Clone)]
pub struct WishlistSearchRequest {
    pub token: u32,
    pub query: String,
}

impl SlskWrite for WishlistSearchRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.token.write(buf);
        self.query.write(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::SlskRead;
    use bytes::BytesMut;

    #[test]
    fn wishlist_search_round_trip() {
        let req = WishlistSearchRequest {
            token: 5,
            query: "rare track".into(),
        };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(u32::read(&mut buf).unwrap(), 5);
        assert_eq!(String::read(&mut buf).unwrap(), "rare track");
    }
}
