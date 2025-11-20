#![cfg(any(feature = "async", doc))]

//! Async iterators.

use crate::ORBIterator;
use crate::OneRB;
use crate::StorageComponent;
use crate::iterators::async_iterators::detached::AsyncDetached;
use crate::iterators::util_macros::delegate;
use crate::iterators::util_macros::muncher;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};

pub(crate) mod cons_iter;
pub(crate) mod detached;
pub(crate) mod prod_iter;
pub(crate) mod work_iter;

/// Trait implemented by async iterators.
pub trait AsyncIterator<'buf> {
    type I: ORBIterator;

    fn register_waker(&self, waker: &Waker);
    fn take_waker(&self) -> Option<Waker>;
    fn wake_next(&self);

    fn inner(&self) -> &Self::I;
    fn inner_mut(&mut self) -> &mut Self::I;

    fn into_sync(self) -> Self::I;

    fn from_sync(iter: Self::I) -> Self;

    fn detach(self) -> AsyncDetached<'buf, Self>
    where
        Self: Sized,
    {
        AsyncDetached::from_iter(self)
    }

    unsafe fn advance(&mut self, count: usize);

    fn get_mut<'b>(
        &'b mut self,
    ) -> ORBFuture<'buf, 'b, Self, (), &'b mut <Self::I as ORBIterator>::Item, true>;

    fn get_mut_slice_exact<'b>(
        &'b mut self,
        count: usize,
    ) -> ORBFuture<
        'buf,
        'b,
        Self,
        usize,
        <<<Self::I as ORBIterator>::Buffer as OneRB>::Storage as StorageComponent>::SliceOutputMut<
            'b,
        >,
        true,
    >;

    fn get_mut_slice_avail<'b>(
        &'b mut self,
    ) -> ORBFuture<
        'buf,
        'b,
        Self,
        (),
        <<<Self::I as ORBIterator>::Buffer as OneRB>::Storage as StorageComponent>::SliceOutputMut<
            'b,
        >,
        true,
    >;

    fn get_mut_slice_multiple_of<'b>(
        &'b mut self,
        count: usize,
    ) -> ORBFuture<
        'buf,
        'b,
        Self,
        usize,
        <<<Self::I as ORBIterator>::Buffer as OneRB>::Storage as StorageComponent>::SliceOutputMut<
            'b,
        >,
        true,
    >;

    delegate!(ORBIterator, fn prod_index(&self) -> usize);
    delegate!(ORBIterator, fn work_index(&self) -> usize);
    delegate!(ORBIterator, fn cons_index(&self) -> usize);
    delegate!(ORBIterator, fn alive_iters(&self) -> u8);
    delegate!(ORBIterator, fn index(&self) -> usize);
    delegate!(ORBIterator, fn available(&(mut) self) -> usize);
}

/// Future returned by methods in async iterators.
pub struct ORBFuture<'buf, 'a, I, P, O, const R: bool>
where
    I: AsyncIterator<'buf> + ?Sized,
{
    iter: &'a mut I,
    p: Option<P>,
    f_r: Option<fn(&mut I, &mut P) -> Option<O>>,
    f_m: Option<fn(&mut I, P) -> Result<O, P>>,
    phantom: PhantomData<&'buf ()>,
}

impl<'buf, 'a, I: AsyncIterator<'buf>, P, O, const R: bool> Unpin
    for ORBFuture<'buf, 'a, I, P, O, R>
{
}

impl<'buf, 'a, I: AsyncIterator<'buf>, P, O, const R: bool> Future
    for ORBFuture<'buf, 'a, I, P, O, R>
{
    type Output = Option<O>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let f_r = self.f_r.take();
        let f_m = self.f_m.take();
        let p = self.p.take();

        let res = if R {
            let mut p = p.unwrap();
            let ret = f_r.as_ref().unwrap()(self.iter, &mut p);
            ret.ok_or(p)
        } else {
            f_m.as_ref().unwrap()(self.iter, p.unwrap())
        };

        match res {
            Ok(r) => {
                return Poll::Ready(Some(r));
            }
            Err(p) => {
                self.f_r = f_r;
                self.f_m = f_m;
                self.p = Some(p);
            }
        }

        self.iter.register_waker(cx.waker());
        Poll::Pending
    }
}

pub(crate) mod async_macros {
    macro_rules! gen_common_futs_fn {
        () => {
            /// Async version of [`ORBIterator::get_mut`].
            fn get_mut<'b>(
                &'b mut self,
            ) -> ORBFuture<'buf, 'b, Self, (), &'b mut B::Item, true> {
                fn f<'buf, 'b, I: AsyncIterator<'buf, I: ORBIterator<Item = T>>, T>(
                    s: &mut I,
                    _: &mut (),
                ) -> Option<&'b mut T> {
                    s.inner_mut().get_mut()
                }

                ORBFuture {
                    iter: self,
                    p: Some(()),
                    f_r: Some(f),
                    f_m: None,
                    phantom: PhantomData,
                }
            }

            /// Async version of [`ORBIterator::get_mut_slice_exact`].
            fn get_mut_slice_exact<'b>(
                &'b mut self,
                count: usize,
            ) -> ORBFuture<
                'buf,
                'b,
                Self,
                usize,
                <B::Storage as StorageComponent>::SliceOutputMut<'b>,
                true,
            > {
                fn f<'buf, 'b, I: AsyncIterator<'buf, I: ORBIterator<Item = T>>, T>(
                    s: &mut I,
                    count: &mut usize,
                ) -> Option<
                    <<<I::I as ORBIterator>::Buffer as OneRB>::Storage as StorageComponent>::SliceOutputMut<
                        'b,
                    >,
                > {
                    s.inner_mut().get_mut_slice_exact(*count)
                }

                ORBFuture {
                    iter: self,
                    p: Some(count),
                    f_r: Some(f),
                    f_m: None,
                    phantom: PhantomData,
                }
            }

            /// Async version of [`ORBIterator::get_mut_slice_avail`].
            fn get_mut_slice_avail<'b>(
                &'b mut self,
            ) -> ORBFuture<
                'buf,
                'b,
                Self,
                (),
                <B::Storage as StorageComponent>::SliceOutputMut<'b>,
                true,
            > {
                fn f<'buf, 'b, I: AsyncIterator<'buf, I: ORBIterator<Item = T>>, T>(
                    s: &mut I,
                    _: &mut (),
                ) -> Option<
                    <<<I::I as ORBIterator>::Buffer as OneRB>::Storage as StorageComponent>::SliceOutputMut<
                        'b,
                    >,
                > {
                    s.inner_mut().get_mut_slice_avail()
                }

                ORBFuture {
                    iter: self,
                    p: Some(()),
                    f_r: Some(f),
                    f_m: None,
                    phantom: PhantomData,
                }
            }

            /// Async version of [`ORBIterator::get_mut_slice_multiple_of`].
            fn get_mut_slice_multiple_of<'b>(
                &'b mut self,
                count: usize,
            ) -> ORBFuture<
                'buf,
                'b,
                Self,
                usize,
                <B::Storage as StorageComponent>::SliceOutputMut<'b>,
                true,
            > {
                fn f<'buf, 'b, I: AsyncIterator<'buf, I: ORBIterator<Item = T>>, T>(
                    s: &mut I,
                    count: &mut usize,
                ) -> Option<
                    <<<I::I as ORBIterator>::Buffer as OneRB>::Storage as StorageComponent>::SliceOutputMut<
                        'b,
                    >,
                > {
                    s.inner_mut().get_mut_slice_multiple_of(*count)
                }

                ORBFuture {
                    iter: self,
                    p: Some(count),
                    f_r: Some(f),
                    f_m: None,
                    phantom: PhantomData,
                }
            }

            unsafe fn advance(&mut self, count: usize) {
                unsafe {
                    self.inner.advance(count);
                }
                self.wake_next();
            }
        };
    }

    pub(crate) use gen_common_futs_fn;
}
