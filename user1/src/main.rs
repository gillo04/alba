#![no_std]
#![no_main]

use stdlib::*;

#[export_name = "_start"]
#[no_mangle]
extern "C" fn main() {
    println!("\\rNumbers from 0 to 9:\\w");
    for i in 0..10 {
        // println!("\\r\t{}\\w", i);
        println!("{}", i);
        //println!("\\rok");
    }
    loop {}
}
