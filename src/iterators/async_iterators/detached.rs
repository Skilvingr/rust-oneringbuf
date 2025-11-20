use crate::OneRB;
use crate::StorageComponent;
use crate::iterators::async_iterators::AsyncIterator;
use core::marker::PhantomData;

use crate::ORBIterator;
#[allow(unused_imports)]
use crate::iterators::Detached;
use crate::iterators::async_iterators::ORBFuture;
use crate::iterators::iterator_trait::PrivateORBIterator;
use crate::iterators::util_macros::delegate;
use crate::iterators::util_macros::muncher;

#[doc = r##"
Async version of [`Detached`].
"##]
#[repr(transparent)]
pub struct AsyncDetached<'buf, I: AsyncIterator<'buf>> {
    inner: I,
    phantom: PhantomData<&'buf ()>,
}

unsafe impl<'buf, I: AsyncIterator<'buf>> Sync for AsyncDetached<'buf, I> {}
unsafe impl<'buf, I: AsyncIterator<'buf>> Send for AsyncDetached<'buf, I> {}

impl<'buf, I: AsyncIterator<'buf>> AsyncDetached<'buf, I> {
    /// Creates [`Self`] from an [`AsyncWorkIter`].
    pub(crate) fn from_iter(iter: I) -> AsyncDetached<'buf, I> {
        Self {
            inner: iter,
            phantom: PhantomData,
        }
    }

    fn inner_mut(&mut self) -> &mut I {
        &mut self.inner
    }

    /// Same as [`Detached::attach`].
    pub fn attach(self) -> I {
        self.sync_index();
        self.inner
    }

    /// Same as [`Detached::sync_index`].
    pub fn sync_index(&self) {
        self.inner.inner().set_atomic_index(self.inner.index())
    }

    /// Same as [`Detached::advance`].
    ///
    /// # Safety
    /// Same as [`Detached::advance`].
    pub unsafe fn advance(&mut self, count: usize) {
        unsafe { self.inner.inner_mut().advance_local(count) };
    }

    /// Same as [`Detached::go_back`].
    ///
    /// # Safety
    /// Same as [`Detached::go_back`].
    pub unsafe fn go_back(&mut self, count: usize) {
        let idx = self.inner.inner_mut().index();
        let buf_len = self.inner.inner_mut().buf_len();

        self.inner.inner_mut().set_local_index(match idx < count {
            true => unsafe { buf_len.unchecked_sub(count).unchecked_sub(idx) },
            false => unsafe { idx.unchecked_sub(count) },
        });

        let avail = self.inner.inner_mut().cached_avail();
        self.inner
            .inner_mut()
            .set_cached_avail(unsafe { avail.unchecked_add(count) });
    }

    delegate!(
        AsyncIterator (inline),
        pub fn get_mut<'b>(&'b (mut) self) ->
        ORBFuture<'buf, 'b, I, (), &'b mut <I::I as ORBIterator>::Item, true>
    );
    delegate!(
        AsyncIterator (inline),
        pub fn get_mut_slice_exact<'b>(&'b (mut) self, count: usize) ->
        ORBFuture<'buf,'b, I, usize,
            <<<I::I as ORBIterator>::Buffer as OneRB>::Storage as StorageComponent>::SliceOutputMut<'b,>, true
        >
    );
    delegate!(
        AsyncIterator (inline),
        pub fn get_mut_slice_avail<'b>(&'b (mut) self) ->
        ORBFuture<'buf,'b, I, (),
            <<<I::I as ORBIterator>::Buffer as OneRB>::Storage as StorageComponent>::SliceOutputMut<'b,>, true
        >
    );
    delegate!(
        AsyncIterator (inline),
        pub fn get_mut_slice_multiple_of<'b>(&'b (mut) self, count: usize) ->
        ORBFuture<'buf,'b, I, usize,
            <<<I::I as ORBIterator>::Buffer as OneRB>::Storage as StorageComponent>::SliceOutputMut<'b,>, true
        >
    );
}
