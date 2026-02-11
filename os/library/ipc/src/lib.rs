#![no_std]

extern crate alloc;

pub mod api;

pub mod ipc {
    include!(concat!(env!("OUT_DIR"), "/ipc.rs"));
}

pub mod calc{
    include!(concat!(env!("OUT_DIR"), "/ipc.calculator.rs"));
}