pub mod rb;

use core::marker::PhantomData;
use core::mem::transmute;
use core::ops::Index;
use core::slice;

use crate::ring_buffer::storage_components::{PStorageComponent, StorageComponent};
use crate::utils::UnsafeSyncCell;

/// Stack-allocated storage.
pub struct StackStorage<'buf, T, const N: usize> {
    inner: [UnsafeSyncCell<T>; N],
    phantom: PhantomData<&'buf ()>,
}

impl<'buf, T, const N: usize> From<[T; N]> for StackStorage<'buf, T, N> {
    fn from(value: [T; N]) -> StackStorage<'buf, T, N> {
        let value = core::mem::ManuallyDrop::new(value);
        let ptr = &value as *const _ as *const [UnsafeSyncCell<T>; N];

        Self {
            inner: unsafe { ptr.read() },
            phantom: PhantomData,
        }
    }
}

impl<'buf, T, const N: usize> From<[UnsafeSyncCell<T>; N]> for StackStorage<'buf, T, N> {
    fn from(value: [UnsafeSyncCell<T>; N]) -> StackStorage<'buf, T, N> {
        Self {
            inner: value,
            phantom: PhantomData,
        }
    }
}

impl<'buf, T, const N: usize> Index<usize> for StackStorage<'buf, T, N> {
    type Output = UnsafeSyncCell<T>;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        assert!(
            index < N,
            "index out of bounds: the len is {N} but the index is {index}"
        );
        unsafe { self.inner.get_unchecked(index) }
    }
}

impl<'buf, T, const N: usize> StorageComponent for StackStorage<'buf, T, N> {
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

impl<'buf, T, const N: usize> PStorageComponent for StackStorage<'buf, T, N> {
    #[inline]
    fn _index(&self, index: usize) -> &UnsafeSyncCell<Self::Item> {
        unsafe { self.inner.get_unchecked(index) }
    }

    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    fn next_chunk<'a>(&self, index: usize, count: usize) -> Self::SliceOutput<'a> {
        let ptr = self.inner.as_ptr();

        unsafe {
            if index + count > N {
                (
                    transmute::<&[UnsafeSyncCell<T>], &[T]>(slice::from_raw_parts(
                        ptr.add(index),
                        N.unchecked_sub(index),
                    )),
                    transmute::<&[UnsafeSyncCell<T>], &[T]>(slice::from_raw_parts(
                        ptr,
                        index.unchecked_add(count).unchecked_sub(N),
                    )),
                )
            } else {
                (
                    transmute::<&[UnsafeSyncCell<T>], &[T]>(slice::from_raw_parts(
                        ptr.add(index),
                        count,
                    )),
                    &mut [] as &[T],
                )
            }
        }
    }

    #[inline]
    fn next_chunk_mut<'a>(&mut self, index: usize, count: usize) -> Self::SliceOutputMut<'a> {
        let ptr = self.inner.as_mut_ptr();

        unsafe {
            if index + count > N {
                (
                    transmute::<&mut [UnsafeSyncCell<T>], &mut [T]>(slice::from_raw_parts_mut(
                        ptr.add(index),
                        N.unchecked_sub(index),
                    )),
                    transmute::<&mut [UnsafeSyncCell<T>], &mut [T]>(slice::from_raw_parts_mut(
                        ptr,
                        index.unchecked_add(count).unchecked_sub(N),
                    )),
                )
            } else {
                (
                    transmute::<&mut [UnsafeSyncCell<T>], &mut [T]>(slice::from_raw_parts_mut(
                        ptr.add(index),
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
        use crate::storage_components::StackStorage;
        use crate::utils::UnsafeSyncCell;

        let _ = StackStorage::from([0; 100]);
        let _: StackStorage<i32, 100> =
            StackStorage::from(core::array::from_fn(|_| UnsafeSyncCell::new(0)));
    }
}
