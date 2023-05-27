use core::ptr;
use core::alloc::AllocError;
use core::ops::{Drop, Deref};

const STRING_HEAP_SIZE: usize = 2048;
static mut STRING_HEAP: [u8; STRING_HEAP_SIZE] = [0_u8; STRING_HEAP_SIZE];


struct StringAllocator;

impl StringAllocator {
    unsafe fn alloc(size: usize) -> Result<ptr::NonNull<[u8]>, AllocError> {
        let true_size = size+2;

        todo!()
    }

    unsafe fn free(ptr: ptr::NonNull<[u8]>) {
        let u8_ptr = ptr.as_ptr() as *mut u8;
        let size = u16::from_ne_bytes(*(u8_ptr.sub(2) as *mut [u8; 2]));
        u8_ptr.sub(2).write_bytes(0, size as usize + 2)
    }
}

pub struct String {
    size: usize,
    len: usize,
    bytes: Option<ptr::NonNull<[u8]>>
}

impl String {
    pub fn new() -> Self {
        Self {
            size: 0,
            len: 0,
            bytes: None
        }
    }

    pub fn new_with_capacity(size: usize) -> Result<Self, AllocError> {
        let bytes = Some(unsafe {
            StringAllocator::alloc(size)?
        });
        Ok(Self {
            size,
            len: 0,
            bytes,
        })
    }
}

impl Drop for String {
    fn drop(&mut self) {
        if let Some(p) = self.bytes {
            unsafe {
                StringAllocator::free(p)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_allocate_string() {
        let s = String::new_with_capacity(32);
    }

}
