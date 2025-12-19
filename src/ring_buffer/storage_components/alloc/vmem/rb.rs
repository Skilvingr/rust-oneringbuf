#[cfg(any(feature = "async", doc))]
use crate::AsyncVmemRB;
use crate::LocalVmemRB;
use crate::SharedVmemRB;
#[allow(unused_imports)]
use crate::iterators::ProdIter;
#[cfg(any(feature = "async", doc))]
use crate::iters_components::AsyncComp;
#[cfg(any(feature = "async", doc))]
use crate::iters_components::AsyncCompMut;
use crate::iters_components::LocalComp;
use crate::iters_components::LocalCompMut;
use crate::iters_components::SharedComp;
use crate::iters_components::SharedCompMut;
#[cfg(any(feature = "async", doc))]
use crate::ring_buffer::types::AsyncVmemRBMut;
use crate::ring_buffer::types::LocalVmemRBMut;
use crate::ring_buffer::types::SharedVmemRBMut;
use crate::storage_components::VmemStorage;
use crate::utils::UnsafeSyncCell;
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

fn get_range_max<T>(capacity: usize) -> usize {
    let min_size = capacity * size_of::<T>();
    return super::vmem_helper::get_page_size_mul(min_size) / size_of::<T>();
}

macro_rules! impl_rb {
    ($t: tt, $i: tt) => {
        impl<T> From<Vec<T>> for $t<T> {
            #[doc = concat!("Converts a `Vec<T>` into a [`", stringify!($t), "`].")]
            /// Note that the length of the buffer will be equal to the length of the vector, and *not*
            /// to its capacity.
            /// # Behaviour with `vmem` feature
            /// When `vmem` feature is enabled, the capacity of the buffer must be a multiple of
            /// the system's page size, so must be the length of the passed `Vec`.
            /// Please, use [`crate::utils::vmem_helper::get_page_size_mul`] to get a suitable length.
            fn from(value: Vec<T>) -> Self {
                assert!(value.len() > 0);

                Self::_from(VmemStorage::from(value), $i::default())
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
                assert!(capacity > 0);

                Self::_from(
                    VmemStorage::from(
                        (0..get_range_max::<T>(capacity))
                        .map(|_| UnsafeSyncCell::new_zeroed()).collect::<Box<[UnsafeSyncCell<T>]>>()
                    ),
                    $i::default()
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
                    assert!(capacity > 0);
                    Self::from(vec![T::default(); get_range_max::<T>(capacity)])
            }
        }
    };
}

#[cfg(any(feature = "async", doc))]
impl_rb!(AsyncVmemRB, AsyncComp);
#[cfg(any(feature = "async", doc))]
impl_rb!(AsyncVmemRBMut, AsyncCompMut);

impl_rb!(SharedVmemRB, SharedComp);
impl_rb!(SharedVmemRBMut, SharedCompMut);
impl_rb!(LocalVmemRB, LocalComp);
impl_rb!(LocalVmemRBMut, LocalCompMut);
