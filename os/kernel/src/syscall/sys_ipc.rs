use core::{ptr::slice_from_raw_parts, str::from_utf8};

use crate::ipc::ipc::{self};
use syscall::return_vals::{self, Errno};
use alloc::string::{String, ToString};


pub fn sys_register(name_ptr: *const u8, name_len: usize) -> isize {
    let ep_name = ptr_to_string(name_ptr, name_len);
    let syscall_result = ipc::register(&ep_name.unwrap()) ;
    return_vals::convert_syscall_result_to_ret_code(syscall_result)
}

pub fn sys_receive(handle: usize, msg_ptr: *mut u8, recv_len: usize) -> isize{
    return_vals::convert_syscall_result_to_ret_code(ipc::receive_msg(handle, msg_ptr, recv_len))
}

pub fn sys_send(handle: usize, msg_ptr:*mut u8, msg_len: usize ) -> isize {
    return return_vals::convert_syscall_result_to_ret_code(ipc::send_msg(handle, msg_ptr, msg_len));
}

pub fn sys_lookup(name_ptr: *const u8, name_len: usize) -> isize {
    let ep_name = ptr_to_string(name_ptr, name_len);
    let syscall_result = ipc::lookup(&ep_name.unwrap()) ;
    return_vals::convert_syscall_result_to_ret_code(syscall_result)
}


/// Convert a raw pointer resulting from a CString to a UTF-8 String
fn ptr_to_string(ptr: *const u8, len: usize) -> Result<String, Errno> {
    if ptr.is_null() {
        return Err(Errno::ENOTDIR);
    }
    let path = from_utf8(unsafe { slice_from_raw_parts(ptr, len).as_ref().unwrap() });
    match path {
        Ok(path_str) => Ok(path_str.to_string()),
        Err(_) => Err(Errno::EBADSTR),
    }   
}