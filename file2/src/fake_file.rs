use std::io::{Result, SeekFrom};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(crate) struct Write {
    offset: usize,
    data: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct File {
    path: PathBuf,
    journal: Vec<Write>,
    sync: Vec<u8>,
    position: usize,
}

impl File {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<File> {
        let mut res = File::default();
        res.path = path.as_ref().to_path_buf();
        Ok(res)
    }

    pub fn create<P: AsRef<Path>>(path: P) -> Result<File> {
        File::open(path)
    }

    pub fn sync_all(&self) -> Result<()> {}

    pub fn sync_data(&self) -> Result<()> {}

    pub fn set_len(&self, size: u64) -> Result<()> {}

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {}

    pub fn write(&mut self, buf: &[u8]) -> Result<usize> {}

    pub fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        match pos {
            SeekFrom::End(e) => {}
        }
    }
}
