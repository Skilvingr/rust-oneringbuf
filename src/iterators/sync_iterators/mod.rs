//! Sync iterators.

use crate::ring_buffer::{OneRB, SharedRB, wrappers::refs::IntoRef};

pub(crate) mod cons_iter;
pub(crate) mod detached;
pub(crate) mod prod_iter;
pub(crate) mod work_iter;

pub(crate) struct Iter<B: IntoRef + OneRB> {
    index: usize,
    cached_avail: usize,
    buffer: B::TargetRef,
}

unsafe impl<B: IntoRef + OneRB + SharedRB> Send for Iter<B> {}

impl<B: IntoRef + OneRB> Iter<B> {
    pub(crate) fn new(value: B::TargetRef) -> Self {
        Self {
            index: 0,
            buffer: value,
            cached_avail: 0,
        }
    }
}
