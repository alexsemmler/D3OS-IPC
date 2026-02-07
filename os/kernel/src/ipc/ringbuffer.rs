use alloc::vec::Vec;

pub struct RingBuffer<T> {
    buffer: Vec<Option<T>>,
    capacity: usize,
    head: usize,
    tail: usize,
    full: bool,
}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Capacity must be greater than zero.");
        RingBuffer {
            buffer: (0..capacity).map(|_| None).collect(),
            capacity,
            head: 0,
            tail: 0,
            full: false,
        }
    }

    pub fn peek(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            self.buffer[self.head].as_ref()
        }
    }

    pub fn is_empty(&self) -> bool {
        !self.full && (self.head == self.tail)
    }

    pub fn is_full(&self) -> bool {
        self.full
    }

    pub fn push(&mut self, item: T) -> Result<(), T> {
        if self.full {
            return Err(item);
        }

        self.buffer[self.tail] = Some(item);
        self.tail = (self.tail + 1) % self.capacity;
        self.full = self.tail == self.head;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let item = self.buffer[self.head].take();
        self.head = (self.head + 1) % self.capacity;
        self.full = false;
        item
    }

    pub fn len(&self) -> usize {
        if self.full {
            self.capacity
        } else if self.tail >= self.head {
            self.tail - self.head
        } else {
            self.capacity - self.head + self.tail
        }

    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

