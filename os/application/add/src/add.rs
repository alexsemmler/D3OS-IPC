#![no_std]

extern crate alloc;

use alloc::string::String;
#[allow(unused_imports)]
use runtime::*;
use terminal::{println};
use alloc::vec::Vec;
use ipc::calc::{CalculationRequest, CalculatorServiceClient};

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
    let client_ep_name = "calc_client";

    println!("{}client started{}", color::GREEN, color::RESET);

    if args_count == 3 {
        let op1 = match args_vec[1].parse::<f32>() {
            Ok(num) => num,
            Err(e) => {
                println!("{}Failed to parse operand1: {}{}", color::GREEN, e, color::RESET);
                return
            },
        };
        let op2 = match args_vec[2].parse::<f32>() {
            Ok(num) => num,
            Err(e) => {
                println!("{}Failed to parse operand2: {}{}", color::GREEN, e, color::RESET);
                return
            },
        };

        let req = CalculationRequest{
            a: op1,
            b: op2,
            source: String::from(client_ep_name),
        };

        let client = CalculatorServiceClient::new(client_ep_name).expect("Failed to create new CalculatorServiceClient");
        println!("{}client sending request for {} + {}{}", color::GREEN, req.a, req.b, color::RESET);

        let resp = client.add(&req);
        
        let res  = resp.expect("failed to get result!").result;
        println!("{}client got response: {:?}{}", color::GREEN, res, color::RESET);

    } else {
        print_usage();
    }
}