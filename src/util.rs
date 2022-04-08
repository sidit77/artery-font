use std::io::Read;
use bytemuck::{Zeroable, Pod};

type UtfResult<T> = std::result::Result<T, std::string::FromUtf8Error>;
type IoResult<T> = std::io::Result<T>;

pub struct ReadWrapper<R> {
    inner: R,
    total_length: usize,
    #[cfg(not(feature = "no-checksum"))]
    checksum: crate::crc32::Hasher
}

impl<R: Read> Read for ReadWrapper<R> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        let len = self.inner.read(buf)?;
        self.total_length += len;
        #[cfg(not(feature = "no-checksum"))]
        self.checksum.update(&buf[..len]);
        Ok(len)
    }
}

impl<R: Read> ReadWrapper<R> {

    pub fn new(inner: R) -> Self {
        Self {
            inner,
            total_length: 0,
            #[cfg(not(feature = "no-checksum"))]
            checksum: crate::crc32::Hasher::new()
        }
    }

    pub fn read_struct<S: Zeroable + Pod>(&mut self) -> IoResult<S> {
        let mut result = S::zeroed();
        self.read_exact(bytemuck::bytes_of_mut(&mut result))?;
        Ok(result)
    }

    pub fn read_struct_array<S: Zeroable + Pod + Clone>(&mut self, len: usize) -> IoResult<Vec<S>> {
        let mut vec = vec![S::zeroed(); len];
        self.read_exact(bytemuck::cast_slice_mut(&mut vec[..]))?;
        Ok(vec)
    }

    pub fn realign(&mut self) -> IoResult<()> {
        let mut dump = [0u8; 4];
        if self.total_length & 0x03 != 0 {
            let len = 0x04 - (self.total_length & 0x03);
            self.read_exact(&mut dump[..len])?;
        }
        Ok(())
    }

    pub fn read_string(&mut self, len: usize) -> IoResult<UtfResult<String>> {
        if len > 0 {
            let mut buf = vec![0u8; len + 1];
            self.read_exact(buf.as_mut_slice())?;
            self.realign()?;
            buf.pop();
            Ok(String::from_utf8(buf))
        } else {
            Ok(Ok(String::new()))
        }
    }

    #[cfg(not(feature = "no-checksum"))]
    pub fn checksum(&self) -> u32 {
        self.checksum.clone().finalize()
    }

    pub fn bytes_read(&self) -> usize {
        self.total_length
    }

}
