#![cfg(feature = "alloc")]

use alloc::boxed::Box;
use core::ops::Deref;
use core::ptr::NonNull;

use crate::ring_buffer::{OneRB, iters_components::PIterComponent, wrappers::refs::BufRef};

pub struct DroppableRef<B: OneRB> {
    inner: NonNull<B>,
}

impl<B: OneRB> From<B> for DroppableRef<B> {
    fn from(value: B) -> Self {
        let x = Box::new(value);

        Self {
            inner: NonNull::new(Box::into_raw(x)).unwrap(),
        }
    }
}

impl<B: OneRB> DroppableRef<B> {
    #[inline(never)]
    fn try_drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.inner.as_ptr());
        }
    }
}

impl<B: OneRB> Clone for DroppableRef<B> {
    fn clone(&self) -> Self {
        Self { inner: self.inner }
    }
}

impl<B: OneRB> Deref for DroppableRef<B> {
    type Target = B;

    fn deref(&self) -> &Self::Target {
        unsafe { self.inner.as_ref() }
    }
}

impl<B: OneRB> Drop for DroppableRef<B> {
    fn drop(&mut self) {
        if unsafe { self.inner.as_ref().iters().drop_iter() } != 1 {
            return;
        }

        self.iters().acquire_fence();

        self.try_drop();
    }
}

impl<B: OneRB> BufRef for DroppableRef<B> {
    type Buffer = B;
}
