use core::ops::Deref;
use core::ptr::NonNull;

use crate::ring_buffer::{OneRB, iters_components::PIterComponent, wrappers::refs::BufRef};

pub struct NonDroppableRef<B: OneRB> {
    inner: NonNull<B>,
}

impl<B: OneRB> From<&mut B> for NonDroppableRef<B> {
    fn from(buf: &'_ mut B) -> Self {
        Self {
            inner: NonNull::from(buf),
        }
    }
}

impl<B: OneRB> Clone for NonDroppableRef<B> {
    fn clone(&self) -> Self {
        Self { inner: self.inner }
    }
}

impl<B: OneRB> Deref for NonDroppableRef<B> {
    type Target = B;

    fn deref(&self) -> &Self::Target {
        unsafe { self.inner.as_ref() }
    }
}

impl<B: OneRB> Drop for NonDroppableRef<B> {
    fn drop(&mut self) {
        if unsafe { self.inner.as_ref().iters().drop_iter() } != 1 {
            return;
        }

        self.iters().acquire_fence();
    }
}

impl<B: OneRB> BufRef for NonDroppableRef<B> {
    type Buffer = B;
}
