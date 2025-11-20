#![allow(unused_mut)]

pub mod async_tests;
pub mod common;
pub mod sync_tests;

#[cfg(all(feature = "vmem", unix))]
macro_rules! common_def {
    () => {
        #[cfg(target_arch = "aarch64")]
        const BUFFER_SIZE: usize = 16384;
        #[cfg(not(target_arch = "aarch64"))]
        const BUFFER_SIZE: usize = 4096;
    };
}
#[cfg(not(all(feature = "vmem", unix)))]
macro_rules! common_def {
    () => {
        const BUFFER_SIZE: usize = 400;
    };
}

#[cfg(all(feature = "alloc", feature = "vmem", unix))]
macro_rules! get_buf {
    (Local) => {
        oneringbuf::LocalVmemRB::from(vec![0; BUFFER_SIZE])
    };
    (LocalMut) => {
        oneringbuf::LocalVmemRBMut::from(vec![0; BUFFER_SIZE])
    };
    (Shared) => {
        oneringbuf::SharedVmemRB::from(vec![0; BUFFER_SIZE])
    };
    (SharedMut) => {
        oneringbuf::SharedVmemRBMut::from(vec![0; BUFFER_SIZE])
    };
}
#[cfg(all(feature = "alloc", not(all(feature = "vmem", unix))))]
macro_rules! get_buf {
    (Local) => {
        oneringbuf::LocalHeapRB::from(vec![0; BUFFER_SIZE])
    };
    (LocalMut) => {
        oneringbuf::LocalHeapRBMut::from(vec![0; BUFFER_SIZE])
    };
    (Shared) => {
        oneringbuf::SharedHeapRB::from(vec![0; BUFFER_SIZE])
    };
    (SharedMut) => {
        oneringbuf::SharedHeapRBMut::from(vec![0; BUFFER_SIZE])
    };
}
#[cfg(all(not(feature = "alloc"), not(all(feature = "vmem", unix))))]
macro_rules! get_buf {
    (Local) => {
        oneringbuf::LocalStackRB::from([0; BUFFER_SIZE])
    };
    (LocalMut) => {
        oneringbuf::LocalStackRBMut::from([0; BUFFER_SIZE])
    };
    (Shared) => {
        oneringbuf::SharedStackRB::from([0; BUFFER_SIZE])
    };
    (SharedMut) => {
        oneringbuf::SharedStackRBMut::from([0; BUFFER_SIZE])
    };
}
pub(crate) use {common_def, get_buf};

#[cfg(feature = "alloc")]
#[test]
#[should_panic]
fn len_zero_heap() {
    let _ = oneringbuf::SharedHeapRB::<i32>::default(0);
}

#[cfg(all(feature = "alloc", feature = "vmem", unix))]
#[test]
#[should_panic]
fn len_zero_vmem() {
    let _ = oneringbuf::SharedVmemRB::<i32>::default(0);
}

#[cfg(not(all(feature = "vmem", unix)))]
#[test]
#[should_panic]
fn len_zero_stack() {
    let _ = oneringbuf::SharedStackRB::<i32, 0>::default();
}
