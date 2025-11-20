use crate::iterators::iterator_trait::{ORBIterator, PrivateORBIterator};
use crate::iterators::private_impl;
use crate::iterators::sync_iterators::Iter;
#[allow(unused_imports)]
use crate::iterators::sync_iterators::detached::Detached;
use crate::ring_buffer::OneRB;
use crate::ring_buffer::wrappers::refs::IntoRef;
use crate::ring_buffer::{SharedRB, iters_components::PIterComponent};
use crate::storage_components::PStorageComponent;
#[cfg(feature = "async")]
use crate::{
    iterators::{AsyncWorkIter, async_iterators::AsyncIterator},
    iters_components::async_iters::AsyncIterComp,
};

#[doc = r##"
Iterator used to mutate elements in-place.

<div class="warning">

This iterator returns mutable references to data stored within the buffer.
Thus, as stated in the docs below, [`Self::advance`] has to be called when done with the mutation
in order to move the iterator.
</div>

[`Self::advance`] updates a global iterator, which is read by the consumer to decide if it can move on.
To avoid this [`Detached`] can be obtained by calling [`Self::detach`].
"##]
#[repr(transparent)]
pub struct WorkIter<B: IntoRef + OneRB> {
    pub(crate) inner: Iter<B>,
}

unsafe impl<B: IntoRef + OneRB + SharedRB> Send for WorkIter<B> {}

impl<B: IntoRef + OneRB<Item = T>, T> PrivateORBIterator for WorkIter<B> {
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
        self.inner.buffer.iters().set_work_index(index);
    }

    #[inline]
    fn succ_index(&self) -> usize {
        self.inner.buffer.iters().prod_index()
    }

    private_impl!();
}

impl<B: IntoRef + OneRB<Item = T>, T> ORBIterator for WorkIter<B> {
    type Item = T;
    type Buffer = B;
}

#[cfg(feature = "async")]
impl<B: IntoRef + OneRB<Iters: AsyncIterComp>> WorkIter<B> {
    pub fn into_async(self) -> AsyncWorkIter<B> {
        AsyncIterator::from_sync(self)
    }
}

impl<B: IntoRef + OneRB<Item = T>, T> WorkIter<B> {
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
}
