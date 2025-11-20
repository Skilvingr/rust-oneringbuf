use core::marker::PhantomData;
use core::task::Waker;

use crate::iterators::ConsIter;
use crate::iterators::async_iterators::async_macros::gen_common_futs_fn;
use crate::iterators::async_iterators::{AsyncIterator, ORBFuture};
use crate::iterators::iterator_trait::{ORBIterator, PrivateORBIterator};
use crate::iterators::util_macros::delegate;
use crate::iters_components::async_iters::AsyncIterComp;
use crate::ring_buffer::OneRB;
use crate::ring_buffer::storage_components::StorageComponent;
use crate::ring_buffer::wrappers::refs::IntoRef;

#[doc = r##"
Async version of [`ConsIter`].
"##]
pub struct AsyncConsIter<B: IntoRef + OneRB<Iters: AsyncIterComp>> {
    inner: ConsIter<B>,
}
unsafe impl<B: IntoRef + OneRB<Iters: AsyncIterComp>> Sync for AsyncConsIter<B> {}
unsafe impl<B: IntoRef + OneRB<Iters: AsyncIterComp>> Send for AsyncConsIter<B> {}

impl<'buf, B: IntoRef + OneRB<Iters: AsyncIterComp>> AsyncIterator<'buf> for AsyncConsIter<B> {
    type I = ConsIter<B>;

    fn register_waker(&self, waker: &Waker) {
        self.inner.buffer().iters().register_cons_waker(waker);
    }

    fn take_waker(&self) -> Option<Waker> {
        self.inner.buffer().iters().take_cons_waker()
    }

    fn wake_next(&self) {
        self.inner.buffer().iters().wake_prod();
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

impl<'buf, B: IntoRef + OneRB<Iters: AsyncIterComp>> AsyncConsIter<B> {
    delegate!(ConsIter, pub fn reset_index(&(mut) self));

    /// Async version of [`ConsIter::peek_ref`].
    pub fn peek_ref<'b>(&'b mut self) -> ORBFuture<'buf, 'b, Self, (), &'b B::Item, true> {
        #[inline]
        fn f<'b, B: IntoRef + OneRB<Iters: AsyncIterComp>>(
            s: &mut AsyncConsIter<B>,
            _: &mut (),
        ) -> Option<&'b B::Item> {
            s.inner_mut().peek_ref()
        }

        ORBFuture {
            iter: self,
            p: Some(()),
            f_r: Some(f),
            f_m: None,
            phantom: PhantomData,
        }
    }

    /// Async version of [`ConsIter::peek_slice`].
    pub fn peek_slice<'b>(
        &'b mut self,
        count: usize,
    ) -> ORBFuture<'buf, 'b, Self, usize, <B::Storage as StorageComponent>::SliceOutput<'b>, true>
    {
        #[inline]
        fn f<'b, B: IntoRef + OneRB<Iters: AsyncIterComp>>(
            s: &mut AsyncConsIter<B>,
            count: &mut usize,
        ) -> Option<<B::Storage as StorageComponent>::SliceOutput<'b>> {
            s.inner_mut().peek_slice(*count)
        }

        ORBFuture {
            iter: self,
            p: Some(count),
            f_r: Some(f),
            f_m: None,
            phantom: PhantomData,
        }
    }

    /// Async version of [`ConsIter::peek_available`].
    pub fn peek_available<'b>(
        &'b mut self,
    ) -> ORBFuture<'buf, 'b, Self, (), <B::Storage as StorageComponent>::SliceOutput<'b>, true>
    {
        #[inline]
        fn f<'b, B: IntoRef + OneRB<Iters: AsyncIterComp>>(
            s: &mut AsyncConsIter<B>,
            _: &mut (),
        ) -> Option<<B::Storage as StorageComponent>::SliceOutput<'b>> {
            s.inner_mut().peek_available()
        }

        ORBFuture {
            iter: self,
            p: Some(()),
            f_r: Some(f),
            f_m: None,
            phantom: PhantomData,
        }
    }

    /// Async version of [`ConsIter::pop`].
    /// # Safety
    /// See above.
    pub fn pop<'b>(&'b mut self) -> ORBFuture<'buf, 'b, Self, (), B::Item, true> {
        #[inline]
        fn f<B: IntoRef + OneRB<Iters: AsyncIterComp>>(
            s: &mut AsyncConsIter<B>,
            _: &mut (),
        ) -> Option<B::Item> {
            s.inner_mut().pop()
        }

        ORBFuture {
            iter: self,
            p: Some(()),
            f_r: Some(f),
            f_m: None,
            phantom: PhantomData,
        }
    }

    /// Async version of [`ConsIter::pop_move`].
    /// # Safety
    /// See above.
    pub unsafe fn pop_move<'b>(&'b mut self) -> ORBFuture<'buf, 'b, Self, (), B::Item, true> {
        #[inline]
        fn f<B: IntoRef + OneRB<Iters: AsyncIterComp>>(
            s: &mut AsyncConsIter<B>,
            _: &mut (),
        ) -> Option<B::Item> {
            unsafe { s.inner_mut().pop_move() }
        }

        ORBFuture {
            iter: self,
            p: Some(()),
            f_r: Some(f),
            f_m: None,
            phantom: PhantomData,
        }
    }

    /// Async version of [`ConsIter::copy_item`].
    pub fn copy_item<'b>(
        &'b mut self,
        dst: &'b mut B::Item,
    ) -> ORBFuture<'buf, 'b, Self, &'b mut B::Item, (), true>
    where
        B::Item: Copy,
    {
        #[inline]
        fn f<B: IntoRef + OneRB<Iters: AsyncIterComp>>(
            s: &mut AsyncConsIter<B>,
            dst: &mut &mut B::Item,
        ) -> Option<()>
        where
            B::Item: Copy,
        {
            s.inner_mut().copy_item(*dst)
        }

        ORBFuture {
            iter: self,
            p: Some(dst),
            f_r: Some(f),
            f_m: None,
            phantom: PhantomData,
        }
    }

    /// Async version of [`ConsIter::clone_item`].
    pub fn clone_item<'b>(
        &'b mut self,
        dst: &'b mut B::Item,
    ) -> ORBFuture<'buf, 'b, Self, &'b mut B::Item, (), true>
    where
        B::Item: Clone,
    {
        #[inline]
        fn f<B: IntoRef + OneRB<Iters: AsyncIterComp>>(
            s: &mut AsyncConsIter<B>,
            dst: &mut &mut B::Item,
        ) -> Option<()>
        where
            B::Item: Clone,
        {
            s.inner_mut().clone_item(*dst)
        }

        ORBFuture {
            iter: self,
            p: Some(dst),
            f_r: Some(f),
            f_m: None,
            phantom: PhantomData,
        }
    }

    /// Async version of [`ConsIter::copy_slice`].
    pub fn copy_slice<'b>(
        &'b mut self,
        dst: &'b mut [B::Item],
    ) -> ORBFuture<'buf, 'b, Self, &'b mut [B::Item], (), true>
    where
        B::Item: Copy,
    {
        #[inline]
        fn f<B: IntoRef + OneRB<Iters: AsyncIterComp>>(
            s: &mut AsyncConsIter<B>,
            dst: &mut &mut [B::Item],
        ) -> Option<()>
        where
            B::Item: Copy,
        {
            s.inner_mut().copy_slice(dst)
        }

        ORBFuture {
            iter: self,
            p: Some(dst),
            f_r: Some(f),
            f_m: None,
            phantom: PhantomData,
        }
    }

    /// Async version of [`ConsIter::clone_slice`].
    pub fn clone_slice<'b>(
        &'b mut self,
        dst: &'b mut [B::Item],
    ) -> ORBFuture<'buf, 'b, Self, &'b mut [B::Item], (), true>
    where
        B::Item: Clone,
    {
        #[inline]
        fn f<B: IntoRef + OneRB<Iters: AsyncIterComp>>(
            s: &mut AsyncConsIter<B>,
            dst: &mut &mut [B::Item],
        ) -> Option<()>
        where
            B::Item: Clone,
        {
            s.inner_mut().clone_slice(dst)
        }

        ORBFuture {
            iter: self,
            p: Some(dst),
            f_r: Some(f),
            f_m: None,
            phantom: PhantomData,
        }
    }
}
