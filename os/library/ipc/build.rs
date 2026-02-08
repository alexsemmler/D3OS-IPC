use std::env;
use std::path::PathBuf;
use quote::{format_ident, quote};

struct IpcServiceGenerator;

impl prost_build::ServiceGenerator for IpcServiceGenerator {
    fn generate(&mut self, service: prost_build::Service, buf: &mut String) {

        // --- 1. Common Names ---
        let service_name = &service.name; // e.g. "Calculator"
        let server_ep_name_str = service.name.to_lowercase(); // "calculator"
        
        // Identifier for the constant holding the service name
        let ep_const_ident = format_ident!("SERVER_EP_NAME");

        // --- 2. Client Generation (Existing Logic) ---
        let client_struct_ident = format_ident!("{}Client", service_name);
        
        let mut client_methods = Vec::new();
        for method in &service.methods {
            let method_ident = format_ident!("{}", method.name);
            let input_type = format_ident!("{}", method.input_type);
            let output_type = format_ident!("{}", method.output_type);
            
            let method_code = quote! {
                pub fn #method_ident(&self, req: &#input_type) -> core::result::Result<#output_type, i32> {
                    // 1. Serialize
                    let len = prost::Message::encoded_len(req);
                    let mut payload = alloc::vec::Vec::with_capacity(len);
                    if prost::Message::encode(req, &mut payload).is_err() {
                        return core::result::Result::Err(-1); 
                    }

                    // 2. Send (using stored handle)
                    api::send(self.server_handle, payload.as_mut_ptr(), payload.len());

                    // 3. Receive Reply (with resize loop)
                    let mut resp_len = 1024;
                    let mut resp_buf = alloc::vec![0u8; resp_len];
                    let mut actual_len = 0;

                    loop {
                        match api::receive(self.client_handle, resp_buf.as_mut_ptr(), resp_len) {
                            Ok(len) => {
                                actual_len = len;
                                break;
                            },
                            Err(Errno::ERRCV) => {
                                resp_len *= 2;
                                resp_buf = alloc::vec![0u8; resp_len];
                            },
                            Err(_) => continue, 
                        }
                    }

                    unsafe { resp_buf.set_len(actual_len); }

                    // 4. Deserialize
                    match <#output_type as prost::Message>::decode(resp_buf.as_slice()) {
                        Ok(val) => Ok(val),
                        Err(_) => Err(-1), 
                    }
                }
            };
            client_methods.push(method_code);
        }

        // --- 3. Server Generation (New Logic) ---
        
        let server_trait_ident = format_ident!("{}", service_name); // e.g. trait Calculator
        let server_struct_ident = format_ident!("{}Server", service_name); // struct CalculatorServer

        // We assume the FIRST method is the main handler for this IPC endpoint 
        // because we don't have method headers in the packet yet.
        let main_method = &service.methods[0];
        let main_input = format_ident!("{}", main_method.input_type);
        let main_output = format_ident!("{}", main_method.output_type);
        let main_fn_name = format_ident!("{}", main_method.name);

        // Generate the Trait Definition
        // The user must return the Response AND the destination string (client source name)
        let trait_def = quote! {
            pub trait #server_trait_ident {
                fn #main_fn_name(&self, req: #main_input) -> (#main_output, alloc::string::String);
            }
        };

        // Generate the Server Struct and Run Loop
        let server_impl = quote! {
            /// Generated Server Wrapper
            pub struct #server_struct_ident<T: #server_trait_ident> {
                inner: T,
                server_handle: usize,
            }

            impl<T: #server_trait_ident> #server_struct_ident<T> {
                
                /// Registers the server endpoint and prepares the struct
                pub fn new(inner: T) -> Result<Self, Errno> {
                    let server_handle = api::register(#ep_const_ident)?;
                    Ok(Self {
                        inner,
                        server_handle,
                    })
                }

                /// Runs the main server loop.
                /// This handles resizing buffers, decoding, processing (via your trait), and replying.
                pub fn run(&self) -> ! {
                    loop {
                        // 1. Receive Loop
                        let mut recv_len = 1024;
                        let mut recv_payload = alloc::vec![0u8; recv_len];
                        
                        let actual_len = loop {
                            match api::receive(self.server_handle, recv_payload.as_mut_ptr(), recv_len) {
                                Ok(len) => break len,
                                Err(Errno::ERRCV) => {
                                    recv_len *= 2;
                                    recv_payload = alloc::vec![0u8; recv_len];
                                },
                                Err(_) => continue,
                            }
                        };
                        unsafe { recv_payload.set_len(actual_len); }

                        // 2. Decode Request
                        let req = match <#main_input as prost::Message>::decode(recv_payload.as_slice()) {
                            Ok(v) => v,
                            Err(_) => continue, // Malformed message, ignore
                        };

                        // 3. Process via User Trait
                        // We expect the user to tell us where to reply (dest_name)
                        let (resp, dest_name) = self.inner.#main_fn_name(req);

                        // 4. Encode Response
                        let resp_len = prost::Message::encoded_len(&resp);
                        let mut resp_buf = alloc::vec::Vec::with_capacity(resp_len);
                        // We unwrap here because encoding a valid struct shouldn't fail
                        prost::Message::encode(&resp, &mut resp_buf).unwrap();

                        // 5. Lookup & Send Reply
                        if let Ok(client_handle) = api::lookup(&dest_name) {
                            api::send(client_handle, resp_buf.as_mut_ptr(), resp_len);
                        }
                    }
                }
            }
        };


        // --- 4. Final Output Construction ---
        let final_code = quote! {
            use crate::api;
            use syscall::return_vals::Errno;

            // The service name constant
            pub const #ep_const_ident: &'static str = #server_ep_name_str;

            // --- Client Side ---
            pub struct #client_struct_ident {
                server_handle: usize,
                client_handle: usize,
            }

            impl #client_struct_ident {
                pub fn new(source_name: &str) -> Result<Self, Errno> {
                    let client_handle = api::register(source_name)?;
                    let server_handle = api::lookup(#ep_const_ident)?;
                    Ok(Self { server_handle, client_handle })
                }
                #(#client_methods)*
            }

            // --- Server Side ---
            #trait_def
            #server_impl
        };

        buf.push_str(&final_code.to_string());
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