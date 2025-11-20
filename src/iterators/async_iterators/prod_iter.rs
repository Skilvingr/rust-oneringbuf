use core::marker::PhantomData;
use core::task::Waker;

use crate::iterators::ProdIter;
use crate::iterators::async_iterators::async_macros::gen_common_futs_fn;
use crate::iterators::async_iterators::{AsyncIterator, ORBFuture};
use crate::iterators::iterator_trait::{ORBIterator, PrivateORBIterator};
use crate::iters_components::async_iters::AsyncIterComp;
use crate::ring_buffer::OneRB;
use crate::ring_buffer::storage_components::StorageComponent;
use crate::ring_buffer::wrappers::refs::IntoRef;

#[doc = r##"
Async version of [`ProdIter`].
"##]
pub struct AsyncProdIter<B: IntoRef + OneRB<Iters: AsyncIterComp>> {
    inner: ProdIter<B>,
}
unsafe impl<B: IntoRef + OneRB<Iters: AsyncIterComp>> Sync for AsyncProdIter<B> {}
unsafe impl<B: IntoRef + OneRB<Iters: AsyncIterComp>> Send for AsyncProdIter<B> {}

impl<'buf, B: IntoRef + OneRB<Iters: AsyncIterComp>> AsyncIterator<'buf> for AsyncProdIter<B> {
    type I = ProdIter<B>;

    fn register_waker(&self, waker: &Waker) {
        self.inner.buffer().iters().register_prod_waker(waker);
    }

    fn take_waker(&self) -> Option<Waker> {
        self.inner.buffer().iters().take_prod_waker()
    }

    fn wake_next(&self) {
        self.inner.buffer().iters().wake_middle_iter();
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

impl<'buf, B: IntoRef + OneRB<Iters: AsyncIterComp>> AsyncProdIter<B> {
    /// Async version of [`ProdIter::push`].
    pub fn push<'b>(&'b mut self, item: B::Item) -> ORBFuture<'buf, 'b, Self, B::Item, (), false> {
        #[inline]
        fn f<B: IntoRef + OneRB<Iters: AsyncIterComp>>(
            s: &mut AsyncProdIter<B>,
            item: B::Item,
        ) -> Result<(), B::Item> {
            s.inner_mut().push(item)
        }

        ORBFuture {
            iter: self,
            p: Some(item),
            f_r: None,
            f_m: Some(f),
            phantom: PhantomData,
        }
    }

    /// Async version of [`ProdIter::push_slice`].
    pub fn push_slice<'b>(
        &'b mut self,
        slice: &'b [B::Item],
    ) -> ORBFuture<'buf, 'b, Self, &'b [B::Item], (), true>
    where
        B::Item: Copy,
    {
        #[inline]
        fn f<B: IntoRef + OneRB<Iters: AsyncIterComp>>(
            s: &mut AsyncProdIter<B>,
            slice: &mut &[B::Item],
        ) -> Option<()>
        where
            B::Item: Copy,
        {
            let ret = s.inner_mut().push_slice(slice);
            s.wake_next();
            ret
        }

        ORBFuture {
            iter: self,
            p: Some(slice),
            f_r: Some(f),
            f_m: None,
            phantom: PhantomData,
        }
    }

    /// Async version of [`ProdIter::push_slice_clone`].
    pub fn push_slice_clone<'b>(
        &'b mut self,
        slice: &'b [B::Item],
    ) -> ORBFuture<'buf, 'b, Self, &'b [B::Item], (), true>
    where
        B::Item: Clone,
    {
        #[inline]
        fn f<B: IntoRef + OneRB<Iters: AsyncIterComp>>(
            s: &mut AsyncProdIter<B>,
            slice: &mut &[B::Item],
        ) -> Option<()>
        where
            B::Item: Clone,
        {
            s.inner_mut().push_slice_clone(slice)
        }

        ORBFuture {
            iter: self,
            p: Some(slice),
            f_r: Some(f),
            f_m: None,
            phantom: PhantomData,
        }
    }

    /// Async version of [`ProdIter::get_next_item_mut`].
    /// # Safety
    /// Same as [`ProdIter::get_next_item_mut`].
    pub unsafe fn get_next_item_mut<'b>(
        &'b mut self,
    ) -> ORBFuture<'buf, 'b, Self, (), &'b mut B::Item, true> {
        #[inline]
        fn f<'b, B: IntoRef + OneRB<Iters: AsyncIterComp>>(
            s: &mut AsyncProdIter<B>,
            _: &mut (),
        ) -> Option<&'b mut B::Item> {
            unsafe { s.inner_mut().get_next_item_mut() }
        }

        ORBFuture {
            iter: self,
            p: Some(()),
            f_r: Some(f),
            f_m: None,
            phantom: PhantomData,
        }
    }

    /// Async version of [`ProdIter::get_next_item_mut_init`].
    pub fn get_next_item_mut_init<'b>(
        &'b mut self,
    ) -> ORBFuture<'buf, 'b, Self, (), *mut B::Item, true> {
        #[inline]
        fn f<B: IntoRef + OneRB<Iters: AsyncIterComp>>(
            s: &mut AsyncProdIter<B>,
            _: &mut (),
        ) -> Option<*mut B::Item> {
            s.inner_mut().get_next_item_mut_init()
        }

        ORBFuture {
            iter: self,
            p: Some(()),
            f_r: Some(f),
            f_m: None,
            phantom: PhantomData,
        }
    }

    /// Async version of [`ProdIter::get_next_slices_mut`].
    /// # Safety
    /// See above.
    pub unsafe fn get_next_slices_mut<'b>(
        &'b mut self,
        count: usize,
    ) -> ORBFuture<'buf, 'b, Self, usize, <B::Storage as StorageComponent>::SliceOutputMut<'b>, true>
    {
        #[inline]
        fn f<'b, B: IntoRef + OneRB<Iters: AsyncIterComp>>(
            s: &mut AsyncProdIter<B>,
            count: &mut usize,
        ) -> Option<<B::Storage as StorageComponent>::SliceOutputMut<'b>> {
            unsafe { s.inner_mut().get_next_slices_mut(*count) }
        }

        ORBFuture {
            iter: self,
            p: Some(count),
            f_r: Some(f),
            f_m: None,
            phantom: PhantomData,
        }
    }
}
