use super::*;

pub fn create_mailbox(name: String) {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 0x42,
            in("rcx") name.as_ptr(),
            in("rdx") name.len(),
        );
    }
}

pub fn delete_mailbox(name: String) {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 0x43,
            in("rcx") name.as_ptr(),
            in("rdx") name.len(),
        );
    }
}

pub fn send(mailbox: String, data: &[u8]) {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 0x44,
            in("rcx") mailbox.as_ptr(),
            in("rdx") mailbox.len(),
            in("r8") data.as_ptr(),
            in("r9") data.len(),
        );
    }
}

pub fn try_receive(mailbox: String, data: &mut [u8]) -> Result<(), ()> {
    let mut res: u64;
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 0x45,
            in("rcx") mailbox.as_ptr(),
            in("rdx") mailbox.len(),
            in("r8") data.as_ptr(),
            out("r10") res,
        );
    }

    if res == 0 {
        return Ok(());
    } else {
        return Err(());
    }
}
