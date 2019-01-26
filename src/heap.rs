use crate::{PodRegion, PodRegionMut, ZeroBits};

use std::alloc::{Alloc, Global};
use std::mem::{align_of, size_of};
use std::ptr::{NonNull, null_mut, write_bytes};
use std::slice::{from_raw_parts, from_raw_parts_mut};

pub struct HeapMem<T: Copy> {
  buf:  *mut T,
  len:  usize,
}

impl<T> Drop for HeapMem<T> where T: Copy {
  fn drop(&mut self) {
    assert!(!self.buf.is_null());
    let p = unsafe { NonNull::new_unchecked(self.buf) };
    self.buf = null_mut();
    match unsafe { Global::default().dealloc_array(p, self.len) } {
      Err(_) => panic!("failed to dealloc HeapMem"),
      Ok(_) => {}
    }
  }
}

impl<T> HeapMem<T> where T: Copy {
  pub unsafe fn alloc(len: usize) -> Self {
    let p = match Global::default().alloc_array(len) {
      Err(_) => panic!("failed to alloc HeapMem"),
      Ok(p) => p,
    };
    let buf: *mut T = p.as_ptr();
    match (size_of::<T>() % align_of::<T>(), buf.align_offset(align_of::<T>())) {
      (0, 0) => {}
      (0, _) => panic!("malloc returned non-naturally aligned pointer"),
      (_, _) => panic!("size is not a multiple of alignment"),
    }
    HeapMem{
      buf,
      len,
    }
  }

  pub unsafe fn alloc_aligned(_len: usize, _alignment: usize) -> Self {
    // FIXME: allocator_api requires storing the alignment.
    unimplemented!();
  }
}

impl<T> HeapMem<T> where T: ZeroBits {
  pub fn zeros(len: usize) -> Self {
    let mem = unsafe { HeapMem::<T>::alloc(len) };
    unsafe { write_bytes::<u8>(mem.as_ptr_mut() as *mut u8, 0, mem.region_len() * size_of::<T>()) };
    mem
  }

  pub fn zeros_aligned(len: usize, alignment: usize) -> Self {
    let mem = unsafe { HeapMem::<T>::alloc_aligned(len, alignment) };
    unsafe { write_bytes::<u8>(mem.as_ptr_mut() as *mut u8, 0, mem.region_len() * size_of::<T>()) };
    mem
  }
}

impl<T> PodRegion<T> for HeapMem<T> where T: Copy {
  fn region_len(&self) -> usize {
    self.len
  }

  fn as_ptr(&self) -> *const T {
    self.buf
  }

  fn as_slice(&self) -> &[T] {
    unsafe { from_raw_parts(self.buf, self.len) }
  }

  fn as_bytes(&self) -> &[u8] {
    unsafe { from_raw_parts(self.buf as *const u8, self.len * size_of::<T>()) }
  }
}

impl<T> PodRegionMut<T> for HeapMem<T> where T: Copy {
  fn as_ptr_mut(&self) -> *mut T {
    self.buf
  }

  fn as_slice_mut(&mut self) -> &mut [T] {
    unsafe { from_raw_parts_mut(self.buf, self.len) }
  }

  fn as_bytes_mut(&mut self) -> &mut [u8] {
    unsafe { from_raw_parts_mut(self.buf as *mut u8, self.len * size_of::<T>()) }
  }
}
