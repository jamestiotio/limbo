use cfg_block::cfg_block;
use std::{mem::ManuallyDrop, pin::Pin, sync::Arc};

pub type Complete = dyn Fn(&Buffer) + Send + Sync;

pub struct Completion {
    pub buf: Buffer,
    pub complete: Box<Complete>,
}

impl Completion {
    pub fn new(buf: Buffer, complete: Box<Complete>) -> Self {
        Self { buf, complete }
    }

    pub fn complete(&self) {
        (self.complete)(&self.buf);
    }
}

pub type BufferData = Pin<Vec<u8>>;

pub type BufferDropFn = Arc<dyn Fn(BufferData)>;

pub struct Buffer {
    data: ManuallyDrop<BufferData>,
    drop: BufferDropFn,
}

impl Drop for Buffer {
    fn drop(&mut self) {
        let data = unsafe { ManuallyDrop::take(&mut self.data) };
        (self.drop)(data);
    }
}

impl Buffer {
    pub fn allocate(size: usize, drop: BufferDropFn) -> Self {
        let data = ManuallyDrop::new(Pin::new(vec![0; size]));
        Self { data, drop }
    }

    pub fn new(data: BufferData, drop: BufferDropFn) -> Self {
        let data = ManuallyDrop::new(data);
        Self { data, drop }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }
}

cfg_block! {
    #[cfg(target_os = "linux")] {
        mod linux;
        pub use linux::{File, IO};
    }

    #[cfg(target_os = "macos")] {
        mod darwin;
        pub use darwin::{File, IO};
    }
}
