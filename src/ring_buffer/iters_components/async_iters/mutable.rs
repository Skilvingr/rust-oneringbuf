use crossbeam_utils::CachePadded;
use futures_util::task::AtomicWaker;

use core::{
    sync::atomic::Ordering::{Acquire, Release},
    task::Waker,
};

#[cfg(all(feature = "alloc", feature = "vmem", unix))]
use crate::storage_components::VmemStorage;
use crate::{
    OneRingBuf,
    iterators::{
        AsyncConsIter, AsyncProdIter, AsyncWorkIter, ConsIter, ProdIter, WorkIter,
        async_iterators::AsyncIterator,
    },
    iters_components::{
        MutIterComp, async_iters::AsyncIterComp, shared_iters::mutable::SharedCompMut,
    },
    ring_buffer::{
        iters_components::{IterComponent, PIterComponent},
        wrappers::refs::non_droppable::NonDroppableRef,
    },
    storage_components::StackStorage,
};
#[cfg(feature = "alloc")]
use crate::{
    ring_buffer::wrappers::refs::droppable::DroppableRef, storage_components::HeapStorage,
};

/// Mutable async iterators component usable in concurrent environments.
pub struct AsyncCompMut {
    inner: SharedCompMut,

    pub(crate) prod_waker: CachePadded<AtomicWaker>,
    pub(crate) work_waker: CachePadded<AtomicWaker>,
    pub(crate) cons_waker: CachePadded<AtomicWaker>,
}

impl MutIterComp for AsyncCompMut {}

impl AsyncCompMut {
    pub const fn default() -> Self {
        Self {
            inner: SharedCompMut::default(),

            prod_waker: CachePadded::new(AtomicWaker::new()),
            work_waker: CachePadded::new(AtomicWaker::new()),
            cons_waker: CachePadded::new(AtomicWaker::new()),
        }
    }
}

impl AsyncIterComp for AsyncCompMut {
    fn wake_middle_iter(&self) {
        self.work_waker.wake();
    }

    fn register_prod_waker(&self, waker: &Waker) {
        self.prod_waker.register(waker);
    }

    fn take_prod_waker(&self) -> Option<Waker> {
        self.prod_waker.take()
    }

    fn wake_prod(&self) {
        self.prod_waker.wake();
    }

    fn register_work_waker(&self, waker: &Waker) {
        self.work_waker.register(waker);
    }

    fn take_work_waker(&self) -> Option<Waker> {
        self.work_waker.take()
    }

    fn wake_work(&self) {
        self.work_waker.wake();
    }

    fn register_cons_waker(&self, waker: &Waker) {
        self.cons_waker.register(waker);
    }

    fn take_cons_waker(&self) -> Option<Waker> {
        self.cons_waker.take()
    }

    fn wake_cons(&self) {
        self.cons_waker.wake();
    }
}

impl PIterComponent for AsyncCompMut {
    #[inline(always)]
    fn middle_iter_idx(&self) -> usize {
        self.work_index()
    }

    #[inline(always)]
    fn drop_iter(&self) -> u8 {
        self.inner.alive_iters.fetch_sub(1, Release)
    }

    #[inline(always)]
    fn acquire_fence(&self) {
        #[cfg(not(feature = "thread_sanitiser"))]
        core::sync::atomic::fence(Acquire);

        // ThreadSanitizer does not support memory fences. To avoid false positive
        // reports use atomic loads for synchronization instead.
        #[cfg(feature = "thread_sanitiser")]
        self.inner.alive_iters.load(Acquire);
    }

    #[inline]
    fn prod_index(&self) -> usize {
        self.inner.prod_idx.load(Acquire)
    }

    #[inline]
    fn work_index(&self) -> usize {
        self.inner.work_idx.load(Acquire)
    }

    #[inline]
    fn cons_index(&self) -> usize {
        self.inner.cons_idx.load(Acquire)
    }

    #[inline]
    fn set_prod_index(&self, index: usize) {
        self.inner.prod_idx.store(index, Release);
    }

    #[inline]
    fn set_work_index(&self, index: usize) {
        self.inner.work_idx.store(index, Release);
    }

    #[inline]
    fn set_cons_index(&self, index: usize) {
        self.inner.cons_idx.store(index, Release);
    }

    fn alive_iters(&self) -> u8 {
        self.inner.alive_iters.load(Acquire)
    }
}

impl IterComponent for AsyncCompMut {}

impl<'buf, T, const N: usize> OneRingBuf<StackStorage<'buf, T, N>, AsyncCompMut> {
    /// Returns three iterators: a Producer, a Worker and a Consumer.
    /// <div class="warning">Available only for mutable buffers.</div>
    pub fn split_async_mut(
        &'buf mut self,
    ) -> (
        AsyncProdIter<OneRingBuf<StackStorage<'buf, T, N>, AsyncCompMut>>,
        AsyncWorkIter<OneRingBuf<StackStorage<'buf, T, N>, AsyncCompMut>>,
        AsyncConsIter<OneRingBuf<StackStorage<'buf, T, N>, AsyncCompMut>>,
    ) {
        let r = NonDroppableRef::from(self);
        (
            AsyncProdIter::from_sync(ProdIter::new(r.clone())),
            AsyncWorkIter::from_sync(WorkIter::new(r.clone())),
            AsyncConsIter::from_sync(ConsIter::new(r)),
        )
    }
}

#[cfg(feature = "alloc")]
impl<T> OneRingBuf<HeapStorage<T>, AsyncCompMut> {
    /// Returns three iterators: a Producer, a Worker and a Consumer.
    /// <div class="warning">Available only for mutable buffers.</div>
    pub fn split_async_mut(
        self,
    ) -> (
        AsyncProdIter<OneRingBuf<HeapStorage<T>, AsyncCompMut>>,
        AsyncWorkIter<OneRingBuf<HeapStorage<T>, AsyncCompMut>>,
        AsyncConsIter<OneRingBuf<HeapStorage<T>, AsyncCompMut>>,
    ) {
        let r = DroppableRef::from(self);
        (
            AsyncProdIter::from_sync(ProdIter::new(r.clone())),
            AsyncWorkIter::from_sync(WorkIter::new(r.clone())),
            AsyncConsIter::from_sync(ConsIter::new(r)),
        )
    }
}

#[cfg(all(feature = "alloc", feature = "vmem", unix))]
impl<T> OneRingBuf<VmemStorage<T>, AsyncCompMut> {
    /// Returns three iterators: a Producer, a Worker and a Consumer.
    /// <div class="warning">Available only for mutable buffers.</div>
    pub fn split_async_mut(
        self,
    ) -> (
        AsyncProdIter<OneRingBuf<VmemStorage<T>, AsyncCompMut>>,
        AsyncWorkIter<OneRingBuf<VmemStorage<T>, AsyncCompMut>>,
        AsyncConsIter<OneRingBuf<VmemStorage<T>, AsyncCompMut>>,
    ) {
        let r = DroppableRef::from(self);
        (
            AsyncProdIter::from_sync(ProdIter::new(r.clone())),
            AsyncWorkIter::from_sync(WorkIter::new(r.clone())),
            AsyncConsIter::from_sync(ConsIter::new(r)),
        )
    }
}
