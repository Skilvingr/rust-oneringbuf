pub mod rb;

use core::mem::transmute;
use core::ops::Index;
use core::slice;

use crate::ring_buffer::storage_components::{PStorageComponent, StorageComponent};
use crate::utils::UnsafeSyncCell;
use alloc::boxed::Box;
use alloc::vec::Vec;

/// Heap-allocated storage.
pub struct HeapStorage<T> {
    inner: *mut UnsafeSyncCell<T>,
    len: usize,
}

impl<T> Drop for HeapStorage<T> {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(core::ptr::slice_from_raw_parts_mut(self.inner, self.len));
        }
    }
}

impl<T> HeapStorage<T> {
    fn new(value: Box<[UnsafeSyncCell<T>]>) -> Self {
        let len = value.len();

        let v = Box::into_raw(value);

        unsafe {
            Self {
                inner: (*v).as_mut_ptr(),
                len,
            }
        }
    }
}

impl<T> From<Box<[T]>> for HeapStorage<T> {
    fn from(value: Box<[T]>) -> Self {
        Self::new(unsafe { core::mem::transmute::<Box<[T]>, Box<[UnsafeSyncCell<T>]>>(value) })
    }
}

impl<T> From<Box<[UnsafeSyncCell<T>]>> for HeapStorage<T> {
    fn from(value: Box<[UnsafeSyncCell<T>]>) -> Self {
        Self::new(value)
    }
}

impl<T> From<Vec<T>> for HeapStorage<T> {
    fn from(value: Vec<T>) -> Self {
        Self::from(value.into_boxed_slice())
    }
}

impl<T> From<Vec<UnsafeSyncCell<T>>> for HeapStorage<T> {
    fn from(value: Vec<UnsafeSyncCell<T>>) -> Self {
        Self::new(value.into_boxed_slice())
    }
}

impl<T> Index<usize> for HeapStorage<T> {
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

impl<T> StorageComponent for HeapStorage<T> {
    type Item = T;
    type SliceOutput<'a>
        = (&'a [T], &'a [T])
    where
        T: 'a;
    type SliceOutputMut<'a>
        = (&'a mut [T], &'a mut [T])
    where
        T: 'a;
}

impl<T> PStorageComponent for HeapStorage<T> {
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
            if index + count > self.len {
                (
                    transmute::<&[UnsafeSyncCell<T>], &[T]>(slice::from_raw_parts(
                        self.inner.add(index),
                        self.len.unchecked_sub(index),
                    )),
                    transmute::<&[UnsafeSyncCell<T>], &[T]>(slice::from_raw_parts(
                        self.inner,
                        index.unchecked_add(count).unchecked_sub(self.len),
                    )),
                )
            } else {
                (
                    transmute::<&[UnsafeSyncCell<T>], &[T]>(slice::from_raw_parts(
                        self.inner.add(index),
                        count,
                    )),
                    &mut [] as &[T],
                )
            }
        }
    }

    #[inline]
    fn next_chunk_mut<'a>(&mut self, index: usize, count: usize) -> Self::SliceOutputMut<'a> {
        unsafe {
            if index + count > self.len {
                (
                    transmute::<&mut [UnsafeSyncCell<T>], &mut [T]>(slice::from_raw_parts_mut(
                        self.inner.add(index),
                        self.len.unchecked_sub(index),
                    )),
                    transmute::<&mut [UnsafeSyncCell<T>], &mut [T]>(slice::from_raw_parts_mut(
                        self.inner,
                        index.unchecked_add(count).unchecked_sub(self.len),
                    )),
                )
            } else {
                (
                    transmute::<&mut [UnsafeSyncCell<T>], &mut [T]>(slice::from_raw_parts_mut(
                        self.inner.add(index),
                        count,
                    )),
                    &mut [] as &mut [T],
                )
            }
        }
    }

    #[inline]
    fn _push_slice(
        &mut self,
        index: usize,
        slice: &[Self::Item],
        f: fn(&mut [Self::Item], &[Self::Item]),
    ) {
        let count = slice.len();

        let (binding_h, binding_t) = self.next_chunk_mut(index, count);
        let mid = binding_h.len();
        if mid == slice.len() {
            f(binding_h, slice);
        } else {
            unsafe {
                f(binding_h, slice.get_unchecked(..mid));
                f(binding_t, slice.get_unchecked(mid..));
            }
        }
    }

    #[inline]
    fn _extract_slice(
        &mut self,
        index: usize,
        dst: &mut [Self::Item],
        f: fn(&[Self::Item], &mut [Self::Item]),
    ) {
        let count = dst.len();

        let (binding_h, binding_t) = self.next_chunk_mut(index, count);
        let mid = binding_h.len();
        if mid == dst.len() {
            f(binding_h, dst);
        } else {
            unsafe {
                f(binding_h, dst.get_unchecked_mut(..mid));
                f(binding_t, dst.get_unchecked_mut(mid..));
            }
        }
    }
}

pub mod test {
    #[test]
    fn from_tests() {
        use crate::storage_components::HeapStorage;
        use crate::utils::UnsafeSyncCell;
        use alloc::vec;

        const BUF_LEN: usize = 100;

        let _ = HeapStorage::from(vec![0; BUF_LEN]);
        let _ = HeapStorage::from(vec![0; BUF_LEN].into_boxed_slice());
        let _: HeapStorage<i32> = HeapStorage::from(vec![UnsafeSyncCell::new(0i32); BUF_LEN]);
        let _: HeapStorage<i32> =
            HeapStorage::from(vec![UnsafeSyncCell::new(0i32); BUF_LEN].into_boxed_slice());
    }
}
