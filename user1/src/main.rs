#![no_std]
#![no_main]

use alloc::string::*;
use stdlib::alloc::vec;
use stdlib::alloc::vec::*;
use stdlib::fs::*;
use stdlib::graphics::*;
use stdlib::ipc::*;
use stdlib::*;

#[export_name = "_start"]
#[no_mangle]
extern "C" fn main() {
    stdlib::heap::init().unwrap();

    create_mailbox(String::from("Hello"));
    println!("Created mailbox");

    let msg = String::from("ABC");
    send(String::from("Hello"), msg.as_bytes());
    println!("Sent message");

    let mut res = String::from("   ");
    unsafe {
        try_receive(String::from("Hello"), res.as_bytes_mut()).expect("Should be able to receive");
    }
    println!("Received message: {res}");

    delete_mailbox(String::from("Hello"));
    println!("Deleted mailbox");
    exit();
}
