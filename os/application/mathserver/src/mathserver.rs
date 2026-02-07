#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use alloc::vec;
#[allow(unused_imports)]
use runtime::*;
use terminal::{print, println};
use ipc::calc::{ CalculationRequest, CalculationResponse, SERVER_EP_NAME};
use prost::{self, Message};
use ipc::api;


#[unsafe(no_mangle)]
pub fn main() {

    println!("server started");
    api::register(SERVER_EP_NAME);

    loop {

        let recv_len = core::mem::size_of::<CalculationRequest>();

        let mut recv_payload = vec![0u8; 23];
        while api::receive(SERVER_EP_NAME,  recv_payload.as_mut_ptr(), recv_len).is_err() {}


        let calc_req = match <CalculationRequest as prost::Message>::decode(recv_payload.as_slice()) {
            core::result::Result::Ok(val) => val,
            core::result::Result::Err(_) => {
                println!("server failed to decode message");
                continue;
            }
        };
        println!("server received message: {:?}", calc_req);

        println!("server processing message");
        // process message
        let result =  calc_req.a + calc_req.b;
        let calc_resp = CalculationResponse {
            result,
        };

        let resp_len = calc_resp.encoded_len();
        let mut buf = Vec::new();
        buf.reserve(resp_len);
        calc_resp.encode(&mut buf).unwrap();
        println!("server sending response: {:?}", calc_resp);
        // send back result
        api::send(&calc_req.source, buf.as_mut_ptr(), resp_len);
        println!("server finished");
        

    }
}