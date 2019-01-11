#![feature(align_offset)]
#![feature(allocator_api)]
#![feature(ptr_internals)]

#[cfg(feature = "f16")] use half::{f16 as f16_stub};

use std::alloc::{Alloc, Global};
use std::mem::{align_of, size_of};
use std::ptr::{NonNull, null_mut, write_bytes};
use std::slice::{from_raw_parts, from_raw_parts_mut};

pub trait ZeroBits: Copy + 'static {}

impl ZeroBits for u8 {}
impl ZeroBits for u16 {}
impl ZeroBits for u32 {}
impl ZeroBits for u64 {}
impl ZeroBits for usize {}

impl ZeroBits for i8 {}
impl ZeroBits for i16 {}
impl ZeroBits for i32 {}
impl ZeroBits for i64 {}
impl ZeroBits for isize {}

#[cfg(feature = "f16")] impl ZeroBits for f16_stub {}
impl ZeroBits for f32 {}
impl ZeroBits for f64 {}

pub trait Region<T: Copy + 'static> {
  fn region_len(&self) -> usize;
  fn as_ptr(&self) -> *const T;
  fn as_slice(&self) -> &[T];
  fn as_bytes(&self) -> &[u8];
}

pub trait RegionMut<T: Copy + 'static>: Region<T> {
  fn as_ptr_mut(&self) -> *mut T;
  fn as_slice_mut(&mut self) -> &mut [T];
  fn as_bytes_mut(&mut self) -> &mut [u8];
}

pub struct HeapMem<T: Copy + 'static> {
  buf:  *mut T,
  len:  usize,
}

impl<T> Drop for HeapMem<T> where T: Copy + 'static {
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

impl<T> HeapMem<T> where T: Copy + 'static {
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

impl<T> Region<T> for HeapMem<T> where T: Copy + 'static {
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

impl<T> RegionMut<T> for HeapMem<T> where T: Copy + 'static {
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
