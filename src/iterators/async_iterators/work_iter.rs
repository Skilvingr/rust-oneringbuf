use crate::iterators::async_iterators::async_macros::gen_common_futs_fn;
use crate::iterators::async_iterators::{AsyncIterator, ORBFuture};
use crate::iterators::iterator_trait::PrivateORBIterator;
use crate::iterators::util_macros::delegate;
use crate::iters_components::async_iters::AsyncIterComp;
use crate::ring_buffer::OneRB;
use crate::ring_buffer::storage_components::StorageComponent;
use crate::ring_buffer::wrappers::refs::IntoRef;
#[allow(unused_imports)]
use crate::{ORBIterator, iterators::WorkIter};
use core::marker::PhantomData;
use core::task::Waker;

#[doc = r##"
Async version of [`WorkIter`].
"##]
pub struct AsyncWorkIter<B: IntoRef + OneRB<Iters: AsyncIterComp>> {
    pub(crate) inner: WorkIter<B>,
}
unsafe impl<B: IntoRef + OneRB<Iters: AsyncIterComp>> Sync for AsyncWorkIter<B> {}
unsafe impl<B: IntoRef + OneRB<Iters: AsyncIterComp>> Send for AsyncWorkIter<B> {}

impl<'buf, B: IntoRef + OneRB<Iters: AsyncIterComp>> AsyncIterator<'buf> for AsyncWorkIter<B> {
    type I = WorkIter<B>;

    fn register_waker(&self, waker: &Waker) {
        self.inner.buffer().iters().register_work_waker(waker);
    }

    fn take_waker(&self) -> Option<Waker> {
        self.inner.buffer().iters().take_work_waker()
    }

    fn wake_next(&self) {
        self.inner.buffer().iters().wake_cons();
    }

    #[inline]
    fn inner(&self) -> &Self::I {
        &self.inner
    }
    #[inline]
    fn inner_mut(&mut self) -> &mut Self::I {
        &mut self.inner
    }
    fn into_sync(self) -> Self::I {
        self.inner
    }
    fn from_sync(iter: Self::I) -> Self {
        Self { inner: iter }
    }

    gen_common_futs_fn!();
}

impl<'buf, B: IntoRef + OneRB<Iters: AsyncIterComp>> AsyncWorkIter<B> {
    delegate!(WorkIter, pub fn reset_index(&(mut) self));
}
