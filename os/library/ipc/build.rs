use std::env;
use std::path::PathBuf;
use quote::{format_ident, quote};

struct IpcServiceGenerator;

impl prost_build::ServiceGenerator for IpcServiceGenerator {
    fn generate(&mut self, service: prost_build::Service, buf: &mut String) {
        
        let service_name = &service.name; 
        let server_ep_name_str = service.name.to_lowercase();
        let client_struct_ident = format_ident!("{}Client", service_name);
        let server_trait_ident = format_ident!("{}", service_name);
        let server_struct_ident = format_ident!("{}Server", service_name);
        let ep_const_ident = format_ident!("SERVER_EP_NAME");

        let envelope_type = quote! { crate::ipc::IpcEnvelope };
        let header_type = quote! { crate::ipc::IpcHeader };

        // --- CLIENT GENERATION ---
        let mut client_methods = Vec::new();
        
        for method in &service.methods {
            let method_name = &method.name;
            let method_ident = format_ident!("{}", method_name);
            let input_type = format_ident!("{}", method.input_type);
            let output_type = format_ident!("{}", method.output_type);

            let method_code = quote! {
                pub fn #method_ident(&self, req: &#input_type) -> core::result::Result<#output_type, i32> {
                    
                    // Serialize Inner Request
                    let req_len = prost::Message::encoded_len(req);
                    let mut req_buf = alloc::vec::Vec::with_capacity(req_len);
                    prost::Message::encode(req, &mut req_buf).map_err(|_| -1)?;

                    // Build Header & Envelope (Generic!)
                    let header = #header_type {
                        source: self.source_name.clone(),
                        fn_name: #method_name.into(),
                        response_expected: true,
                        payload_len: req_len as u32,
                    };

                    let envelope = #envelope_type {
                        header: Some(header),
                        payload: req_buf,
                    };

                    // Serialize Envelope
                    let env_len = prost::Message::encoded_len(&envelope);
                    let mut env_buf = alloc::vec::Vec::with_capacity(env_len);
                    prost::Message::encode(&envelope, &mut env_buf).map_err(|_| -1)?;

                    // Send
                    api::send(self.server_handle, env_buf.as_mut_ptr(), env_len);

                    // Receive Reply
                    let mut resp_len = 1024;
                    let mut resp_buf = alloc::vec![0u8; resp_len];
                    let mut actual_len = 0;

                    loop {
                        match api::receive(self.client_handle, resp_buf.as_mut_ptr(), resp_len) {
                            Ok(len) => { actual_len = len; break; },
                            Err(Errno::ERRCV) => { resp_len *= 2; resp_buf = alloc::vec![0u8; resp_len]; },
                            Err(_) => continue, 
                        }
                    }
                    unsafe { resp_buf.set_len(actual_len); }
                    
                    match <#output_type as prost::Message>::decode(resp_buf.as_slice()) {
                        Ok(val) => Ok(val),
                        Err(_) => Err(-1), 
                    }
                }
            };
            client_methods.push(method_code);
        }

        // --- SERVER GENERATION ---
        
        let mut trait_methods = Vec::new();
        let mut dispatch_arms = Vec::new();

        for method in &service.methods {
            let method_name = &method.name;
            let method_ident = format_ident!("{}", method_name);
            let input_type = format_ident!("{}", method.input_type);
            let output_type = format_ident!("{}", method.output_type);

            trait_methods.push(quote! {
                fn #method_ident(&self, req: #input_type) -> #output_type;
            });

            dispatch_arms.push(quote! {
                #method_name => {
                    let req = match <#input_type as prost::Message>::decode(payload.as_slice()) {
                        Ok(r) => r,
                        Err(_) => {
                            continue; 
                        }
                    };
                    
                    let resp = self.inner.#method_ident(req);
                    
                    let resp_len = prost::Message::encoded_len(&resp);
                    let mut resp_buf = alloc::vec::Vec::with_capacity(resp_len);
                    prost::Message::encode(&resp, &mut resp_buf).unwrap();
                    
                    if let Ok(client_handle) = api::lookup(&header.source) {
                        api::send(client_handle, resp_buf.as_mut_ptr(), resp_len);
                    }
                }
            });
        }

        let server_code = quote! {
            pub trait #server_trait_ident {
                #(#trait_methods)*
            }

            pub struct #server_struct_ident<T: #server_trait_ident> {
                inner: T,
                server_handle: usize,
            }

            impl<T: #server_trait_ident> #server_struct_ident<T> {
                pub fn new(inner: T) -> Result<Self, Errno> {
                    let server_handle = api::register(#ep_const_ident)?;
                    Ok(Self { inner, server_handle })
                }

                pub fn run(&self) -> ! {
                    loop {
                        let mut buf_len = 1024;
                        let mut buf = alloc::vec![0u8; buf_len];
                        let actual_len = loop {
                            match api::receive(self.server_handle, buf.as_mut_ptr(), buf_len) {
                                Ok(len) => break len,
                                Err(Errno::ERRCV) => { buf_len *= 2; buf = alloc::vec![0u8; buf_len]; },
                                Err(_) => continue,
                            }
                        };
                        unsafe { buf.set_len(actual_len); }

                        let envelope = match #envelope_type::decode(buf.as_slice()) {
                            Ok(env) => env,
                            Err(_) => continue,
                        };

                        let header = match envelope.header {
                            Some(h) => h,
                            None => continue,
                        };
                        
                        let payload = envelope.payload;

                        match header.fn_name.as_str() {
                            #(#dispatch_arms)*
                            _ => {}
                        }
                    }
                }
            }
        };

        // Rest of code assembly
        
        let final_code = quote! {
            use crate::api;
            use prost::Message;
            use syscall::return_vals::Errno;

            pub const #ep_const_ident: &'static str = #server_ep_name_str;

            pub struct #client_struct_ident {
                server_handle: usize,
                client_handle: usize,
                source_name: alloc::string::String,
            }

            impl #client_struct_ident {
                pub fn new(source_name: &str) -> Result<Self, Errno> {
                    let client_handle = api::register(source_name)?;
                    let server_handle = api::lookup(#ep_const_ident)?;
                    Ok(Self { 
                        server_handle, 
                        client_handle, 
                        source_name: alloc::string::String::from(source_name) 
                    })
                }
                #(#client_methods)*
            }

            #server_code
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
    
    config.compile_protos(
        &["proto/ipc.proto", "proto/calculator.proto"], 
        &["proto/"]
    ).unwrap();

    built::write_built_file().expect("Failed to acquire build-time information");
}