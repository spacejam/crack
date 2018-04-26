use std::fs;
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct File(fs::File);

impl Deref for File {
    type Target = fs::File;

    fn deref(&self) -> &fs::File {
        &self.inner
    }
}

impl DerefMut for File {
    fn deref_mut(&mut self) -> &mut fs::File {
        &mut self.inner
    }
}
