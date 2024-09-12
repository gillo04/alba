#![allow(unused)]

use crate::memory::*;

pub trait Fs {
    fn read_file(&self, path: &str) -> Result<File, &str>;
}

pub struct File {
    pub mapping: VirtualMapping,
    pub size: u64,
}
