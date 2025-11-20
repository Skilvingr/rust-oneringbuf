use core::cell::UnsafeCell;

use crate::{
    iters_components::NonMutIterComp,
    ring_buffer::iters_components::{IterComponent, PIterComponent},
};

/// Non-mutable iterators component usable in single-threaded environments.
pub struct LocalComp {
    prod_idx: UnsafeCell<usize>,
    cons_idx: UnsafeCell<usize>,

    alive_iters: UnsafeCell<u8>,
}

impl NonMutIterComp for LocalComp {}

impl Default for LocalComp {
    fn default() -> Self {
        Self {
            prod_idx: Default::default(),
            cons_idx: Default::default(),
            alive_iters: 2.into(),
        }
    }
}

impl PIterComponent for LocalComp {
    #[inline(always)]
    fn middle_iter_idx(&self) -> usize {
        self.prod_index()
    }

    fn drop_iter(&self) -> u8 {
        unsafe {
            let ret = *self.alive_iters.get();
            *self.alive_iters.get() -= 1;
            ret
        }
    }

    fn acquire_fence(&self) {}

    #[inline]
    fn prod_index(&self) -> usize {
        unsafe { *self.prod_idx.get() }
    }

    #[inline]
    fn work_index(&self) -> usize {
        0
    }

    #[inline]
    fn cons_index(&self) -> usize {
        unsafe { *self.cons_idx.get() }
    }

    #[inline]
    fn set_prod_index(&self, index: usize) {
        unsafe {
            *self.prod_idx.get() = index;
        }
    }

    #[inline]
    fn set_work_index(&self, _index: usize) {}

    #[inline]
    fn set_cons_index(&self, index: usize) {
        unsafe {
            *self.cons_idx.get() = index;
        }
    }

    fn alive_iters(&self) -> u8 {
        unsafe { *self.alive_iters.get() }
    }
}

impl IterComponent for LocalComp {}
