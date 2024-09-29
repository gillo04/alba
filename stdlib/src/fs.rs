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
                "int 0x80",
                in("rax") 0x30,
                in("rcx") path.as_ptr(),
                in("rdx") path.len(),
                out("r8") file.ptr,
                out("r9") file.size
            );
        }

        if file.ptr == 0 {
            return Err("Error loading file");
        }
        Ok(file)
    }
}
