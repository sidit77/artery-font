use std::io::Read;
use zerocopy::{AsBytes, FromBytes};
use crate::crc32::Hasher;

type UtfResult<T> = std::result::Result<T, std::string::FromUtf8Error>;
type IoResult<T> = std::io::Result<T>;

pub struct ReadWrapper<R> {
    inner: R,
    total_length: usize,
    checksum: Hasher
}

impl<R: Read> ReadWrapper<R> {

    pub fn new(inner: R) -> Self {
        Self {
            inner,
            total_length: 0,
            checksum: Hasher::new()
        }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> IoResult<()> {
        self.inner.read_exact(buf)?;
        self.total_length += buf.len();
        self.checksum.update(buf);
        Ok(())
    }

    pub fn read_struct<S: AsBytes + FromBytes>(&mut self) -> IoResult<S> {
        let mut result = S::new_zeroed();
        self.read(result.as_bytes_mut())?;
        Ok(result)
    }

    pub fn read_struct_array<S: AsBytes + FromBytes + Clone>(&mut self, len: usize) -> IoResult<Vec<S>> {
        let mut vec = vec![S::new_zeroed(); len];
        self.read(vec.as_bytes_mut())?;
        Ok(vec)
    }

    pub fn realign(&mut self) -> IoResult<()> {
        let mut dump = [0u8; 4];
        if self.total_length & 0x03 != 0 {
            let len = 0x04 - (self.total_length & 0x03);
            self.read(&mut dump[..len])?;
        }
        Ok(())
    }

    pub fn read_string(&mut self, len: usize) -> IoResult<UtfResult<String>> {
        if len > 0 {
            let mut buf = vec![0u8; len + 1];
            self.read(buf.as_mut_slice())?;
            self.realign()?;
            buf.pop();
            Ok(String::from_utf8(buf))
        } else {
            Ok(Ok(String::new()))
        }
    }

    pub fn checksum(&self) -> u32 {
        self.checksum.clone().finalize()
    }

    pub fn bytes_read(&self) -> usize {
        self.total_length
    }

}