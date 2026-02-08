use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    vec::Vec,
    vec,
};
use log::info;
use spin::{mutex::Mutex, Once, RwLock};
use core::ptr;
use super::ringbuffer::RingBuffer;
use syscall::return_vals::Errno;

// --- Architecture Definitions ---

const RB_CAPACITY: usize = 16;

pub struct Endpoint {
    // The name is now mostly for debugging/reverse lookup, 
    // strictly speaking not needed for logic anymore.
    pub debug_name: String, 
    pub queue: Mutex<RingBuffer<Vec<u8>>>,
}

struct IpcState {
    /// Maps "d3os.service.audio" -> ID (e.g., 5)
    registry: BTreeMap<String, usize>,
    
    /// The actual storage. We use Option to allow slots to be freed (None).
    endpoints: Vec<Option<Endpoint>>,
    
    /// A stack of indices that were freed and can be reused.
    free_slots: Vec<usize>,
}

impl IpcState {
    fn new() -> Self {
        Self {
            registry: BTreeMap::new(),
            endpoints: Vec::new(),
            free_slots: Vec::new(),
        }
    }
}

// Single global lock for the IPC system state
static IPC: Once<RwLock<IpcState>> = Once::new();

// --- Endpoint Implementation ---

impl Endpoint {
    fn new(name: String) -> Self {
        Self {
            debug_name: name,
            queue: Mutex::new(RingBuffer::new(RB_CAPACITY)),
        }
    }

    pub fn add_msg(&self, msg: Vec<u8>) -> Result<usize, Errno> {
        let mut rb = self.queue.lock();
        if rb.push(msg).is_ok() {
            Ok(0)
        } else {
            info!("IPC RingBuffer full for {}", self.debug_name);
            Err(Errno::EUNKN) // Or Errno::EFULL
        }
    }

    pub fn get_msg(&self, buff_ptr: *mut u8, recv_len: usize) -> Result<usize, Errno> {
        let mut rb = self.queue.lock();

        // 1. Peek size safely
        let msg_len = match rb.peek() {
            Some(m) => m.len(),
            None => return Err(Errno::ERBEMPTY),
        };

        // 2. Validate buffer size
        if msg_len > recv_len {
            info!("Buffer too small ({} < {})", recv_len, msg_len);
            return Err(Errno::ERRCV);
        }

        // 3. Pop safely (handle concurrent modifications just in case)
        let msg = match rb.pop() {
            Some(m) => m,
            None => return Err(Errno::ERBEMPTY),
        };

        // 4. Copy
        unsafe {
            ptr::copy_nonoverlapping(msg.as_ptr(), buff_ptr, msg_len);
        }
        
        Ok(msg_len)
    }
}

// --- Public API ---

pub fn init() {
    info!("IPC service initialized");
    IPC.call_once(|| RwLock::new(IpcState::new()));
}

/// Registers a new endpoint name.
/// Returns the new HANDLE (usize) on success.
pub fn register(name: &str) -> Result<usize, Errno> {
    let ipc_lock = IPC.get().ok_or(Errno::EUNKN)?;
    let mut state = ipc_lock.write();

    // 1. Check if name already exists
    if let Some(&id) = state.registry.get(name) {
        // Option: return the existing ID, or error saying "Already Exists"
        return Ok(id); 
    }

    // 2. Create the endpoint
    let endpoint = Endpoint::new(name.to_string());

    // 3. Find a slot (recycle or new)
    let id = if let Some(free_id) = state.free_slots.pop() {
        state.endpoints[free_id] = Some(endpoint);
        free_id
    } else {
        let new_id = state.endpoints.len();
        state.endpoints.push(Some(endpoint));
        new_id
    };

    // 4. Update Registry
    state.registry.insert(name.to_string(), id);

    info!("Registered IPC Endpoint '{}' with ID {}", name, id);
    Ok(id)
}

/// Looks up a name and returns the Handle ID.
/// This is the "Discovery" phase.
pub fn lookup(name: &str) -> Result<usize, Errno> {
    let ipc_lock = IPC.get().ok_or(Errno::EUNKN)?;
    let state = ipc_lock.read();

    state.registry.get(name).cloned().ok_or(Errno::EUNKN)
}

/// Sends a message directly to an ID. 
/// This is O(1) and very fast.
pub fn send_msg(handle: usize, msg_ptr: *const u8, msg_len: usize) -> Result<usize, Errno> {
    let ipc_lock = IPC.get().ok_or(Errno::EUNKN)?;
    let state = ipc_lock.read();

    // 1. Validate Handle
    let endpoint = state.endpoints.get(handle)
        .and_then(|opt| opt.as_ref())
        .ok_or(Errno::EUNKN)?;

    // 2. Prepare Message
    let mut msg = vec![0u8; msg_len];
    unsafe {
        ptr::copy_nonoverlapping(msg_ptr, msg.as_mut_ptr(), msg_len);
    }

    // 3. Send
    endpoint.add_msg(msg)
}

/// Receives a message from a specific handle (usually the process's own handle).
pub fn receive_msg(handle: usize, buff_ptr: *mut u8, recv_len: usize) -> Result<usize, Errno> {
    let ipc_lock = IPC.get().ok_or(Errno::EUNKN)?;
    let state = ipc_lock.read();

    // 1. Validate Handle
    let endpoint = state.endpoints.get(handle)
        .and_then(|opt| opt.as_ref())
        .ok_or(Errno::EUNKN)?;

    // 2. Receive
    endpoint.get_msg(buff_ptr, recv_len)
}

/// Helper: Clean up an endpoint (e.g., when a process dies)
pub fn unregister(name: &str) -> Result<(), Errno> {
    let ipc_lock = IPC.get().ok_or(Errno::EUNKN)?;
    let mut state = ipc_lock.write();

    if let Some(id) = state.registry.remove(name) {
        if id < state.endpoints.len() {
            state.endpoints[id] = None; // Free the memory
            state.free_slots.push(id);  // Mark ID as reusable
        }
        Ok(())
    } else {
        Err(Errno::EUNKN)
    }
}