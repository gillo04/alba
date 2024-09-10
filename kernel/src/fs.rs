#![allow(unused)]

use crate::memory::*;

pub trait Fs {
    fn read_file(&self, path: &str) -> Result<VirtualMapping, &str>;
}
