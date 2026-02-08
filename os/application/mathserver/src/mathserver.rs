#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use alloc::vec;
#[allow(unused_imports)]
use runtime::*;
use terminal::{print, println};
use ipc::calc::{CalculationRequest, CalculationResponse, SERVER_EP_NAME};
use prost::{self, Message};
use ipc::api;
use syscall::return_vals::Errno;

pub mod color {
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const MAGENTA: &str = "\x1b[35m";
    pub const CYAN: &str = "\x1b[36m";
    pub const WHITE: &str = "\x1b[37m";
    pub const RESET: &str = "\x1b[0m";
}

#[unsafe(no_mangle)]
pub fn main() {
    println!("{}server started{}", color::MAGENTA, color::RESET);

    // Register and get the SERVER HANDLE.
    let server_handle = api::register(SERVER_EP_NAME)
        .expect("Failed to register server endpoint");

    loop {
        // Protobufs are variable length. We start with a reasonable buffer (1KB).
        let mut recv_len = 1024;
        let mut recv_payload = vec![0u8; recv_len];

        // Robust Receive Loop
        // We pass 'server_handle' (usize), not the string name.
        let actual_len = loop {
            match api::receive(server_handle, recv_payload.as_mut_ptr(), recv_len) {
                Ok(len) => break len, // Success!
                Err(Errno::ERRCV) => {
                    // Buffer was too small. Resize and try again.
                    println!("{}Buffer too small, resizing...{}", color::MAGENTA, color::RESET);
                    recv_len *= 2;
                    recv_payload = vec![0u8; recv_len];
                }
                Err(_) => {
                    // Yield in the future
                    continue;
                }
            }
        };

        // Update vector length so decoding works correctly
        unsafe { recv_payload.set_len(actual_len); }

        // Decode
        let calc_req = match CalculationRequest::decode(recv_payload.as_slice()) {
            Ok(val) => val,
            Err(_) => {
                println!("{}server failed to decode message{}", color::MAGENTA, color::RESET);
                continue;
            }
        };
        println!("{}server received message: {:?}{}", color::MAGENTA, calc_req, color::RESET);
        println!("{}server processing message{}", color::MAGENTA, color::RESET);

        // Process
        let result = calc_req.a + calc_req.b;
        let calc_resp = CalculationResponse {
            result,
        };

        // Encode Response
        let resp_len = calc_resp.encoded_len();
        let mut buf = Vec::with_capacity(resp_len);
        calc_resp.encode(&mut buf).unwrap();
        println!("{}server sending response: {:?}{}", color::MAGENTA, calc_resp, color::RESET);

        // Send Response (The Bridge)
        // Your Proto still has 'source' as a String, but send() needs a Handle.
        // We must LOOKUP the string to find the handle.
        match api::lookup(&calc_req.source) {
            Ok(client_handle) => {
                api::send(client_handle, buf.as_mut_ptr(), resp_len);
            },
            Err(_) => {
                println!("{}Error: Could not find client '{}' to reply to.{}", color::MAGENTA, calc_req.source, color::RESET);
            }
        }
        
    }
}