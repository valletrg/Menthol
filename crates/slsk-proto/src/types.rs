use crate::codec::{SlskRead, SlskWrite};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ConnectionType {
    PeerToPeer  = b'P',
    FileTransfer = b'F',
    Distributed  = b'D',
}

impl TryFrom<u8> for ConnectionType {
    type Error = crate::error::ProtoError;
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            b'P' => Ok(Self::PeerToPeer),
            b'F' => Ok(Self::FileTransfer),
            b'D' => Ok(Self::Distributed),
            _ => Err(crate::error::ProtoError::InvalidEnumValue {
                value: v as u32,
                type_name: "ConnectionType",
            }),
        }
    }
}

impl SlskWrite for ConnectionType {
    fn write(&self, buf: &mut impl bytes::BufMut) {
        buf.put_u8(*self as u8);
    }
}

impl SlskRead for ConnectionType {
    fn read(buf: &mut impl bytes::Buf) -> Result<Self, crate::error::ProtoError> {
        Self::try_from(u8::read(buf)?)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum UserStatus {
    Offline = 0,
    Away    = 1,
    Online  = 2,
}

impl TryFrom<u32> for UserStatus {
    type Error = crate::error::ProtoError;
    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Self::Offline),
            1 => Ok(Self::Away),
            2 => Ok(Self::Online),
            _ => Err(crate::error::ProtoError::InvalidEnumValue {
                value: v,
                type_name: "UserStatus",
            }),
        }
    }
}

impl SlskWrite for UserStatus {
    fn write(&self, buf: &mut impl bytes::BufMut) {
        (*self as u32).write(buf);
    }
}

impl SlskRead for UserStatus {
    fn read(buf: &mut impl bytes::Buf) -> Result<Self, crate::error::ProtoError> {
        Self::try_from(u32::read(buf)?)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TransferDirection {
    Download = 0,
    Upload   = 1,
}

impl TryFrom<u32> for TransferDirection {
    type Error = crate::error::ProtoError;
    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Self::Download),
            1 => Ok(Self::Upload),
            _ => Err(crate::error::ProtoError::InvalidEnumValue {
                value: v,
                type_name: "TransferDirection",
            }),
        }
    }
}

impl SlskWrite for TransferDirection {
    fn write(&self, buf: &mut impl bytes::BufMut) {
        (*self as u32).write(buf);
    }
}

impl SlskRead for TransferDirection {
    fn read(buf: &mut impl bytes::Buf) -> Result<Self, crate::error::ProtoError> {
        Self::try_from(u32::read(buf)?)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum FileAttributeType {
    Bitrate    = 0,
    Duration   = 1,
    Vbr        = 2,
    SampleRate = 4,
    BitDepth   = 5,
}

impl TryFrom<u32> for FileAttributeType {
    type Error = crate::error::ProtoError;
    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Self::Bitrate),
            1 => Ok(Self::Duration),
            2 => Ok(Self::Vbr),
            4 => Ok(Self::SampleRate),
            5 => Ok(Self::BitDepth),
            _ => Err(crate::error::ProtoError::InvalidEnumValue {
                value: v,
                type_name: "FileAttributeType",
            }),
        }
    }
}

impl SlskWrite for FileAttributeType {
    fn write(&self, buf: &mut impl bytes::BufMut) {
        (*self as u32).write(buf);
    }
}

impl SlskRead for FileAttributeType {
    fn read(buf: &mut impl bytes::Buf) -> Result<Self, crate::error::ProtoError> {
        Self::try_from(u32::read(buf)?)
    }
}
