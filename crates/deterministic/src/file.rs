use std::path::PathBuf;
use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom, Write};

use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct File {
    path: PathBuf,
    stable: Vec<u8>,
    updates: Vec<Vec<u8>>,
    cached: Vec<u8>,
    offset: usize,
    is_crashing: bool,
}

impl File {
    pub fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        unimplemented!()
    }
    pub fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize> {
        unimplemented!()
    }
    pub fn sync_all(&self) -> Result<()> {
        unimplemented!()
    }
    pub fn sync_data(&self) -> Result<()> {
        unimplemented!()
    }
    pub fn set_len(&self, size: u64) -> Result<()> {
        unimplemented!()
    }
}

impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        if self.is_crashing {
            return Err(Error::new(ErrorKind::BrokenPipe, "oh no!"));
        }
        self.offset = match pos {
            SeekFrom::Start(s) => s as usize,
            SeekFrom::End(e) => self.cached.len() - e as usize,
            SeekFrom::Current(c) => self.offset + c as usize,
        };

        if self.offset > self.cached.len() {
            unimplemented!("fill file with zeroes from past tip")
        }

        Ok(self.offset as u64)
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        unimplemented!()
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        unimplemented!()
    }
    fn flush(&mut self) -> Result<()> {
        unimplemented!()
    }
}

#[derive(Default, Debug)]
pub struct Filesystem {
    files: HashMap<PathBuf, File>,
}
