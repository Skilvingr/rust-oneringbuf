# Changelog

All notable changes to this project will be documented in this file.
Dates are written in DD/MM/YYYY order.

<a name="v0.6.0"></a>
## v0.6.0 (20/11/2025)

### Breaking Changes
* **Renamed the crate to `OneRingBuf`.**
* Renamed `Concurrent*` buffers to `Shared*` (e.g., `ConcurrentHeapRB` is now `SharedHeapRB`).
* Renamed `*workable*` methods to `*mut*`. For example: `get_workable_slice_exact` -> `get_mut_slice_exact`.
* `ConsIter` and `AsyncProdIter` no longer require a `const W: bool` parameter.
* Buffers with in-place mutability are now distinct types with a `*Mut` suffix (e.g., `SharedHeapRBMut`).
    * The `split_mut()` method is now exclusive to these `*Mut` buffer types.
    * The `split()` method is now exclusive to non-mutable buffer types.
* Reworked asynchronous support:
    * Async iterators can no longer be obtained from synchronous buffers.
    * Introduced dedicated async buffers: `AsyncHeapRB`, `AsyncStackRB`, and `AsyncVmemRB` (and their `*Mut` counterparts).
* Removed `*_alive` and `set_*_alive` methods. Users can track the status of the iterators by themselves, using counters, for example.

### New Features
* Stack-allocated, heap-allocated, and vmem-optimised buffers are no longer mutually exclusive and can be used within the same project.

### Bug Fixes
* Fixed a bug in the `drop` implementation, identified by running sanitisers.
* Fixed issue #2.
* Fixed a memory leak that occurred with uninitialised buffers.

### Other Changes
* Vmem buffer size requirements are now more flexible. The total buffer size in bytes (`length * size_of::<T>`) must be a multiple of the page size, not necessarily the length.
* Added sanitisers to the CI pipeline.


<a name="v0.5.4"></a>
## v0.5.4 (02/10/2025)

* The `index` method now panics on out-of-bounds access.

<a name="v0.5.3"></a>
## v0.5.3 (09/05/2025)

* Fixed documentation.

<a name="v0.5.2"></a>
## v0.5.2 (08/05/2025)

* Fixed virtual memory optimisation.
* Added additional tests.

<a name="v0.5.1"></a>
## v0.5.1 (17/04/2025)

* Added `UnsafeSyncCell::take_inner` method.
* Fixed virtual memory optimisation.

<a name="v0.5.0"></a>
## v0.5.0 (15/04/2025)

* Moved iterators to the `mutringbuf::iterators` module.
* Added support for virtual memory optimisation.

<a name="v0.4.2"></a>
## v0.4.2 (07/02/2025)

* Re-exported `ConcurrentRB` trait.

<a name="v0.4.1"></a>
## v0.4.1 (27/01/2025)

* Removed the `set_index` method from the common trait; it is now only available for `Detached` iterators.
* Implemented performance optimisations.
* Fixed a memory leak.
* Re-exported `Storage` trait.

<a name="v0.4.0"></a>
## v0.4.0 (15/11/2024)

* Moved buffer split methods to the `HeapSplit` and `StackSplit` traits.
* `pop()` in `ConsIter` now performs a bitwise copy (`ptr::read`) of the cell's content.
* The previous `pop()` behaviour is now available via the `pop_move()` method.
* Methods previously exclusive to `WorkIter` (e.g., `get_mut`, `get_mut_slice_exact`) are now available for all iterator types.
* All iterator types can now be detached, yielding a `Detached` or `AsyncDetached` iterator.
* Enabled async iterators in `no-std` environments.
* Updated several import paths.

<a name="v0.3.1"></a>
## v0.3.1 (27/06/2024)

* All iterators can now retrieve the indices of other iterators.
* Added a method to producer iterators for fetching a tuple of mutable slices, allowing direct writes.
* Added a convenience method to busy-wait for a specific number of items.
* Fixed a bug that could cause iterator indices to overlap.

<a name="v0.3.0"></a>
## v0.3.0 (13/05/2024)

* Added a `new_zeroed()` method to `ConcurrentStackRB` and `LocalStackRB` to create buffers with uninitialised elements.
* Renamed `new()` methods in `ConcurrentHeapRB` and `LocalHeapRB` to `new_zeroed()`.
* Split `ProdIter` methods into standard and `*_init` variants to support working with uninitialised memory.
* Fixed several instances of Undefined Behaviour.
* Resolved memory leaks when dropping a buffer.
* Added initial asynchronous support.

<a name="v0.2.0"></a>
## v0.2.0 (06/05/2024)

* Removed the accumulator from `WorkIter`.
* Simplified buffer types in iterator definitions (e.g., `ProdIter<ConcurrentHeapRB<usize>>` instead of `ProdIter<ConcurrentHeapRB<usize>, usize>`).

<a name="v0.1.3"></a>
## v0.1.3 (29/03/2024)

* Added `get_next_item_mut` and `get_next_item_mut_init` methods to `ProdIter` for writing data via mutable references.

<a name="v0.1.2"></a>
## v0.1.2 (22/03/2024)

* Renamed `Iterator` trait to `MRBIterator` to prevent conflicts with standard library imports.

<a name="v0.1.1"></a>
## v0.1.1 (17/03/2024)

* Removed unnecessary `alloc` usage.
* Removed redundant `drop` implementation for stack-allocated buffers.
* Added tests for stack-allocated buffers.

<a name="v0.1.0"></a>
## v0.1.0 (17/03/2024)

* Renamed `new_heap(capacity)` to `new(capacity)`.
* Added a `default` method for heap-allocated buffers.

<a name="v0.1.0-alpha.1"></a>
## v0.1.0-alpha.1 (15/03/2024)

* Renamed `Iterable` trait to `Iterator`.
* Added a new `MutRB` trait to represent a generic ring buffer.
