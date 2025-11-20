use crate::OneRB;
use crate::StorageComponent;
#[allow(unused_imports)]
use crate::iterators::WorkIter;
use crate::iterators::iterator_trait::ORBIterator;
use crate::iterators::util_macros::delegate;
use crate::iterators::util_macros::muncher;
#[cfg(feature = "async")]
use crate::{
    iterators::{AsyncDetached, async_iterators::AsyncIterator},
    iters_components::async_iters::AsyncIterComp,
};

#[doc = r##"
Detached iterator: does not update the atomic index when advancing.

This makes it possible to explore available data back and forth, putting other iterators on hold.

A typical use case of this structure is to search something amidst produced data, aligning the detached
iterator to a suitable index and then returning to a normal iterator.

This struct can only be created by [`detaching`](ORBIterator::detach) an iterator.

When done worker iterator can be re-obtained via [`Self::attach`].

Note that, in order to avoid buffer saturation, the global index can be synced with [`Self::sync_index`];
this synchronises indices making the consumer iterator able to move on.

<div class="warning">

As [`WorkIter`], this iterator returns mutable references to data stored within the buffer.
Thus, as stated in the docs written for the former, [`Self::advance`] has to be called when done with the mutation
in order to move the iterator.
</div>
"##]
#[repr(transparent)]
pub struct Detached<I: ORBIterator> {
    inner: I,
}

unsafe impl<I: ORBIterator> Send for Detached<I> {}

#[cfg(feature = "async")]
impl<'buf, I: ORBIterator<Buffer: OneRB<Iters: AsyncIterComp>>> Detached<I> {
    pub fn into_async<AS: AsyncIterator<'buf, I = I>>(self) -> AsyncDetached<'buf, AS> {
        AsyncDetached::from_iter(AsyncIterator::from_sync(self.inner))
    }
}

impl<T, I: ORBIterator<Item = T>> Detached<I> {
    /// Creates a [`Self`] from an iterator.
    #[inline]
    pub(crate) fn from_iter(iter: I) -> Detached<I> {
        Self { inner: iter }
    }

    /// Attaches and yields the iterator.
    #[inline]
    pub fn attach(self) -> I {
        self.sync_index();
        self.inner
    }

    #[inline]
    fn inner(&self) -> &I {
        &self.inner
    }
    #[inline]
    fn inner_mut(&mut self) -> &mut I {
        &mut self.inner
    }

    delegate!(ORBIterator (inline), pub fn available(&(mut) self) -> usize);
    delegate!(ORBIterator (inline), pub fn wait_for(&(mut) self, count: usize));
    delegate!(ORBIterator (inline), pub fn index(&self) -> usize);
    delegate!(ORBIterator (inline), pub fn buf_len(&self) -> usize);

    /// Sets the *local* index. To sync the atomic index, use [`Self::sync_index`].
    ///
    /// # Safety
    /// Index must always be between consumer and producer.
    #[inline]
    pub unsafe fn set_index(&mut self, index: usize) {
        self.inner.set_local_index(index);
    }

    /// Resets the *local* index of the iterator. I.e., moves the iterator to the location occupied by its successor.
    /// To sync the atomic index, use [`Self::sync_index`].
    #[inline]
    pub fn reset_index(&mut self) {
        let new_idx = self.inner.succ_index();
        self.inner.set_local_index(new_idx);
    }

    /// Advances the iterator as in [`ORBIterator::advance()`], but does not modify the atomic counter,
    /// making the change local.
    ///
    /// # Safety
    /// See [`ORBIterator::advance`].
    #[inline]
    pub unsafe fn advance(&mut self, count: usize) {
        unsafe { self.inner.advance_local(count) };
    }

    /// Goes back, wrapping if necessary.
    ///
    /// # Safety
    /// Index must always be between consumer and producer.
    pub unsafe fn go_back(&mut self, count: usize) {
        let idx = self.inner.index();

        self.inner.set_local_index(match idx < count {
            true => unsafe { self.inner.buf_len().unchecked_sub(count).unchecked_sub(idx) },
            false => unsafe { idx.unchecked_sub(count) },
        });

        let cached_avail = self.inner.cached_avail();
        self.inner
            .set_cached_avail(unsafe { cached_avail.unchecked_add(count) });
    }

    delegate!(ORBIterator (inline), pub fn prod_index(&self) -> usize);
    delegate!(ORBIterator (inline), pub fn work_index(&self) -> usize);
    delegate!(ORBIterator (inline), pub fn cons_index(&self) -> usize);

    delegate!(ORBIterator (inline), pub fn get_mut(&(mut) self) -> Option<&'_ mut T>);
    delegate!(ORBIterator (inline), pub fn get_mut_slice_exact(&(mut) self, count: usize) -> Option<<<I::Buffer as OneRB>::Storage as StorageComponent>::SliceOutputMut<'_>>);
    delegate!(ORBIterator (inline), pub fn get_mut_slice_avail(&(mut) self) -> Option<<<I::Buffer as OneRB>::Storage as StorageComponent>::SliceOutputMut<'_>>);
    delegate!(ORBIterator (inline), pub fn get_mut_slice_multiple_of(&(mut) self, rhs: usize) -> Option<<<I::Buffer as OneRB>::Storage as StorageComponent>::SliceOutputMut<'_>>);

    /// Synchronises the underlying atomic index with the local index. I.e. let the consumer iterator
    /// advance.
    #[inline]
    pub fn sync_index(&self) {
        self.inner.set_atomic_index(self.inner.index());
    }
}
