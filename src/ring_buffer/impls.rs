#[cfg(feature = "alloc")]
use crate::{
    ring_buffer::wrappers::refs::droppable::DroppableRef, storage_components::HeapStorage,
};

#[cfg(all(feature = "alloc", feature = "vmem", unix))]
use crate::storage_components::VmemStorage;

use crate::{
    OneRingBuf,
    iterators::{ConsIter, ProdIter, WorkIter},
    iters_components::{MutIterComp, NonMutIterComp},
    ring_buffer::{
        iters_components::IterComponent,
        wrappers::refs::{IntoRef, non_droppable::NonDroppableRef},
    },
    storage_components::StackStorage,
};

impl<'buf, T, const N: usize, I: IterComponent> IntoRef
    for OneRingBuf<StackStorage<'buf, T, N>, I>
{
    type TargetRef = NonDroppableRef<Self>;

    fn into_ref(mut s: Self) -> Self::TargetRef {
        NonDroppableRef::from(&mut s)
    }
}

impl<'buf, T, const N: usize, I: NonMutIterComp> OneRingBuf<StackStorage<'buf, T, N>, I> {
    /// Returns two iterators: a Producer and a Consumer.
    /// <div class="warning">Available only for non-mutable buffers.</div>
    pub fn split(
        &'buf mut self,
    ) -> (
        ProdIter<OneRingBuf<StackStorage<'buf, T, N>, I>>,
        ConsIter<OneRingBuf<StackStorage<'buf, T, N>, I>>,
    ) {
        let r = NonDroppableRef::from(self);
        (ProdIter::new(r.clone()), ConsIter::new(r))
    }
}
impl<'buf, T, const N: usize, I: MutIterComp> OneRingBuf<StackStorage<'buf, T, N>, I> {
    /// Returns three iterators: a Producer, a Worker and a Consumer.
    /// <div class="warning">Available only for mutable buffers.</div>
    pub fn split_mut(
        &'buf mut self,
    ) -> (
        ProdIter<OneRingBuf<StackStorage<'buf, T, N>, I>>,
        WorkIter<OneRingBuf<StackStorage<'buf, T, N>, I>>,
        ConsIter<OneRingBuf<StackStorage<'buf, T, N>, I>>,
    ) {
        let r = NonDroppableRef::from(self);
        (
            ProdIter::new(r.clone()),
            WorkIter::new(r.clone()),
            ConsIter::new(r),
        )
    }
}

#[cfg(feature = "alloc")]
impl<T, I: IterComponent> IntoRef for OneRingBuf<HeapStorage<T>, I> {
    type TargetRef = DroppableRef<Self>;

    fn into_ref(s: Self) -> Self::TargetRef {
        DroppableRef::from(s)
    }
}

#[cfg(feature = "alloc")]
impl<T, I: NonMutIterComp> OneRingBuf<HeapStorage<T>, I> {
    /// Returns two iterators: a Producer and a Consumer.
    /// <div class="warning">Available only for non-mutable buffers.</div>
    pub fn split(
        self,
    ) -> (
        ProdIter<OneRingBuf<HeapStorage<T>, I>>,
        ConsIter<OneRingBuf<HeapStorage<T>, I>>,
    ) {
        let r = DroppableRef::from(self);
        (ProdIter::new(r.clone()), ConsIter::new(r))
    }
}
#[cfg(feature = "alloc")]
impl<T, I: MutIterComp> OneRingBuf<HeapStorage<T>, I> {
    /// Returns three iterators: a Producer, a Worker and a Consumer.
    /// <div class="warning">Available only for mutable buffers.</div>
    pub fn split_mut(
        self,
    ) -> (
        ProdIter<OneRingBuf<HeapStorage<T>, I>>,
        WorkIter<OneRingBuf<HeapStorage<T>, I>>,
        ConsIter<OneRingBuf<HeapStorage<T>, I>>,
    ) {
        let r = DroppableRef::from(self);
        (
            ProdIter::new(r.clone()),
            WorkIter::new(r.clone()),
            ConsIter::new(r),
        )
    }
}

#[cfg(all(feature = "alloc", feature = "vmem", unix))]
impl<T, I: IterComponent> IntoRef for OneRingBuf<VmemStorage<T>, I> {
    type TargetRef = DroppableRef<Self>;

    fn into_ref(s: Self) -> Self::TargetRef {
        DroppableRef::from(s)
    }
}

#[cfg(all(feature = "alloc", feature = "vmem", unix))]
impl<T, I: NonMutIterComp> OneRingBuf<VmemStorage<T>, I> {
    /// Returns two iterators: a Producer and a Consumer.
    /// <div class="warning">Available only for non-mutable buffers.</div>
    pub fn split(
        self,
    ) -> (
        ProdIter<OneRingBuf<VmemStorage<T>, I>>,
        ConsIter<OneRingBuf<VmemStorage<T>, I>>,
    ) {
        let r = DroppableRef::from(self);
        (ProdIter::new(r.clone()), ConsIter::new(r))
    }
}
#[cfg(all(feature = "alloc", feature = "vmem", unix))]
impl<T, I: MutIterComp> OneRingBuf<VmemStorage<T>, I> {
    /// Returns three iterators: a Producer, a Worker and a Consumer.
    /// <div class="warning">Available only for mutable buffers.</div>
    pub fn split_mut(
        self,
    ) -> (
        ProdIter<OneRingBuf<VmemStorage<T>, I>>,
        WorkIter<OneRingBuf<VmemStorage<T>, I>>,
        ConsIter<OneRingBuf<VmemStorage<T>, I>>,
    ) {
        let r = DroppableRef::from(self);
        (
            ProdIter::new(r.clone()),
            WorkIter::new(r.clone()),
            ConsIter::new(r),
        )
    }
}
