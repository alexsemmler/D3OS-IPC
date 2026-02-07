use std::env;
use std::path::PathBuf;
use quote::{format_ident, quote};

struct IpcServiceGenerator;

impl prost_build::ServiceGenerator for IpcServiceGenerator {
    fn generate(&mut self, service: prost_build::Service, buf: &mut String) {

        let client_struct_ident = format_ident!("{}Client", service.name);
        let server_ep_name = format_ident!("{}", service.name.to_lowercase()).to_string();
        let mut method_fns = Vec::new();

        for method in service.methods {
            let method_ident = format_ident!("{}", method.name);
            let input_type = format_ident!("{}", method.input_type);
            let output_type = format_ident!("{}", method.output_type);
            
            // Generate no_std compatible method code
            let method_code = quote! {
                pub fn #method_ident(&self, req: &#input_type) -> core::result::Result<#output_type, i32> {

                    // Calculate length
                    let len = prost::Message::encoded_len(req);
                    
                    // Allocate buffer (requires extern crate alloc)
                    let mut payload = alloc::vec::Vec::with_capacity(len);
                    
                    // Serialize
                    if prost::Message::encode(req, &mut payload).is_err() {
                        return core::result::Result::Err(-1); 
                    }

                    // Syscall 
                    api::send(
                        #server_ep_name,
                        payload.as_mut_ptr(),
                        payload.len()
                    );

                    // Setup the buffer
                    let mut resp_len = 1024;
                    let mut resp_buf = alloc::vec![0u8; resp_len];

                    // Receive loop 
                    let mut actual_len = 0;
                    loop {
                        // We match the Result directly to get the length
                        match api::receive(self.source, resp_buf.as_mut_ptr(), resp_len) {
                            Ok(len) => {
                                actual_len = len; // Capture the length!
                                break; // Stop waiting
                            },
                            Err(Errno::ERRCV) => {
                                resp_len = resp_len * 2;
                                resp_buf = alloc::vec![0u8; resp_len];
                            },
                            Err(_) => {
                                // Ideally add a yield/sleep here to prevent CPU burning
                                continue; 
                            }
                        }
                    }

                    unsafe {
                        resp_buf.set_len(actual_len as usize);
                    }

                    // Deserialize
                    match <#output_type as prost::Message>::decode(resp_buf.as_slice()) {
                        core::result::Result::Ok(val) => core::result::Result::Ok(val),
                        core::result::Result::Err(_) => core::result::Result::Err(-1), 
                    }
                }
            };

            method_fns.push(method_code);
        }

        let client_code = quote! {

            use crate::api;
            use syscall::return_vals::Errno;

            pub const SERVER_EP_NAME: &'static str = #server_ep_name;

            /// Generated IPC Client for #service_name
            pub struct #client_struct_ident {
                pub source: &'static str,
            }

            impl #client_struct_ident {
                pub fn new(source: &'static str) -> Self {
                    api::register(source);
                    Self {source}
                }

                #(#method_fns)*
            }

        };

        buf.push_str(&client_code.to_string());
    }
}

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));

    let mut config = prost_build::Config::new();
    
    config.out_dir(out_dir);
    config.service_generator(Box::new(IpcServiceGenerator));
    config.btree_map(&["."]);
    config.compile_protos(&["proto/calculator.proto"], &["proto/"]).unwrap();

    built::write_built_file().expect("Failed to acquire build-time information");
}
