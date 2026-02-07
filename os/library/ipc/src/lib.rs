#![no_std]

extern crate alloc;

pub mod api;

pub mod calc{
    include!(concat!(env!("OUT_DIR"), "/ipc.calculator.rs"));
}