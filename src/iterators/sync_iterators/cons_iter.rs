#[allow(unused_imports)]
use crate::iterators::ProdIter;
use crate::iterators::iterator_trait::{ORBIterator, PrivateORBIterator};
use crate::iterators::sync_iterators::Iter;
use crate::iterators::{copy_from_slice_unchecked, private_impl};
use crate::ring_buffer::OneRB;
use crate::ring_buffer::storage_components::PStorageComponent;
use crate::ring_buffer::storage_components::StorageComponent;
use crate::ring_buffer::wrappers::refs::IntoRef;
use crate::ring_buffer::{SharedRB, iters_components::PIterComponent};
#[cfg(feature = "async")]
use crate::{
    iterators::{AsyncConsIter, async_iterators::AsyncIterator},
    iters_components::async_iters::AsyncIterComp,
};

#[doc = r##"
Iterator used to pop data from the buffer.

When working with types which implement both [`Copy`] and [`Clone`] traits, `copy` methods should be
preferred over `clone` methods.
"##]
#[repr(transparent)]
pub struct ConsIter<B: IntoRef + OneRB> {
    inner: Iter<B>,
}

unsafe impl<B: IntoRef + OneRB + SharedRB> Send for ConsIter<B> {}

impl<B: IntoRef + OneRB<Item = T>, T> PrivateORBIterator for ConsIter<B> {
    type _Buffer = B;

    #[inline]
    fn _available(&mut self) -> usize {
        let succ_idx = self.succ_index();

        unsafe {
            self.inner.cached_avail = match self.inner.index <= succ_idx {
                true => succ_idx.unchecked_sub(self.inner.index),
                false => self
                    .buffer()
                    .storage()
                    .len()
                    .unchecked_sub(self.inner.index)
                    .unchecked_add(succ_idx),
            };
        }

        self.inner.cached_avail
    }

    #[inline]
    fn set_atomic_index(&self, index: usize) {
        self.inner.buffer.iters().set_cons_index(index);
    }

    #[inline]
    fn succ_index(&self) -> usize {
        self.inner.buffer.iters().middle_iter_idx()
    }

    private_impl!();
}

impl<B: IntoRef + OneRB<Item = T>, T> ORBIterator for ConsIter<B> {
    type Item = T;
    type Buffer = B;
}

#[cfg(feature = "async")]
impl<B: IntoRef + OneRB<Iters: AsyncIterComp>> ConsIter<B> {
    pub fn into_async(self) -> AsyncConsIter<B> {
        AsyncIterator::from_sync(self)
    }
}

impl<B: IntoRef + OneRB<Item = T>, T> ConsIter<B> {
    pub(crate) fn new(value: B::TargetRef) -> Self {
        Self {
            inner: Iter::new(value),
        }
    }

    /// Resets the index of the iterator. I.e., moves the iterator to the location occupied by its successor.
    #[inline]
    pub fn reset_index(&mut self) {
        let new_idx = self.succ_index();
        self.inner.index = new_idx;
        self.set_atomic_index(new_idx);
    }

    /// Returns a reference to an element.
    /// <div class="warning">
    ///
    /// Being this a reference, [`Self::advance()`] has to be called when done with the data
    /// in order to move the iterator.
    /// </div>
    #[inline]
    pub fn peek_ref<'a>(&mut self) -> Option<&'a T> {
        self.next_ref()
    }

    /// Returns a tuple of slice references, the sum of which with len equal to `count`.
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the data
    /// in order to move the iterator.
    /// </div>
    #[inline]
    pub fn peek_slice<'a>(
        &mut self,
        count: usize,
    ) -> Option<<<B as OneRB>::Storage as StorageComponent>::SliceOutput<'a>> {
        self.check(count).then(|| {
            self.inner
                .buffer
                .storage()
                .next_chunk(self.inner.index, count)
        })
    }

    /// Returns a tuple of slice references, the sum of which with len equal to available data.
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the data
    /// in order to move the iterator.
    /// </div>
    #[inline]
    pub fn peek_available<'a>(
        &mut self,
    ) -> Option<<<B as OneRB>::Storage as StorageComponent>::SliceOutput<'a>> {
        match self.available() {
            0 => None,
            avail => self.peek_slice(avail),
        }
    }

    /// Tries to pop an element, moving it.
    /// # Safety
    /// This method moves items, so locations from which they are moved out are left uninitialised.
    /// These locations must be re-initialised used proper [`ProdIter`] methods (`*_init`) ones
    #[inline]
    pub unsafe fn pop_move(&mut self) -> Option<T> {
        self.next()
    }

    /// Tries to pop an element, duplicating it.
    /// # Safety
    /// This method behaves like `ptr::read`: it duplicates the item by making a bitwise copy, ignoring whether it is `Copy`/`Clone` or not.
    /// So it is your responsibility to ensure that the data may indeed be duplicated.
    /// Ignoring this requirement might lead to errors, like double-frees.
    /// E.g. duplicating a `Vec` or other heap-allocated structs copies only their pointer, so, once the copied `Vec`
    /// gets deallocated, dropping its copy results in a double-free.
    /// `Self::pop` and `Self::pop_clone` should be preferred over this method.
    #[inline]
    pub unsafe fn pop_unsafe(&mut self) -> Option<T> {
        self.next_duplicate()
    }

    /// Tries to pop an element, copying it.
    #[inline]
    pub fn pop(&mut self) -> Option<T>
    where
        T: Copy,
    {
        self.next_duplicate()
    }

    /// Tries to pop an element, cloning it.
    /// When possible, `Self::pop` should be preferred over this method.
    #[inline]
    pub fn pop_clone(&mut self) -> Option<T>
    where
        T: Clone,
    {
        let ret = self.peek_ref().map(|e| e.clone());
        if ret.is_some() {
            unsafe {
                self.advance(1);
            }
        }
        ret
    }

    #[inline]
    fn _extract_item(&mut self, dst: &mut T, f: fn(&T, &mut T)) -> Option<()> {
        if let Some(v) = self.next_ref() {
            f(v, dst);

            unsafe { self.advance(1) };
            Some(())
        } else {
            None
        }
    }

    /// - Returns `Some(())`, copying next item into `dst`, if available.
    /// - Returns `None` doing nothing, otherwise.
    ///
    /// This method uses `copy` and should be preferred over `clone` version, if possible.
    /// <div class="warning">
    ///
    /// Unlike `peek*` methods, this one automatically advances the iterator.
    /// </div>
    #[inline]
    pub fn copy_item(&mut self, dst: &mut T) -> Option<()>
    where
        T: Copy,
    {
        #[inline]
        fn f<T: Copy>(src: &T, dst: &mut T) {
            *dst = *src;
        }
        self._extract_item(dst, f)
    }

    /// Same as [`Self::copy_item`], but uses `clone`, instead.
    /// <div class="warning">
    ///
    /// Unlike `peek*` methods, this one automatically advances the iterator.
    /// </div>
    #[inline]
    pub fn clone_item(&mut self, dst: &mut T) -> Option<()>
    where
        T: Clone,
    {
        fn f<T: Clone>(src: &T, dst: &mut T) {
            *dst = src.clone();
        }
        self._extract_item(dst, f)
    }

    #[inline]
    fn _extract_slice(&mut self, dst: &mut [T], f: fn(&[T], &mut [T])) -> Option<()> {
        let count = dst.len();

        self.check(count).then(|| {
            self.inner
                .buffer
                .storage_mut()
                ._extract_slice(self.inner.index, dst, f);
            unsafe { self.advance(count) };
        })
    }

    /// - Returns `Some(())`, filling `dst` slice with the next `dst.len()` values, if available.
    /// - Returns `None` doing nothing, otherwise.
    ///
    /// This method fills the slice using `copy` and should be preferred over `clone` version, if possible.
    /// <div class="warning">
    ///
    /// Unlike `peek*` methods, this one automatically advances the iterator.
    /// </div>
    #[inline]
    pub fn copy_slice(&mut self, dst: &mut [T]) -> Option<()>
    where
        T: Copy,
    {
        fn f<T: Copy>(binding: &[T], dst: &mut [T]) {
            copy_from_slice_unchecked(binding, dst);
        }

        self._extract_slice(dst, f)
    }

    /// Same as [`Self::copy_slice`], but uses `clone`, instead.
    /// <div class="warning">
    ///
    /// Unlike `peek*` methods, this one automatically advances the iterator.
    /// </div>
    #[inline]
    pub fn clone_slice(&mut self, dst: &mut [T]) -> Option<()>
    where
        T: Clone,
    {
        fn f<T: Clone>(binding: &[T], dst: &mut [T]) {
            dst.clone_from_slice(binding);
        }

        self._extract_slice(dst, f)
    }
}

mod test {

    #[test]
    fn cached_avail() {
        use super::*;

        const BUFFER_SIZE: usize = 100;

        #[cfg(feature = "alloc")]
        let buf = crate::SharedHeapRB::<u32>::default(BUFFER_SIZE + 1);
        #[cfg(not(feature = "alloc"))]
        let mut buf = crate::SharedStackRB::<u32, { BUFFER_SIZE + 1 }>::default();

        let (mut prod, mut cons) = buf.split();

        assert_eq!(cons.inner.cached_avail, 0);

        unsafe {
            prod.advance(10);
        }

        assert_eq!(cons.inner.cached_avail, 0);

        cons.check(1);

        assert_eq!(cons.inner.cached_avail, 10);

        unsafe {
            cons.advance(9);
        }

        assert_eq!(cons.inner.cached_avail, 1);
    }
}
