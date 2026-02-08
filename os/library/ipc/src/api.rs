/* ╔═════════════════════════════════════════════════════════════════════════╗
   ║ Module: ipc                                                         ║
   ╟─────────────────────────────────────────────────────────────────────────╢
   ║ Descr.: Syscalls for ipc functions.                                 ║
   ╟─────────────────────────────────────────────────────────────────────────╢
   ║ Author: Alexander Semmler                                           ║
   ╚═════════════════════════════════════════════════════════════════════════╝
*/
use syscall::{return_vals::{Errno}, syscall, SystemCall};


pub fn register(ep_name: &str) -> Result<usize, Errno> {
    syscall(SystemCall::Register, &[ep_name.as_ptr() as usize, ep_name.len()])
}

pub fn lookup(ep_name: &str) -> Result<usize, Errno> {
    syscall(SystemCall::Lookup, &[ep_name.as_ptr() as usize, ep_name.len()])
}

pub fn receive(handle: usize, msg_ptr: *mut u8, recv_len: usize) -> Result<usize, Errno> {
    syscall(SystemCall::Receive, &[handle, msg_ptr as usize as usize, recv_len])
}

pub fn send(handle: usize, msg_ptr: *mut u8, msg_len: usize) {
    syscall(SystemCall::Send, &[handle, msg_ptr as usize, msg_len]).expect("Failed to send to IPC Endpoint");
}

