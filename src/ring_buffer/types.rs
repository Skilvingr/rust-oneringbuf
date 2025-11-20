#[cfg(all(feature = "async"))]
use crate::iters_components::async_iters::{mutable::AsyncCompMut, non_mutable::AsyncComp};
#[cfg(feature = "alloc")]
use crate::storage_components::HeapStorage;
#[cfg(all(feature = "alloc", feature = "vmem", unix))]
use crate::storage_components::VmemStorage;
use crate::{
    OneRingBuf,
    iters_components::{
        local_iters::{mutable::LocalCompMut, non_mutable::LocalComp},
        shared_iters::{mutable::SharedCompMut, non_mutable::SharedComp},
    },
    storage_components::StackStorage,
};

// Stack
/// Non-mutable stack-allocated ring buffer suitable for single-threaded usage.
pub type LocalStackRB<'buf, T, const N: usize> = OneRingBuf<StackStorage<'buf, T, N>, LocalComp>;
/// Non-mutable stack-allocated ring buffer suitable for multi-threaded usage.
pub type SharedStackRB<'buf, T, const N: usize> = OneRingBuf<StackStorage<'buf, T, N>, SharedComp>;
/// Mutable stack-allocated ring buffer suitable for single-threaded usage.
pub type LocalStackRBMut<'buf, T, const N: usize> =
    OneRingBuf<StackStorage<'buf, T, N>, LocalCompMut>;
/// Mutable stack-allocated ring buffer suitable for multi-threaded usage.
pub type SharedStackRBMut<'buf, T, const N: usize> =
    OneRingBuf<StackStorage<'buf, T, N>, SharedCompMut>;

// Heap
/// Non-mutable heap-allocated ring buffer suitable for single-threaded usage.
#[cfg(feature = "alloc")]
pub type LocalHeapRB<T> = OneRingBuf<HeapStorage<T>, LocalComp>;
/// Non-mutable heap-allocated ring buffer suitable for multi-threaded usage.
#[cfg(feature = "alloc")]
pub type SharedHeapRB<T> = OneRingBuf<HeapStorage<T>, SharedComp>;
/// Mutable heap-allocated ring buffer suitable for single-threaded usage.
#[cfg(feature = "alloc")]
pub type LocalHeapRBMut<T> = OneRingBuf<HeapStorage<T>, LocalCompMut>;
/// Mutable heap-allocated ring buffer suitable for multi-threaded usage.
#[cfg(feature = "alloc")]
pub type SharedHeapRBMut<T> = OneRingBuf<HeapStorage<T>, SharedCompMut>;

// Vmem
/// Non-mutable ring buffer using virtual memory storage suitable for single-threaded usage.
#[cfg(all(feature = "alloc", feature = "vmem", unix))]
pub type LocalVmemRB<T> = OneRingBuf<VmemStorage<T>, LocalComp>;
/// Non-mutable ring buffer using virtual memory storage suitable for multi-threaded usage.
#[cfg(all(feature = "alloc", feature = "vmem", unix))]
pub type SharedVmemRB<T> = OneRingBuf<VmemStorage<T>, SharedComp>;
/// Mutable ring buffer using virtual memory storage suitable for single-threaded usage.
#[cfg(all(feature = "alloc", feature = "vmem", unix))]
pub type LocalVmemRBMut<T> = OneRingBuf<VmemStorage<T>, LocalCompMut>;
/// Mutable ring buffer using virtual memory storage suitable for multi-threaded usage.
#[cfg(all(feature = "alloc", feature = "vmem", unix))]
pub type SharedVmemRBMut<T> = OneRingBuf<VmemStorage<T>, SharedCompMut>;

// Async Stack
/// Non-mutable stack-allocated asynchronous ring buffer.
#[cfg(all(feature = "async"))]
pub type AsyncStackRB<'buf, T, const N: usize> = OneRingBuf<StackStorage<'buf, T, N>, AsyncComp>;
/// Mutable stack-allocated asynchronous ring buffer.
#[cfg(all(feature = "async"))]
pub type AsyncStackRBMut<'buf, T, const N: usize> =
    OneRingBuf<StackStorage<'buf, T, N>, AsyncCompMut>;
#[cfg(all(feature = "async"))]
unsafe impl<T, const N: usize> Sync for AsyncStackRB<'_, T, N> {}
#[cfg(all(feature = "async"))]
unsafe impl<T, const N: usize> Sync for AsyncStackRBMut<'_, T, N> {}

// Async Heap
/// Non-mutable heap-allocated asynchronous ring buffer.
#[cfg(all(feature = "async", feature = "alloc"))]
pub type AsyncHeapRB<T> = OneRingBuf<HeapStorage<T>, AsyncComp>;
/// Mutable heap-allocated asynchronous ring buffer.
#[cfg(all(feature = "async", feature = "alloc"))]
pub type AsyncHeapRBMut<T> = OneRingBuf<HeapStorage<T>, AsyncCompMut>;
#[cfg(all(feature = "async", feature = "alloc"))]
unsafe impl<T> Sync for AsyncHeapRB<T> {}
#[cfg(all(feature = "async", feature = "alloc"))]
unsafe impl<T> Sync for AsyncHeapRBMut<T> {}

// Async Vmem
/// Non-mutable asynchronous ring buffer using virtual memory storage.
#[cfg(all(feature = "async", feature = "alloc", feature = "vmem", unix))]
pub type AsyncVmemRB<T> = OneRingBuf<VmemStorage<T>, AsyncComp>;
/// Mutable asynchronous ring buffer using virtual memory storage.
#[cfg(all(feature = "async", feature = "alloc", feature = "vmem", unix))]
pub type AsyncVmemRBMut<T> = OneRingBuf<VmemStorage<T>, AsyncCompMut>;
#[cfg(all(feature = "async", feature = "alloc", feature = "vmem", unix))]
unsafe impl<T> Sync for AsyncVmemRB<T> {}
#[cfg(all(feature = "async", feature = "alloc", feature = "vmem", unix))]
unsafe impl<T> Sync for AsyncVmemRBMut<T> {}
