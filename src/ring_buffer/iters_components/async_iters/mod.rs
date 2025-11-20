#![cfg(any(feature = "async", doc))]

use core::task::Waker;

use crate::IterComponent;

pub mod mutable;
pub mod non_mutable;

/// Trait implemented by async iterator components.
///
/// This trait is not meant to be implemented outside of this crate, nor
/// are its methods meant to be called directly.
/// Instead, this trait should be used only as parameter/bound.
pub trait AsyncIterComp: IterComponent {
    fn wake_middle_iter(&self);

    fn register_prod_waker(&self, waker: &Waker);
    fn take_prod_waker(&self) -> Option<Waker>;
    fn wake_prod(&self);

    fn register_work_waker(&self, waker: &Waker);
    fn take_work_waker(&self) -> Option<Waker>;
    fn wake_work(&self);

    fn register_cons_waker(&self, waker: &Waker);
    fn take_cons_waker(&self) -> Option<Waker>;
    fn wake_cons(&self);
}
