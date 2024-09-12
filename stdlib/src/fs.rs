#![allow(unused)]

use super::*;

pub struct File {
    pub ptr: u64,
    pub size: u64,
}

impl File {
    pub fn load(path: &str) -> Result<File, &str> {
        let mut file = File { ptr: 0, size: 0 };
        unsafe {
            asm!(
                "int 0x43",
                in("rax") path.as_ptr(),
                in("rcx") path.len(),
                out("rdx") file.ptr,
                out("r8") file.size
            );
        }

        if file.ptr == 0 {
            return Err("Error loading file");
        }
        Ok(file)
    }
}
