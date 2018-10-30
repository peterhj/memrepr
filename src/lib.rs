#![feature(allocator_api)]
#![feature(ptr_internals)]

#[cfg(feature = "f16")] use float::stub::{f16_stub};

use std::alloc::{Alloc, Global};
use std::mem::{size_of};
use std::ptr::{NonNull, null_mut, write_bytes};
use std::slice::{from_raw_parts, from_raw_parts_mut};

pub trait ZeroBits: Copy {}

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

pub trait ReadOnlyMem<T> where T: Copy {
  fn len(&self) -> usize;
  unsafe fn as_ptr(&self) -> *const T;
  fn as_slice(&self) -> &[T];
  fn as_bytes(&self) -> &[u8];
}

pub trait Mem<T>: ReadOnlyMem<T> where T: Copy {
  unsafe fn as_mut_ptr(&mut self) -> *mut T;
  fn as_mut_slice(&mut self) -> &mut [T];
  fn as_mut_bytes(&mut self) -> &mut [u8];
}

pub struct HeapMem<T> where T: Copy {
  buf:  *mut T,
  len:  usize,
  phsz: usize,
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
    let phsz = len * size_of::<T>();
    let p = match Global::default().alloc_array(len) {
      Err(_) => panic!("failed to alloc HeapMem"),
      Ok(p) => p,
    };
    HeapMem{
      buf:  p.as_ptr(),
      len:  len,
      phsz: phsz,
    }
  }

  pub unsafe fn alloc_aligned(_len: usize, _alignment: usize) -> Self {
    unimplemented!();
  }
}

impl<T> HeapMem<T> where T: ZeroBits {
  pub fn zeros(len: usize) -> Self {
    let mut mem = unsafe { HeapMem::<T>::alloc(len) };
    unsafe { write_bytes::<T>(mem.as_mut_ptr(), 0, mem.len()) };
    mem
  }

  pub fn zeros_aligned(_len: usize, _alignment: usize) -> Self {
    unimplemented!();
  }
}

impl<T> ReadOnlyMem<T> for HeapMem<T> where T: Copy {
  fn len(&self) -> usize {
    self.len
  }

  unsafe fn as_ptr(&self) -> *const T {
    self.buf
  }

  fn as_slice(&self) -> &[T] {
    unsafe { from_raw_parts(self.buf, self.len) }
  }

  fn as_bytes(&self) -> &[u8] {
    unsafe { from_raw_parts(self.buf as *const u8, self.phsz) }
  }
}

impl<T> Mem<T> for HeapMem<T> where T: Copy {
  unsafe fn as_mut_ptr(&mut self) -> *mut T {
    self.buf
  }

  fn as_mut_slice(&mut self) -> &mut [T] {
    unsafe { from_raw_parts_mut(self.buf, self.len) }
  }

  fn as_mut_bytes(&mut self) -> &mut [u8] {
    unsafe { from_raw_parts_mut(self.buf as *mut u8, self.phsz) }
  }
}
