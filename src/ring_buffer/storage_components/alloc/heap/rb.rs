#[cfg(any(feature = "async", doc))]
use crate::AsyncHeapRB;
use crate::LocalHeapRB;
use crate::SharedHeapRB;
#[allow(unused_imports)]
use crate::iterators::ProdIter;
#[cfg(any(feature = "async", doc))]
use crate::ring_buffer::types::AsyncHeapRBMut;
use crate::ring_buffer::types::LocalHeapRBMut;
use crate::ring_buffer::types::SharedHeapRBMut;
use crate::storage_components::HeapStorage;
use crate::utils::UnsafeSyncCell;

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

macro_rules! impl_rb {
    ($t: tt) => {
        impl<T> From<Vec<T>> for $t<T> {
            #[doc = concat!("Converts a `Vec<T>` into a [`", stringify!($t), "`].")]
            /// Note that the length of the buffer will be equal to the length of the vector, and *not*
            /// to its capacity.
            /// # Behaviour with `vmem` feature
            /// When `vmem` feature is enabled, the capacity of the buffer must be a multiple of
            /// the system's page size, so must be the length of the passed `Vec`.
            /// Please, use [`crate::utils::vmem_helper::get_page_size_mul`] to get a suitable length.
            fn from(value: Vec<T>) -> Self {
                Self::_from(HeapStorage::from(value))
            }
        }

        impl<T> $t<T> {
            #[doc = concat!("Creates a new [`", stringify!($t), "`] with given capacity and zeroed (uninitialised) elements.")]
            /// # Safety
            /// The buffer must be then initialised using proper [`ProdIter`] methods (`*_init` ones).
            /// # Behaviour with `vmem` feature
            /// When `vmem` feature is enabled, the capacity of the buffer must be a multiple of
            /// the system's page size.
            /// This method accepts a minimum size, which will then be used to compute the actual
            /// size (equal to or greater than it).
            pub unsafe fn new_zeroed(capacity: usize) -> Self {
                Self::_from(
                    HeapStorage::from(
                        (0..capacity)
                        .map(|_| UnsafeSyncCell::new_zeroed()).collect::<Box<[UnsafeSyncCell<T>]>>()
                    )
                )
            }

            #[doc = concat!("Creates a new [`", stringify!($t), "`] with given capacity and elements initialised to `default`.")]
            /// # Behaviour with `vmem` feature
            /// When `vmem` feature is enabled, the capacity of the buffer must be a multiple of
            /// the system's page size.
            /// This method accepts a minimum size, which will then be used to compute the actual
            /// size (equal to or greater than it).
            pub fn default(capacity: usize) -> Self
                where T: Default + Clone {
                Self::from(vec![T::default(); capacity])
            }
        }
    };
}

#[cfg(any(feature = "async", doc))]
impl_rb!(AsyncHeapRB);
#[cfg(any(feature = "async", doc))]
impl_rb!(AsyncHeapRBMut);

impl_rb!(SharedHeapRB);
impl_rb!(SharedHeapRBMut);
impl_rb!(LocalHeapRB);
impl_rb!(LocalHeapRBMut);
