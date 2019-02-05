#![cfg_attr(feature = "std", feature(align_offset))]
#![cfg_attr(feature = "std", feature(allocator_api))]
#![cfg_attr(feature = "std", feature(ptr_internals))]

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "f16")]
use half::{f16 as f16_stub};

#[cfg(feature = "std")]
pub mod heap;

pub trait PodRegion<T: Copy> {
  fn region_len(&self) -> usize;
  fn as_ptr(&self) -> *const T;
  fn as_slice(&self) -> &[T];
  fn as_bytes(&self) -> &[u8];
}

pub trait PodRegionMut<T: Copy>: PodRegion<T> {
  fn as_ptr_mut(&self) -> *mut T;
  fn as_slice_mut(&mut self) -> &mut [T];
  fn as_bytes_mut(&mut self) -> &mut [u8];
}

pub trait DmaRegion<T: Copy + 'static> {
  fn dma_region_len(&self) -> usize;
  fn as_dma_ptr(&self) -> *const T;
}

pub trait DmaRegionMut<T: Copy + 'static>: DmaRegion<T> {
  fn as_dma_ptr_mut(&self) -> *mut T;
}

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

#[cfg(feature = "cuda")]
impl ZeroBits for ::cuda::ffi::cuda_fp16::__half {}
#[cfg(feature = "cuda")]
impl ZeroBits for ::cuda::ffi::cuda_fp16::__half2 {}
#[cfg(feature = "f16")]
impl ZeroBits for f16_stub {}
impl ZeroBits for f32 {}
impl ZeroBits for f64 {}

impl<T> ZeroBits for *const T {}
impl<T> ZeroBits for *mut T {}
