extern crate file2;

use std::fs;
use std::io::{Read, Write, Seek, SeekFrom};

fn runner(f: &mut fs::File) {
    let buf = b"awtfwafta";
    f.write(buf).unwrap();
    f.metadata().unwrap();
}

fn invariant(f: &mut fs::File) {
    f.seek(SeekFrom::Start(0)).unwrap();

    let mut read_buf = vec![];
    f.read_to_end(&mut read_buf).unwrap();
    assert_eq!(&*read_buf, buf);
}

#[test]
fn yoooo() {
    let path = "some.file";
    // let mut f = file2::File::create(path).unwrap();
    let mut f: file2::File = file2::OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(path)
        .unwrap();

    runner(&mut f);

    f.mess();

    invariant(&mut f);
}
