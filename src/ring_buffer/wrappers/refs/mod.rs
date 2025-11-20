use core::ops::Deref;

pub mod droppable;
pub mod non_droppable;

pub(crate) trait BufRef: Deref<Target = Self::Buffer> + Clone {
    type Buffer;
}

/// Trait used to create a shared reference out of a buffer.
///
/// This trait is not meant to be implemented outside of this crate, nor
/// are its methods meant to be called directly.
/// Instead, this trait should be used only as parameter/bound.
pub trait IntoRef {
    type TargetRef: BufRef<Buffer = Self>;

    fn into_ref(s: Self) -> Self::TargetRef;
}
