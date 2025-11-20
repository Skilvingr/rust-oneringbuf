use crate::iterators::sync_iterators::detached::Detached;
use crate::ring_buffer::OneRB;
use crate::ring_buffer::iters_components::PIterComponent;
use crate::ring_buffer::storage_components::{PStorageComponent, StorageComponent};

/// Trait implemented by iterators.
///
/// This trait is not meant to be implemented outside of this crate.
pub trait ORBIterator: PrivateORBIterator<_Buffer = Self::Buffer> {
    /// Type of the items stored in the underlying buffer.
    type Item;
    /// Type of the underlying buffer.
    type Buffer: OneRB<Item = Self::Item>;

    /// Detaches the iterator, yielding a [`Detached`] iterator.
    ///
    /// This allows to "peek" at the buffer's contents without advancing the primary consumer.
    /// For a more detailed explanation and examples, see the documentation for [`Detached`].
    #[inline]
    fn detach(self) -> Detached<Self>
    where
        Self: Sized,
    {
        Detached::from_iter(self)
    }

    /// Advances the iterator by `count`.
    ///
    /// # Safety
    /// An iterator should never overstep its successor, so it must always be: `count` <= [`ORBIterator::available()`]!
    #[inline]
    unsafe fn advance(&mut self, count: usize) {
        unsafe { self._advance(count) };
    }

    /// Returns the number of items available for an iterator.
    #[inline]
    fn available(&mut self) -> usize {
        self._available()
    }

    /// Waits, blocking the thread in a loop, until there are at least `count` available items.
    fn wait_for(&mut self, count: usize) {
        while self.available() < count {}
    }

    /// Returns the index of the iterator.
    #[inline]
    fn index(&self) -> usize {
        self._index()
    }

    /// Returns the length of the buffer.
    #[inline]
    fn buf_len(&self) -> usize {
        self.buffer().len()
    }

    /// Returns how many iterators are still alive.
    fn alive_iters(&self) -> u8 {
        self.buffer().iters().alive_iters()
    }

    /// Returns the index of the producer.
    #[inline(always)]
    fn prod_index(&self) -> usize {
        self.buffer().iters().prod_index()
    }
    /// Returns the index of the worker.
    #[inline(always)]
    fn work_index(&self) -> usize {
        self.buffer().iters().work_index()
    }
    /// Returns the index of the consumer.
    #[inline(always)]
    fn cons_index(&self) -> usize {
        self.buffer().iters().cons_index()
    }

    /// Returns a mutable references to the current value.
    ///
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the mutation
    /// in order to move the iterator.
    /// </div>
    #[inline]
    fn get_mut<'a>(&mut self) -> Option<&'a mut Self::Item> {
        self.next_ref_mut()
    }

    /// Returns a tuple of mutable slice references, the sum of which with len equal to `count`.
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the mutation
    /// in order to move the iterator.
    /// </div>
    #[inline]
    fn get_mut_slice_exact<'a>(
        &mut self,
        count: usize,
    ) -> Option<<<Self::Buffer as OneRB>::Storage as StorageComponent>::SliceOutputMut<'a>> {
        self.check(count).then(|| {
            self.buffer()
                .storage_mut()
                .next_chunk_mut(self._index(), count)
        })
    }

    /// Returns a tuple of mutable slice references, the sum of which with len equal to [`Self::available()`].
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the mutation
    /// in order to move the iterator.
    /// </div>
    #[inline]
    fn get_mut_slice_avail<'a>(
        &mut self,
    ) -> Option<<<Self::Buffer as OneRB>::Storage as StorageComponent>::SliceOutputMut<'a>> {
        match self.available() {
            0 => None,
            avail => self.get_mut_slice_exact(avail),
        }
    }

    /// Returns a tuple of mutable slice references, the sum of which with len equal to the
    /// higher multiple of `rhs`.
    /// <div class="warning">
    ///
    /// Being these references, [`Self::advance()`] has to be called when done with the mutation
    /// in order to move the iterator.
    /// </div>
    #[inline]
    fn get_mut_slice_multiple_of<'a>(
        &mut self,
        rhs: usize,
    ) -> Option<<<Self::Buffer as OneRB>::Storage as StorageComponent>::SliceOutputMut<'a>> {
        let avail = self.available();

        unsafe {
            match avail.unchecked_sub(avail % rhs) {
                0 => None,
                avail => self.get_mut_slice_exact(avail),
            }
        }
    }
}

pub(crate) trait PrivateORBIterator {
    type _Buffer: OneRB;

    fn buffer(&self) -> &Self::_Buffer;
    fn _available(&mut self) -> usize;
    fn cached_avail(&self) -> usize;
    fn set_cached_avail(&mut self, avail: usize);
    fn _index(&self) -> usize;
    fn set_local_index(&mut self, index: usize);
    /// Sets the global index of this iterator.
    fn set_atomic_index(&self, index: usize);

    /// Returns the global index of successor.
    fn succ_index(&self) -> usize;

    #[inline]
    unsafe fn _advance(&mut self, count: usize) {
        unsafe { self.advance_local(count) };

        self.set_atomic_index(self._index());
    }

    #[inline]
    unsafe fn advance_local(&mut self, count: usize) {
        self.set_local_index(unsafe { self._index().unchecked_add(count) });

        if self._index() >= self.buffer().len() {
            self.set_local_index(unsafe { self._index().unchecked_sub(self.buffer().len()) });
        }

        self.set_cached_avail(self.cached_avail().saturating_sub(count));
    }

    /// Checks whether the current index can be returned
    #[inline]
    fn check(&mut self, count: usize) -> bool {
        self.cached_avail() >= count || self._available() >= count
    }

    /// Returns Some(current element), if `check()` returns `true`, else None
    #[inline]
    fn next(&mut self) -> Option<<Self::_Buffer as OneRB>::Item> {
        self.check(1).then(|| unsafe {
            let ret = self.buffer().storage()._index(self._index()).take_inner();

            self._advance(1);

            ret
        })
    }

    /// Returns Some(current element), if `check()` returns `true`, else None. The value is duplicated.
    #[inline]
    fn next_duplicate(&mut self) -> Option<<Self::_Buffer as OneRB>::Item> {
        self.check(1).then(|| unsafe {
            let ret = self
                .buffer()
                .storage()
                ._index(self._index())
                .inner_duplicate();

            self._advance(1);

            ret
        })
    }

    /// Returns Some(&UnsafeSyncCell<current element>), if `check()` returns `true`, else None
    #[inline]
    fn next_ref<'a>(&mut self) -> Option<&'a <Self::_Buffer as OneRB>::Item> {
        unsafe {
            self.check(1)
                .then(|| self.buffer().storage()._index(self._index()).inner_ref())
        }
    }

    /// Returns Some(&UnsafeSyncCell<current element>), if `check()` returns `true`, else None
    #[inline]
    fn next_ref_mut<'a>(&mut self) -> Option<&'a mut <Self::_Buffer as OneRB>::Item> {
        unsafe {
            self.check(1).then(|| {
                self.buffer()
                    .storage()
                    ._index(self._index())
                    .inner_ref_mut()
            })
        }
    }

    /// As next_ref_mut, but can be used for initialisation of inner MaybeUninit.
    #[inline]
    fn next_ref_mut_init(&mut self) -> Option<*mut <Self::_Buffer as OneRB>::Item> {
        self.check(1)
            .then(|| self.buffer().storage()._index(self._index()).as_mut_ptr())
    }
}

pub(crate) mod iter_macros {
    macro_rules! private_impl {
        () => {
            #[inline]
            #[allow(refining_impl_trait)]
            fn buffer(&self) -> &B {
                &self.inner.buffer
            }

            #[inline]
            fn _index(&self) -> usize {
                self.inner.index
            }
            #[inline]
            fn set_local_index(&mut self, index: usize) {
                self.inner.index = index;
            }

            #[inline]
            fn cached_avail(&self) -> usize {
                self.inner.cached_avail
            }
            #[inline]
            fn set_cached_avail(&mut self, avail: usize) {
                self.inner.cached_avail = avail;
            }
        };
    }

    pub(crate) use private_impl;
}
