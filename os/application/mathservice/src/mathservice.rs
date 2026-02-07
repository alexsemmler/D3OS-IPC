#![no_std]


extern crate alloc;

use concurrent::thread;
#[allow(unused_imports)]
use runtime::*;
use alloc::vec;

#[unsafe(no_mangle)]
pub fn main() {

    thread::start_application("mathserver", vec![]);

}
