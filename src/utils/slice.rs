// Source: https://stackoverflow.com/questions/65178245/how-do-i-write-to-a-mutable-slice-from-multiple-threads-at-arbitrary-indexes-wit

use std::cell::UnsafeCell;

#[derive(Copy, Clone)]
pub struct UnsafeSlice<'a, T> {
    slice: &'a [UnsafeCell<T>],
}

unsafe impl<'a, T: Send + Sync> Send for UnsafeSlice<'a, T> {}
unsafe impl<'a, T: Send + Sync> Sync for UnsafeSlice<'a, T> {}

impl<'a, T> UnsafeSlice<'a, T> {
    pub fn new(slice: &'a mut [T]) -> Self {
        let ptr = slice as *mut [T] as *const [UnsafeCell<T>];
        Self {
            slice: unsafe { &*ptr },
        }
    }

    /// SAFETY: It is UB if two threads write to the same index without
    /// synchronization.
    pub unsafe fn write(&self, i: usize, value: T) {
        let ptr = self.slice[i].get();
        *ptr = value;
    }
}

impl<'a, T> UnsafeSlice<'a, T> where T: Copy {
    pub unsafe fn read(&self, i: usize) -> T {
        *self.slice[i].get()
    }
}
