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
    iterators::{AsyncConsIter, AsyncProdIter, ConsIter, ProdIter, async_iterators::AsyncIterator},
    iters_components::{
        NonMutIterComp, async_iters::AsyncIterComp, shared_iters::non_mutable::SharedComp,
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

/// Non-mutable async iterators component usable in concurrent environments.
pub struct AsyncComp {
    inner: SharedComp,

    prod_waker: CachePadded<AtomicWaker>,
    cons_waker: CachePadded<AtomicWaker>,
}

impl NonMutIterComp for AsyncComp {}

impl AsyncComp {
    pub const fn default() -> Self {
        Self {
            inner: SharedComp::default(),

            prod_waker: CachePadded::new(AtomicWaker::new()),
            cons_waker: CachePadded::new(AtomicWaker::new()),
        }
    }
}

impl AsyncIterComp for AsyncComp {
    fn wake_middle_iter(&self) {
        self.cons_waker.wake();
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

    fn register_work_waker(&self, _waker: &Waker) {}

    fn take_work_waker(&self) -> Option<Waker> {
        None
    }

    fn wake_work(&self) {}

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

impl PIterComponent for AsyncComp {
    #[inline(always)]
    fn middle_iter_idx(&self) -> usize {
        self.prod_index()
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
        0
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
    fn set_work_index(&self, _index: usize) {}

    #[inline]
    fn set_cons_index(&self, index: usize) {
        self.inner.cons_idx.store(index, Release);
    }

    fn alive_iters(&self) -> u8 {
        self.inner.alive_iters.load(Acquire)
    }
}

impl IterComponent for AsyncComp {}

impl<'buf, T, const N: usize> OneRingBuf<StackStorage<'buf, T, N>, AsyncComp> {
    /// Returns two iterators: a Producer and a Consumer.
    /// <div class="warning">Available only for non-mutable buffers.</div>
    pub fn split_async(
        &'buf mut self,
    ) -> (
        AsyncProdIter<OneRingBuf<StackStorage<'buf, T, N>, AsyncComp>>,
        AsyncConsIter<OneRingBuf<StackStorage<'buf, T, N>, AsyncComp>>,
    ) {
        let r = NonDroppableRef::from(self);
        (
            AsyncProdIter::from_sync(ProdIter::new(r.clone())),
            AsyncConsIter::from_sync(ConsIter::new(r)),
        )
    }
}

#[cfg(feature = "alloc")]
impl<T> OneRingBuf<HeapStorage<T>, AsyncComp> {
    /// Returns two iterators: a Producer and a Consumer.
    /// <div class="warning">Available only for non-mutable buffers.</div>
    pub fn split_async(
        self,
    ) -> (
        AsyncProdIter<OneRingBuf<HeapStorage<T>, AsyncComp>>,
        AsyncConsIter<OneRingBuf<HeapStorage<T>, AsyncComp>>,
    ) {
        let r = DroppableRef::from(self);
        (
            AsyncProdIter::from_sync(ProdIter::new(r.clone())),
            AsyncConsIter::from_sync(ConsIter::new(r)),
        )
    }
}

#[cfg(all(feature = "alloc", feature = "vmem", unix))]
impl<T> OneRingBuf<VmemStorage<T>, AsyncComp> {
    /// Returns two iterators: a Producer and a Consumer.
    /// <div class="warning">Available only for non-mutable buffers.</div>
    pub fn split_async(
        self,
    ) -> (
        AsyncProdIter<OneRingBuf<VmemStorage<T>, AsyncComp>>,
        AsyncConsIter<OneRingBuf<VmemStorage<T>, AsyncComp>>,
    ) {
        let r = DroppableRef::from(self);
        (
            AsyncProdIter::from_sync(ProdIter::new(r.clone())),
            AsyncConsIter::from_sync(ConsIter::new(r)),
        )
    }
}
