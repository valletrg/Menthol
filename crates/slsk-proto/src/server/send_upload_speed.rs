use bytes::BufMut;
use crate::codec::SlskWrite;

pub const CODE: u32 = 121;

#[derive(Debug, Clone)]
pub struct SendUploadSpeedRequest {
    pub speed: u32,
}

impl SlskWrite for SendUploadSpeedRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.speed.write(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    use crate::codec::SlskRead;

    #[test]
    fn send_upload_speed_round_trip() {
        let req = SendUploadSpeedRequest { speed: 50000 };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(u32::read(&mut buf).unwrap(), 50000);
    }
}
