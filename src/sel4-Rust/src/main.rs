#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(core_intrinsics)]

#[macro_use]
mod lang_items;
mod kernel;
mod types;
pub mod sbi;
pub mod console;
pub mod libsel4;
pub mod object;
pub mod failures;
pub mod model;
pub mod machine;

use core::arch::global_asm;
use sbi::shutdown;

global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    println!("Hello world!");
    shutdown()
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}