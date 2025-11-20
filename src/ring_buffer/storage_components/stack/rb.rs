#[cfg(doc)]
use crate::iterators::ProdIter;
#[cfg(any(feature = "async", doc))]
use crate::{AsyncStackRB, ring_buffer::types::AsyncStackRBMut};

use crate::storage_components::StackStorage;
use crate::{
    LocalStackRB, SharedStackRB,
    ring_buffer::types::{LocalStackRBMut, SharedStackRBMut},
    utils::UnsafeSyncCell,
};

macro_rules! impl_rb {
    ($t: tt) => {
        impl<'buf, T, const N: usize> From<[T; N]> for $t<'buf, T, N> {
            #[doc = concat!("Converts an array into a [`", stringify!($t), "`].")]
            fn from(value: [T; N]) -> Self {
                Self::_from(StackStorage::from(value))
            }
        }

        impl<'buf, T, const N: usize> $t<'buf, T, N> {
            #[doc = concat!("Creates a new [`", stringify!($t), "`] with given capacity and zeroed (uninitialised) elements.")]
            /// # Safety
            /// The buffer must be then initialised using proper [`ProdIter`] methods (`*_init` ones).
            pub unsafe fn new_zeroed() -> Self {
                let v: [UnsafeSyncCell<T>; N] = core::array::from_fn(|_| UnsafeSyncCell::new_zeroed());

                Self::_from(StackStorage::from(v))
            }
        }

        impl<'buf, T: Default + Copy, const N: usize> Default for $t<'buf, T, N> {
            #[doc = concat!("Creates a new [`", stringify!($t), "`] with given capacity and elements initialised to `default`.")]
            fn default() -> Self {
                Self::from([T::default(); N])
            }
        }
    };
}

#[cfg(any(feature = "async", doc))]
impl_rb!(AsyncStackRB);
#[cfg(any(feature = "async", doc))]
impl_rb!(AsyncStackRBMut);

impl_rb!(SharedStackRB);
impl_rb!(SharedStackRBMut);
impl_rb!(LocalStackRB);
impl_rb!(LocalStackRBMut);
