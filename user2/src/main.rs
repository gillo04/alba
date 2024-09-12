#![no_std]
#![no_main]

use stdlib::*;

#[export_name = "_start"]
#[no_mangle]
extern "C" fn main() {
    println!("\\gNumbers from 9 to 0:\\w");
    for i in (0..10).rev() {
        // println!("\\g\t{}\\w", i);
        println!("{}", i);
        //println!("\\gok");
    }
    loop {}
}
