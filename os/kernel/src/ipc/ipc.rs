use alloc::{string::{String, ToString}, vec::Vec, vec};
use log::info;
use spin::{mutex::Mutex, Once, RwLock};
use smallmap::Map;
use syscall::return_vals::Errno;
use core::ptr;
use super::ringbuffer::RingBuffer;

static EP_VEC:  Once<RwLock<Vec<Endpoint>>> = Once::new();
const RB_CAPACITY: usize = 16; // capacity of the ringbuffer

pub struct Endpoint {
    pub ep_id: String,
    pub queue: Mutex<RingBuffer<Vec<u8>>>,
}

impl Endpoint {
    pub fn add_msg(&self, msg: Vec<u8>) -> Result<usize, Errno>{
        let mut rb = self.queue.lock();
        if rb.push(msg).is_ok() {
            Ok(0)
        } else {
            info!("Tried to add a message to a full RingBuffer");
            Err(Errno::EUNKN)
        }
        
    }
    
    pub fn get_msg(&self, buff_ptr: *mut u8, recv_len: usize) -> Result<usize, Errno>{
        let mut rb = self.queue.lock();

        let msg_len = match rb.peek() {
            Some(m) => m.len(),
            None => return Err(Errno::ERBEMPTY),
        };

        if msg_len > recv_len {
            info!("Buffer too small to receive message!");
            return Err(Errno::ERRCV); // Buffer too small
        }

        let msg = rb.pop().expect("Queue modified unexpectedly");

        unsafe {
            ptr::copy_nonoverlapping(
                msg.as_ptr(), 
                buff_ptr, 
                msg_len
            );
        }
        return Ok(msg_len);
        
    }
}

pub fn init() {
    info!("IPC service initialized");
    EP_VEC.call_once(|| RwLock::new(Vec::new()));
}

pub fn send_msg(ep_name: &str, msg_ptr: *mut u8, msg_len: usize ) -> Result<usize, Errno> {

    if let Some(ep_vec) = EP_VEC.get() {
        let locked_ep_vec = ep_vec.write();
        for ep in locked_ep_vec.iter() {
            //info!("Comparing {} with {}", ep.ep_id, ep_name);
            
            if ep.ep_id == ep_name {
                let mut msg = vec![0u8; msg_len];
                unsafe {
                    ptr::copy_nonoverlapping(
                        msg_ptr, 
                        msg.as_mut_ptr(),
                        msg_len,
                    );
                }
                return ep.add_msg(msg);
            }
        }
        info!("Tried to send a message to an Endpoint that doesn't exist!");
        Err(Errno::EUNKN)
    } else {
        info!("Tried to send a message when EP_VEC wasn't initialized!");
        Err(Errno::EUNKN)
    }
}

pub fn receive_msg(ep_name: &str, msg_ptr: *mut u8, recv_len: usize) -> Result<usize, Errno> {


    if let Some(ep_vec) = EP_VEC.get() {
        let locked_ep_vec = ep_vec.read();
        for ep in locked_ep_vec.iter() {
            if ep.ep_id == ep_name {
                return ep.get_msg(msg_ptr, recv_len);
            }
        }
        Err(Errno::EUNKN)
    } else {
        Err(Errno::EUNKN)
    }
}

pub fn register(name: &str) -> Result<usize, Errno> {
    if let Some(ep_vec) = EP_VEC.get() {
        let mut locked_ep_vec = ep_vec.write();
        for ep in locked_ep_vec.iter() {
            if ep.ep_id == name {
                return Ok(1);
            }
        }
        let ep = Endpoint{ep_id: name.to_string(), queue: Mutex::new(RingBuffer::new(RB_CAPACITY))};
        locked_ep_vec.push(ep);
        Ok(0)
    } else {
        panic!("Tried to register an IPC endpoint without initializing EP_VEC!");
    }
}

pub fn lookup(name: &str) -> Result<usize, Errno>{
    if let Some(ep_vec) = EP_VEC.get() {
        let locked_ep_vec = ep_vec.read();
        for ep in locked_ep_vec.iter() {
            if ep.ep_id == name {
                return Ok(1);
            }
        }
        Err(Errno::EUNKN)
    } else {
        panic!("Tried to lookup an IPC endpoint without initializing EP_VEC!");
    }
}
