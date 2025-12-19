use core::cell::UnsafeCell;

#[cfg(feature = "async")]
use crate::iters_components::async_iters::{mutable::AsyncCompMut, non_mutable::AsyncComp};
use crate::{
    iters_components::shared_iters::{mutable::SharedCompMut, non_mutable::SharedComp},
    ring_buffer::{iters_components::IterComponent, storage_components::PStorageComponent},
};

pub mod impls;
pub mod iters_components;
pub mod storage_components;
pub mod types;
pub mod wrappers;

/// Trait implemented by concurrent ring buffer.
pub(crate) trait SharedRB {}

impl<S: PStorageComponent> SharedRB for OneRingBuf<S, SharedComp> {}
impl<S: PStorageComponent> SharedRB for OneRingBuf<S, SharedCompMut> {}
#[cfg(feature = "async")]
impl<S: PStorageComponent> SharedRB for OneRingBuf<S, AsyncComp> {}
#[cfg(feature = "async")]
impl<S: PStorageComponent> SharedRB for OneRingBuf<S, AsyncCompMut> {}

/// Trait implemented by ring buffers.
///
/// This trait provides the core functionality for all ring buffer types in this crate.
/// It defines the associated types for the item, storage, and iterators, and provides
/// methods to access the storage and iterators, and to get the length of the buffer.
pub trait OneRB {
    /// The type of items stored in the ring buffer.
    type Item;
    /// The storage component for the ring buffer.
    type Storage: PStorageComponent<Item = Self::Item>;
    /// The iterator component for the ring buffer.
    type Iters: IterComponent;

    /// Returns a reference to the iterator component.
    fn iters(&self) -> &Self::Iters;
    /// Returns a reference to the storage component.
    fn storage(&self) -> &Self::Storage;
    /// Returns a mutable reference to the storage component.
    fn storage_mut(&self) -> &mut Self::Storage;
    /// Returns the length of the buffer.
    fn len(&self) -> usize;
}

/// The One Ring aka the main struct of this crate.
/// All the other buffers are based upon this.
pub struct OneRingBuf<S: PStorageComponent, I: IterComponent> {
    pub(crate) inner: UnsafeCell<S>,
    pub(crate) iters: I,
}

impl<S: PStorageComponent, I: IterComponent> OneRB for OneRingBuf<S, I> {
    type Item = S::Item;
    type Storage = S;
    type Iters = I;

    #[inline]
    fn iters(&self) -> &Self::Iters {
        &self.iters
    }

    #[inline]
    fn storage(&self) -> &Self::Storage {
        unsafe { &*self.inner.get() }
    }

    #[inline]
    fn storage_mut(&self) -> &mut Self::Storage {
        unsafe { &mut *self.inner.get() }
    }

    #[inline]
    fn len(&self) -> usize {
        self.storage().len()
    }
}

impl<S: PStorageComponent, I: IterComponent> OneRingBuf<S, I> {
    pub(crate) const fn _from(value: S, iters: I) -> Self {
        Self {
            inner: UnsafeCell::new(value),
            iters: iters,
        }
    }
}
