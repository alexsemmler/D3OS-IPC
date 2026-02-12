#![no_std]

extern crate alloc;

#[allow(unused_imports)]
use runtime::*;
use terminal::{print, println};
use ipc::calc::{Calculator, CalculatorServer, CalculationRequest, CalculationResponse};

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

struct MyMathServer;

impl Calculator for MyMathServer {
    
    fn add(&self, req: CalculationRequest) -> CalculationResponse {
        println!("{}Server received request for: {} + {}{}", color::MAGENTA, req.a, req.b, color::RESET);
        let resp = CalculationResponse {result: req.a + req.b};
        println!("{}Server sending: {} as a response{}", color::MAGENTA, resp.result, color::RESET);
        resp
    }
}

#[unsafe(no_mangle)]
pub fn main() {
    println!("Starting Math Server...");
    let server = CalculatorServer::new(MyMathServer).unwrap();
    server.run(); 
}