use super::*;
use std::fs;
use std::io::Result;
use std::path::Path;

#[derive(Debug)]
pub struct OpenOptions {
    pub(crate) inner: fs::OpenOptions,
}

impl OpenOptions {
    pub fn new() -> OpenOptions {
        OpenOptions { inner: fs::OpenOptions::new() }
    }

    pub fn read(&mut self, read: bool) -> &mut OpenOptions {
        #[cfg(not(feature = "filecracker"))] self.inner.read(read);
        self
    }

    pub fn write(&mut self, write: bool) -> &mut OpenOptions {
        #[cfg(not(feature = "filecracker"))] self.inner.write(write);
        self
    }

    pub fn append(&mut self, append: bool) -> &mut OpenOptions {
        #[cfg(not(feature = "filecracker"))] self.inner.append(append);
        self
    }

    pub fn truncate(&mut self, truncate: bool) -> &mut OpenOptions {
        #[cfg(not(feature = "filecracker"))] self.inner.truncate(truncate);
        self
    }

    pub fn create(&mut self, create: bool) -> &mut OpenOptions {
        #[cfg(not(feature = "filecracker"))] self.inner.create(create);
        self
    }

    pub fn create_new(&mut self, create_new: bool) -> &mut OpenOptions {
        #[cfg(not(feature = "filecracker"))] self.inner.create_new(create_new);
        self
    }

    pub fn open<P: AsRef<Path>>(&self, path: P) -> Result<File> {
        #[cfg(not(feature = "filecracker"))]
        Ok(File {
            inner: self.inner.open(&path)?,
            path: path.as_ref().to_path_buf(),
        })
    }
}
