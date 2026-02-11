#![no_std]

extern crate alloc;

use alloc::string::String;
#[allow(unused_imports)]
use runtime::*;
use terminal::{print, println};
use ipc::calc::{Calculator, CalculatorServer, CalculationRequest, CalculationResponse};
struct MyMathServer;

impl Calculator for MyMathServer {
    
    fn add(&self, req: CalculationRequest) -> CalculationResponse {
        CalculationResponse {
            result: req.a + req.b,
        }
    }
}

#[unsafe(no_mangle)]
pub fn main() {
    println!("Starting Math Server...");
    let server = CalculatorServer::new(MyMathServer).unwrap();
    server.run(); 
}