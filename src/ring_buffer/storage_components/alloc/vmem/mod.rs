#![cfg(all(feature = "vmem", unix))]

pub mod rb;
pub mod vmem_helper;

use core::mem::transmute;
use core::ops::Index;
use core::slice;

use crate::ring_buffer::storage_components::{PStorageComponent, StorageComponent};
use crate::utils::UnsafeSyncCell;
use alloc::boxed::Box;
use alloc::vec::Vec;

/// Heap-allocated storage with vmem optimisation.
pub struct VmemStorage<T> {
    inner: *mut UnsafeSyncCell<T>,
    len: usize,
}

impl<T> Drop for VmemStorage<T> {
    fn drop(&mut self) {
        unsafe {
            let p = core::ptr::slice_from_raw_parts_mut(self.inner, self.len);
            core::ptr::drop_in_place(p);

            let size = self.len * size_of::<T>();

            // glibc manual says that it is fine to unmap two mapped blocks at the same
            // time. Nevertheless, I don't trust such a guarantee, as everything can change
            // at any time, so the blocks get unmapped one after the other.
            libc::munmap(self.inner.byte_add(size) as _, size);
            libc::munmap(self.inner as _, size);
        }
    }
}

impl<T> VmemStorage<T> {
    fn new(value: Box<[UnsafeSyncCell<T>]>) -> Self {
        let r = vmem_helper::new(&value);

        Self {
            inner: r,
            len: value.len(),
        }
    }
}

impl<T> From<Box<[T]>> for VmemStorage<T> {
    fn from(value: Box<[T]>) -> Self {
        Self::new(unsafe { core::mem::transmute::<Box<[T]>, Box<[UnsafeSyncCell<T>]>>(value) })
    }
}

impl<T> From<Box<[UnsafeSyncCell<T>]>> for VmemStorage<T> {
    fn from(value: Box<[UnsafeSyncCell<T>]>) -> Self {
        Self::new(value)
    }
}

impl<T> From<Vec<T>> for VmemStorage<T> {
    fn from(value: Vec<T>) -> Self {
        Self::from(value.into_boxed_slice())
    }
}

impl<T> From<Vec<UnsafeSyncCell<T>>> for VmemStorage<T> {
    fn from(value: Vec<UnsafeSyncCell<T>>) -> Self {
        Self::new(value.into_boxed_slice())
    }
}

impl<T> Index<usize> for VmemStorage<T> {
    type Output = UnsafeSyncCell<T>;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        assert!(
            index < self.len,
            "index out of bounds: the len is {} but the index is {index}",
            self.len
        );
        unsafe { &*self.inner.add(index) }
    }
}

impl<T> StorageComponent for VmemStorage<T> {
    type Item = T;
    type SliceOutput<'a>
        = &'a [T]
    where
        T: 'a;
    type SliceOutputMut<'a>
        = &'a mut [T]
    where
        T: 'a;
}

impl<T> PStorageComponent for VmemStorage<T> {
    #[inline]
    fn _index(&self, index: usize) -> &UnsafeSyncCell<Self::Item> {
        unsafe { &*self.inner.add(index) }
    }

    #[inline]
    fn len(&self) -> usize {
        self.len
    }

    #[inline]
    fn next_chunk<'a>(&self, index: usize, count: usize) -> Self::SliceOutput<'a> {
        unsafe {
            transmute::<&[UnsafeSyncCell<T>], &[T]>(slice::from_raw_parts(
                self.inner.add(index),
                count,
            ))
        }
    }

    #[inline]
    fn next_chunk_mut<'a>(&mut self, index: usize, count: usize) -> Self::SliceOutputMut<'a> {
        unsafe {
            transmute::<&mut [UnsafeSyncCell<T>], &mut [T]>(slice::from_raw_parts_mut(
                self.inner.add(index),
                count,
            ))
        }
    }

    #[inline]
    fn _push_slice(
        &mut self,
        index: usize,
        slice: &[Self::Item],
        f: fn(&mut [Self::Item], &[Self::Item]),
    ) {
        let binding = self.next_chunk_mut(index, slice.len());
        f(binding, slice);
    }

    #[inline]
    fn _extract_slice(
        &mut self,
        index: usize,
        dst: &mut [Self::Item],
        f: fn(&[Self::Item], &mut [Self::Item]),
    ) {
        let binding = self.next_chunk_mut(index, dst.len());
        f(binding, dst);
    }
}

pub mod test {
    #[test]
    fn from_tests() {
        use crate::storage_components::VmemStorage;
        use crate::utils::UnsafeSyncCell;
        use alloc::vec;

        let buf_len = crate::utils::vmem_helper::page_size();

        let _ = VmemStorage::from(vec![0; buf_len]);
        let _ = VmemStorage::from(vec![0; buf_len].into_boxed_slice());
        let _: VmemStorage<i32> = VmemStorage::from(vec![UnsafeSyncCell::new(0i32); buf_len]);
        let _: VmemStorage<i32> =
            VmemStorage::from(vec![UnsafeSyncCell::new(0i32); buf_len].into_boxed_slice());
    }
}
