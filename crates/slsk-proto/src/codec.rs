use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::error::ProtoError;

pub trait SlskRead: Sized {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError>;
}

pub trait SlskWrite {
    fn write(&self, buf: &mut impl BufMut);
}

impl SlskRead for u8 {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        if !buf.has_remaining() {
            return Err(ProtoError::UnexpectedEof { needed: 1, have: 0 });
        }
        Ok(buf.get_u8())
    }
}

impl SlskWrite for u8 {
    fn write(&self, buf: &mut impl BufMut) {
        buf.put_u8(*self);
    }
}

impl SlskRead for u16 {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        if buf.remaining() < 2 {
            return Err(ProtoError::UnexpectedEof { needed: 2, have: buf.remaining() });
        }
        Ok(buf.get_u16_le())
    }
}

impl SlskWrite for u16 {
    fn write(&self, buf: &mut impl BufMut) {
        buf.put_u16_le(*self);
    }
}

impl SlskRead for u32 {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        if buf.remaining() < 4 {
            return Err(ProtoError::UnexpectedEof { needed: 4, have: buf.remaining() });
        }
        Ok(buf.get_u32_le())
    }
}

impl SlskWrite for u32 {
    fn write(&self, buf: &mut impl BufMut) {
        buf.put_u32_le(*self);
    }
}

impl SlskRead for i32 {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        if buf.remaining() < 4 {
            return Err(ProtoError::UnexpectedEof { needed: 4, have: buf.remaining() });
        }
        Ok(buf.get_i32_le())
    }
}

impl SlskWrite for i32 {
    fn write(&self, buf: &mut impl BufMut) {
        buf.put_i32_le(*self);
    }
}

impl SlskRead for u64 {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        if buf.remaining() < 8 {
            return Err(ProtoError::UnexpectedEof { needed: 8, have: buf.remaining() });
        }
        Ok(buf.get_u64_le())
    }
}

impl SlskWrite for u64 {
    fn write(&self, buf: &mut impl BufMut) {
        buf.put_u64_le(*self);
    }
}

impl SlskRead for bool {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        let b = u8::read(buf)?;
        Ok(b != 0)
    }
}

impl SlskWrite for bool {
    fn write(&self, buf: &mut impl BufMut) {
        buf.put_u8(if *self { 1 } else { 0 });
    }
}

impl SlskRead for String {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        let len = u32::read(buf)? as usize;
        if buf.remaining() < len {
            return Err(ProtoError::UnexpectedEof { needed: len, have: buf.remaining() });
        }
        let bytes = buf.copy_to_bytes(len);
        Ok(String::from_utf8(bytes.to_vec())?)
    }
}

impl SlskWrite for String {
    fn write(&self, buf: &mut impl BufMut) {
        let bytes = self.as_bytes();
        (bytes.len() as u32).write(buf);
        buf.put_slice(bytes);
    }
}

impl SlskRead for Bytes {
    fn read(buf: &mut impl Buf) -> Result<Self, ProtoError> {
        let len = u32::read(buf)? as usize;
        if buf.remaining() < len {
            return Err(ProtoError::UnexpectedEof { needed: len, have: buf.remaining() });
        }
        Ok(buf.copy_to_bytes(len))
    }
}

impl SlskWrite for Bytes {
    fn write(&self, buf: &mut impl BufMut) {
        (self.len() as u32).write(buf);
        buf.put_slice(self);
    }
}

/// wraps an encoded message payload with the length + code header
/// server_msg: true = u32 code, false = u8 code (peer init / distributed)
pub fn frame_message(code: u32, payload: &[u8], u8_code: bool) -> Bytes {
    let code_len = if u8_code { 1 } else { 4 };
    let total_len = code_len + payload.len();
    let mut buf = BytesMut::with_capacity(4 + total_len);
    buf.put_u32_le(total_len as u32);
    if u8_code {
        buf.put_u8(code as u8);
    } else {
        buf.put_u32_le(code);
    }
    buf.put_slice(payload);
    buf.freeze()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_bool() {
        let mut buf = BytesMut::new();
        true.write(&mut buf);
        false.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(bool::read(&mut buf).unwrap(), true);
        assert_eq!(bool::read(&mut buf).unwrap(), false);
    }

    #[test]
    fn round_trip_u32() {
        let mut buf = BytesMut::new();
        42u32.write(&mut buf);
        0u32.write(&mut buf);
        0xDEADBEEFu32.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(u32::read(&mut buf).unwrap(), 42);
        assert_eq!(u32::read(&mut buf).unwrap(), 0);
        assert_eq!(u32::read(&mut buf).unwrap(), 0xDEADBEEF);
    }

    #[test]
    fn round_trip_string() {
        let mut buf = BytesMut::new();
        "hello".to_string().write(&mut buf);
        "".to_string().write(&mut buf);
        "testuser".to_string().write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(String::read(&mut buf).unwrap(), "hello");
        assert_eq!(String::read(&mut buf).unwrap(), "");
        assert_eq!(String::read(&mut buf).unwrap(), "testuser");
    }
}
