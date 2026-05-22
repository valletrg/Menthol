use crate::codec::SlskWrite;
use bytes::BufMut;

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
    use crate::codec::SlskRead;
    use bytes::BytesMut;

    #[test]
    fn send_upload_speed_round_trip() {
        let req = SendUploadSpeedRequest { speed: 50000 };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(u32::read(&mut buf).unwrap(), 50000);
    }
}
