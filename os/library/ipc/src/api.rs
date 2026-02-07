/* ╔═════════════════════════════════════════════════════════════════════════╗
   ║ Module: ipc                                                         ║
   ╟─────────────────────────────────────────────────────────────────────────╢
   ║ Descr.: Syscalls for ipc functions.                                 ║
   ╟─────────────────────────────────────────────────────────────────────────╢
   ║ Author: Alexander Semmler                                           ║
   ╚═════════════════════════════════════════════════════════════════════════╝
*/
use syscall::{return_vals::{Errno}, syscall, SystemCall};


pub fn register(ep_name: &str) {
    syscall(SystemCall::Register, &[ep_name.as_ptr() as usize, ep_name.len()]).expect("Failed to register IPC Endpoint");
}

pub fn lookup(ep_name: &str) {
    syscall(SystemCall::Lookup, &[ep_name.as_ptr() as usize, ep_name.len()]).expect("Failed to register IPC Endpoint");
}

pub fn receive(ep_name: &str, msg_ptr: *mut u8, recv_len: usize) -> Result<usize, Errno>{
    syscall(SystemCall::Receive, &[ep_name.as_ptr() as usize, ep_name.len(), msg_ptr as usize as usize, recv_len])
}

pub fn send(ep_name: &str, msg_ptr: *mut u8, msg_len: usize) {
    syscall(SystemCall::Send, &[ep_name.as_ptr() as usize, ep_name.len(), msg_ptr as usize, msg_len]).expect("Failed to send to IPC Endpoint");
}

