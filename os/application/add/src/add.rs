#![no_std]

extern crate alloc;

use alloc::string::String;
#[allow(unused_imports)]
use runtime::*;
use terminal::{print, println};
use alloc::vec::Vec;
use ipc::calc::{CalculationRequest, CalculatorServiceClient};
use ipc::api;

fn print_usage() {
    println!("usage: add [operand1] [operand2] ");
}

pub fn args_to_vec() -> Vec<String> {
    let args = env::args();
    let mut vec = Vec::new();
    for arg in args {
        vec.push(arg);
    }
    vec
}

#[unsafe(no_mangle)]
pub fn main() {
    let args_vec = args_to_vec();
    let args_count = args_vec.len();

    println!("client started");
    let client_ep_name = "calc_client";
    api::register(client_ep_name);

    if args_count == 3 {
        let op1 = match args_vec[1].parse::<f32>() {
            Ok(num) => num,
            Err(e) => {
                println!("Failed to parse operand1: {}", e);
                return
            },
        };
        let op2 = match args_vec[2].parse::<f32>() {
            Ok(num) => num,
            Err(e) => {
                println!("Failed to parse operand2: {}", e);
                return
            },
        };
        let req = CalculationRequest{
            a: op1,
            b: op2,
            source: String::from(client_ep_name),
        };

        let client = CalculatorServiceClient{
            source: client_ep_name,
        };

        let resp = client.add(&req);
        let res  = resp.expect("failed to get result!").result;
        println!("client got response: {:?}", res);

    } else {
        print_usage();
    }
}