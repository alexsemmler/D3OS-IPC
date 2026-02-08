#![no_std]

extern crate alloc;

use alloc::string::String;
#[allow(unused_imports)]
use runtime::*;
use terminal::{print, println};
use ipc::calc::{Calculator, CalculatorServer, CalculationRequest, CalculationResponse};
struct MyMathServer;

// Implement the generated Trait
impl Calculator for MyMathServer {
    fn add(&self, req: CalculationRequest) -> (CalculationResponse, String) {
        
        // 1. Do the logic
        let result = req.a + req.b;
        
        // 2. Return result AND the destination (source) to reply to
        (
            CalculationResponse { result }, 
            req.source // You extracted this from the message
        )
    }
}

#[unsafe(no_mangle)]
pub fn main() {
    println!("Server started");
    
    // Create the server wrapper
    let server = CalculatorServer::new(MyMathServer).expect("Failed to register");
    
    // Run the loop (this never returns)
    server.run(); 
}