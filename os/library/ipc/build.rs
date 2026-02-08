use std::env;
use std::path::PathBuf;
use quote::{format_ident, quote};

struct IpcServiceGenerator;

impl prost_build::ServiceGenerator for IpcServiceGenerator {
    fn generate(&mut self, service: prost_build::Service, buf: &mut String) {

        let client_struct_ident = format_ident!("{}Client", service.name);
        // "calculator" -> "calculator" (The string name used for lookup)
        let server_ep_name_str = service.name.to_lowercase(); 
        
        let mut method_fns = Vec::new();

        for method in service.methods {
            let method_ident = format_ident!("{}", method.name);
            let input_type = format_ident!("{}", method.input_type);
            let output_type = format_ident!("{}", method.output_type);
            
            // Generate no_std compatible method code
            let method_code = quote! {
                pub fn #method_ident(&self, req: &#input_type) -> core::result::Result<#output_type, i32> {

                    // 1. Serialize Request
                    let len = prost::Message::encoded_len(req);
                    let mut payload = alloc::vec::Vec::with_capacity(len);
                    
                    if prost::Message::encode(req, &mut payload).is_err() {
                        return core::result::Result::Err(-1); 
                    }

                    // 2. Syscall Send
                    // Use the stored server_handle (usize) instead of string
                    api::send(
                        self.server_handle,
                        payload.as_mut_ptr(),
                        payload.len()
                    );

                    // 3. Receive Reply
                    let mut resp_len = 1024;
                    let mut resp_buf = alloc::vec![0u8; resp_len];
                    let mut actual_len = 0;

                    loop {
                        // Receive from OUR OWN handle (client_handle)
                        match api::receive(self.client_handle, resp_buf.as_mut_ptr(), resp_len) {
                            Ok(len) => {
                                actual_len = len;
                                break;
                            },
                            Err(Errno::ERRCV) => {
                                // Buffer too small, resize
                                resp_len = resp_len * 2;
                                resp_buf = alloc::vec![0u8; resp_len];
                            },
                            Err(_) => {
                                continue; 
                            }
                        }
                    }

                    unsafe {
                        resp_buf.set_len(actual_len as usize);
                    }

                    // 4. Deserialize
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

            // The hardcoded service name we expect to find in the registry
            pub const SERVER_EP_NAME: &'static str = #server_ep_name_str;

            /// Generated IPC Client for #service_name
            /// Stores numeric handles for O(1) kernel access.
            pub struct #client_struct_ident {
                server_handle: usize,
                client_handle: usize,
            }

            impl #client_struct_ident {
                /// Initializes the client.
                /// 1. Registers 'source_name' to get a client_handle (for replies).
                /// 2. Looks up 'SERVER_EP_NAME' to get the server_handle (for requests).
                pub fn new(source_name: &str) -> Result<Self, Errno> {
                    
                    // Register ourselves so we can receive replies
                    let client_handle = api::register(source_name)?;
                    
                    // Find the server we want to talk to
                    let server_handle = api::lookup(SERVER_EP_NAME)?;
                    
                    Ok(Self {
                        server_handle,
                        client_handle,
                    })
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
    // Ensure your proto path is correct here
    config.compile_protos(&["proto/calculator.proto"], &["proto/"]).unwrap();

    built::write_built_file().expect("Failed to acquire build-time information");
}