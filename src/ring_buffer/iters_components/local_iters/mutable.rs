use core::cell::UnsafeCell;

use crate::{
    iters_components::MutIterComp,
    ring_buffer::iters_components::{IterComponent, PIterComponent},
};

/// Mutable iterators component usable in single-threaded environments.
pub struct LocalCompMut {
    prod_idx: UnsafeCell<usize>,
    work_idx: UnsafeCell<usize>,
    cons_idx: UnsafeCell<usize>,

    alive_iters: UnsafeCell<u8>,
}

impl MutIterComp for LocalCompMut {}

impl LocalCompMut {
    pub const fn default() -> Self {
        Self {
            prod_idx: UnsafeCell::new(0),
            work_idx: UnsafeCell::new(0),
            cons_idx: UnsafeCell::new(0),
            alive_iters: UnsafeCell::new(3),
        }
    }
}

impl PIterComponent for LocalCompMut {
    #[inline(always)]
    fn middle_iter_idx(&self) -> usize {
        self.work_index()
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
        unsafe { *self.work_idx.get() }
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
    fn set_work_index(&self, index: usize) {
        unsafe {
            *self.work_idx.get() = index;
        }
    }

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

impl IterComponent for LocalCompMut {}
