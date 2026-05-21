use bytes::BufMut;
use crate::codec::SlskWrite;

pub const CODE: u32 = 35;

#[derive(Debug, Clone)]
pub struct SharedFoldersFilesRequest {
    pub dirs:  u32,
    pub files: u32,
}

impl SlskWrite for SharedFoldersFilesRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.dirs.write(buf);
        self.files.write(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    use crate::codec::SlskRead;

    #[test]
    fn shared_folders_files_round_trip() {
        let req = SharedFoldersFilesRequest { dirs: 10, files: 500 };
        let mut buf = BytesMut::new();
        req.write(&mut buf);
        let mut buf = buf.freeze();
        assert_eq!(u32::read(&mut buf).unwrap(), 10);
        assert_eq!(u32::read(&mut buf).unwrap(), 500);
    }
}
