#![cfg_attr(doc, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
#![no_std]
#![allow(private_bounds, private_interfaces)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[doc(inline)]
pub use iterators::ORBIterator;

pub use crate::ring_buffer::iters_components::IterComponent;
pub use crate::ring_buffer::storage_components::StorageComponent;
pub use crate::ring_buffer::{OneRB, OneRingBuf};
pub use ring_buffer::wrappers::refs::IntoRef;

// AsyncStackRB
#[cfg(any(feature = "async", doc))]
pub use crate::ring_buffer::types::{AsyncStackRB, AsyncStackRBMut};

// AsyncHeapRB
#[cfg(all(feature = "alloc", feature = "async"))]
pub use crate::ring_buffer::types::{AsyncHeapRB, AsyncHeapRBMut};

// AsyncHeapRB
#[cfg(all(feature = "alloc", feature = "async", feature = "vmem", unix))]
pub use crate::ring_buffer::types::{AsyncVmemRB, AsyncVmemRBMut};

// Heap
#[cfg(feature = "alloc")]
pub use crate::ring_buffer::types::{LocalHeapRB, LocalHeapRBMut, SharedHeapRB, SharedHeapRBMut};

// Vmem
#[cfg(any(doc, all(feature = "alloc", feature = "vmem", unix)))]
pub use crate::ring_buffer::types::{LocalVmemRB, LocalVmemRBMut, SharedVmemRB, SharedVmemRBMut};

// Stack
pub use crate::ring_buffer::types::{
    LocalStackRB, LocalStackRBMut, SharedStackRB, SharedStackRBMut,
};

pub use ring_buffer::iters_components;
pub use ring_buffer::storage_components;

pub mod iterators;
mod ring_buffer;

/// Various utilities
pub mod utils {
    pub use crate::ring_buffer::wrappers::unsafe_sync_cell::UnsafeSyncCell;
    #[cfg(all(feature = "alloc", feature = "vmem", unix))]
    pub use crate::storage_components::alloc::vmem::vmem_helper;
}
