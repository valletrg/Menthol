use crate::codec::SlskWrite;
use bytes::BufMut;

pub const CODE: u32 = 123;

// GivePrivileges is send-only
#[derive(Debug, Clone)]
pub struct GivePrivilegesRequest {
    pub username: String,
    pub days: u32,
}

impl SlskWrite for GivePrivilegesRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.username.write(buf);
        self.days.write(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::SlskRead;
    use bytes::BytesMut;

    #[test]
    fn give_privileges_round_trip() {
        let req = GivePrivilegesRequest {
            username: "alice".into(),
            days: 30,
        };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(String::read(&mut buf).unwrap(), "alice");
        assert_eq!(u32::read(&mut buf).unwrap(), 30);
    }
}
