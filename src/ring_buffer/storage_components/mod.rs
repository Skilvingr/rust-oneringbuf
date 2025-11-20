//! Components used by the buffers to store data.

use crate::utils::UnsafeSyncCell;

#[cfg(feature = "alloc")]
pub use alloc::heap::HeapStorage;
#[cfg(all(feature = "alloc", feature = "vmem", unix))]
pub use alloc::vmem::VmemStorage;
pub use stack::StackStorage;

pub(crate) mod alloc;
pub(crate) mod stack;

/// Trait implemented by `*Storage` structs.
#[allow(clippy::len_without_is_empty)]
pub(crate) trait PStorageComponent: StorageComponent {
    fn _index(&self, index: usize) -> &UnsafeSyncCell<Self::Item>;

    /// Returns the length of the underlying array.
    fn len(&self) -> usize;

    /// Returns the next chunk long `count` which starts from `index`.
    fn next_chunk<'a>(&self, index: usize, count: usize) -> Self::SliceOutput<'a>;
    /// Returns the next mutable chunk long `count` which starts from `index`.
    fn next_chunk_mut<'a>(&mut self, index: usize, count: usize) -> Self::SliceOutputMut<'a>;

    fn _push_slice(
        &mut self,
        index: usize,
        slice: &[Self::Item],
        f: fn(&mut [Self::Item], &[Self::Item]),
    );

    fn _extract_slice(
        &mut self,
        index: usize,
        dst: &mut [Self::Item],
        f: fn(&[Self::Item], &mut [Self::Item]),
    );
}

/// Trait implemented by storage components.
pub trait StorageComponent {
    /// Type stored in the storage.
    type Item;

    /// Type used when working with non-mutable slices.
    type SliceOutput<'a>
    where
        Self::Item: 'a;

    /// Type used when working with non-mutable slices.
    type SliceOutputMut<'a>
    where
        Self::Item: 'a;
}
