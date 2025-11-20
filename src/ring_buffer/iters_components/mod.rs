//! Components used by the buffers to manage iterators.

#[cfg(feature = "async")]
pub use async_iters::{AsyncIterComp, mutable::AsyncCompMut, non_mutable::AsyncComp};
pub use local_iters::mutable::LocalCompMut;
pub use local_iters::non_mutable::LocalComp;
pub use shared_iters::mutable::SharedCompMut;
pub use shared_iters::non_mutable::SharedComp;

pub(crate) mod async_iters;
pub(crate) mod local_iters;
pub(crate) mod shared_iters;

pub(crate) trait PIterComponent {
    fn middle_iter_idx(&self) -> usize;
    fn drop_iter(&self) -> u8;
    fn acquire_fence(&self);
    fn prod_index(&self) -> usize;
    fn work_index(&self) -> usize;
    fn cons_index(&self) -> usize;
    fn set_prod_index(&self, index: usize);
    fn set_work_index(&self, index: usize);
    fn set_cons_index(&self, index: usize);
    fn alive_iters(&self) -> u8;
}

/// Trait implemented by all iterator components.
pub trait IterComponent: PIterComponent {}
/// Trait implemented by mutable iterator components.
pub trait MutIterComp: IterComponent {}
/// Trait implemented by non-mutable iterator components.
pub trait NonMutIterComp: IterComponent {}
