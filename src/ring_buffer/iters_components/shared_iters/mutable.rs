use core::sync::atomic::{AtomicU8, AtomicUsize};

use crossbeam_utils::CachePadded;

use core::sync::atomic::Ordering::{Acquire, Release};

use crate::{
    iters_components::MutIterComp,
    ring_buffer::iters_components::{IterComponent, PIterComponent},
};

/// Mutable iterators component usable in concurrent environments.
pub struct SharedCompMut {
    pub(crate) prod_idx: CachePadded<AtomicUsize>,
    pub(crate) work_idx: CachePadded<AtomicUsize>,
    pub(crate) cons_idx: CachePadded<AtomicUsize>,

    pub(crate) alive_iters: AtomicU8,
}

impl MutIterComp for SharedCompMut {}

impl SharedCompMut {
    pub const fn default() -> Self {
        Self {
            prod_idx: CachePadded::new(AtomicUsize::new(0)),
            work_idx: CachePadded::new(AtomicUsize::new(0)),
            cons_idx: CachePadded::new(AtomicUsize::new(0)),
            alive_iters: AtomicU8::new(3),
        }
    }
}

impl PIterComponent for SharedCompMut {
    #[inline(always)]
    fn middle_iter_idx(&self) -> usize {
        self.work_index()
    }

    #[inline(always)]
    fn drop_iter(&self) -> u8 {
        self.alive_iters.fetch_sub(1, Release)
    }

    #[inline(always)]
    fn acquire_fence(&self) {
        #[cfg(not(feature = "thread_sanitiser"))]
        core::sync::atomic::fence(Acquire);

        // ThreadSanitizer does not support memory fences. To avoid false positive
        // reports use atomic loads for synchronization instead.
        #[cfg(feature = "thread_sanitiser")]
        self.alive_iters.load(Acquire);
    }

    #[inline]
    fn prod_index(&self) -> usize {
        self.prod_idx.load(Acquire)
    }

    #[inline]
    fn work_index(&self) -> usize {
        self.work_idx.load(Acquire)
    }

    #[inline]
    fn cons_index(&self) -> usize {
        self.cons_idx.load(Acquire)
    }

    #[inline]
    fn set_prod_index(&self, index: usize) {
        self.prod_idx.store(index, Release);
    }

    #[inline]
    fn set_work_index(&self, index: usize) {
        self.work_idx.store(index, Release);
    }

    #[inline]
    fn set_cons_index(&self, index: usize) {
        self.cons_idx.store(index, Release);
    }

    fn alive_iters(&self) -> u8 {
        self.alive_iters.load(Acquire)
    }
}

impl IterComponent for SharedCompMut {}
