# OneRingBuf

[![crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![Rust + Miri + Sanitisers][tests-badge]][tests-url]

[crates-badge]: https://img.shields.io/crates/v/oneringbuf.svg
[crates-url]: https://crates.io/crates/oneringbuf
[docs-badge]: https://docs.rs/oneringbuf/badge.svg
[docs-url]: https://docs.rs/oneringbuf
[tests-badge]: https://github.com/Skilvingr/rust-oneringbuf/actions/workflows/rust.yml/badge.svg
[tests-url]: https://github.com/Skilvingr/rust-oneringbuf/actions/workflows/rust.yml

>_One Ring to rule them all, One ring to find them,\
One Ring to bring them all and in the mem'ry bind them\
In the Land of Rust where the Data lies._

A lock-free, single-producer, single-consumer (SPSC) ring buffer optimised for real-time applications. It offers in-place mutability, asynchronous support, and virtual memory optimisations.

## Key Features

*   **Lock-Free SPSC:** Designed for high-throughput, low-latency communication between two threads.
*   **In-Place Mutability:** Modify elements directly in the buffer without needing to move them, reducing copying and improving performance.
*   **Asynchronous Support:** First-class `async/await` integration for non-blocking operations.
*   **Virtual Memory Optimisation:** Uses `vmem` to map the buffer to contiguous virtual memory, allowing for safe and efficient access to slices of data.
*   **Flexible Memory Allocation:** Supports both stack and heap allocation to suit different needs.

## When to Use `oneringbuf`

This crate was specifically developed for real-time audio stream processing, but it is well-suited for any scenario that requires a high-performance SPSC queue, such as:

*   Real-time signal processing
*   Game development (e.g., for audio or event queues)
*   High-frequency data streaming
*   Anywhere you need to pass data between two threads with minimal overhead.

A practical example for audio processing can be found [here](https://github.com/Skilvingr/rust-oneringbuf/blob/master/examples/cpal.rs).

## Getting Started

Add `oneringbuf` to your `Cargo.toml`:

```toml
[dependencies]
oneringbuf = "0.1.0" # Replace with the latest version
```

Here is a basic example of creating a buffer, producing, and consuming data:

```rust
use oneringbuf::LocalHeapRB;

fn main() {
    // Create a heap-allocated ring buffer with a capacity for 1024 elements.
    let buf = LocalHeapRB::from(vec![0; 1024]);
    let (mut prod, mut cons) = buf.split();

    // Push some data into the buffer.
    for i in 1..=10 {
        prod.push(i).unwrap();
    }

    // Consume the data from the buffer.
    for _ in 0..10 {
        if let Some(val) = cons.pop() {
            println!("Popped: {}", val);
        }
    }
}
```

## Choosing the Right Buffer

`oneringbuf` provides several buffer types to suit different use cases. Here’s a guide to help you choose:

*   **Stack-Allocated (`*StackRB`)**: Ideal for small buffers where the size is known at compile time. Offers fast allocation but is limited by stack space.
*   **Heap-Allocated (`*HeapRB`)**: Suitable for large buffers or when the size is determined at runtime. More flexible but with a small overhead for dynamic memory management.

And for concurrency:

*   **Local (`Local*RB`)**: For single-threaded use. Offers the best performance due to simpler synchronisation.
*   **Shared (`Shared*RB`)**: For multi-threaded environments where the producer and consumer are on different threads. Uses atomic operations for thread safety.
*   **Async (`Async*RB`)**: Designed for `async/await`, providing non-blocking operations suitable for asynchronous runtimes.

## Core Concepts

### In-place Mutability

A key feature of `oneringbuf` is the ability to modify elements directly within the buffer. This is achieved through the `WorkIter`, which provides mutable references to items that have been produced but not yet consumed. This avoids unnecessary data copying and can significantly improve performance in scenarios like audio processing or data transformations.

```rust
use oneringbuf::{LocalHeapRBMut, ORBIterator};

let buf = LocalHeapRBMut::from(vec![0; 4096]);
let (mut prod, mut work, mut cons) = buf.split_mut();

// Produce some data
prod.push(10).unwrap();

// Mutate the data in-place
if let Some(item) = work.get_mut() {
    *item *= 2;
    unsafe { work.advance(1); }
}

// The consumer will now see the modified value
assert_eq!(cons.pop(), Some(20));
```

## Advanced Features

### Virtual Memory (`vmem`) Optimisation

For circular buffers, a powerful optimisation is to map the underlying buffer to two contiguous regions of virtual memory. This allows you to treat the circular buffer as a linear one, making it possible to read or write data across the wrap-around point in a single operation. More information can be found [here](https://en.wikipedia.org/wiki/Circular_buffer#Optimization).

This crate supports this optimisation through the `vmem` feature flag, which is currently limited to `unix` targets. When using `vmem`, the buffer size (length of the buffer times the size of the stored type) must be a multiple of the system's page size (usually `4096` for x86_64).

At the moment, the feature has been tested on GNU/Linux, Android, macOS and iOS.

#### A Note About iOS

`vmem` works by allocating shared memory. While this doesn't represent a problem on other platforms, it is different on iOS. Users should create an app group (more information [here](https://developer.apple.com/documentation/xcode/configuring-app-groups)) and then set the environment variable `IOS_APP_GROUP_NAME` to the name of that group.

## Building and Running Examples

To run the tests, benchmarks, or examples, clone the repository and use the following commands from the root directory.

To run tests:
```shell
cargo +nightly test
```

To run benchmarks:
```shell
cargo bench
```

To run the CPAL example:
```shell
RUSTFLAGS="--cfg cpal" cargo run --example cpal
```
If you encounter an error like `ALSA lib pcm_dsnoop.c:567:(snd_pcm_dsnoop_open) unable to open slave`, please refer to [this issue](https://github.com/Uberi/speech_recognition/issues/526#issuecomment-1670900376).

To run any other example (e.g., `simple_async`):
```shell
cargo run --example simple_async --features async
```

## Best Practices

1.  **Handle Uninitialised Items with Care**: If you create a buffer with uninitialised memory (e.g., with `new_zeroed`), you must use `*_init` methods (e.g., `push_init`) to safely write data. Using normal methods on uninitialised memory can lead to Undefined Behaviour.
2.  **Match Buffer Type to Use Case**: Choose the buffer type that best fits your performance and concurrency needs.
3.  **Buffer Sizing for `vmem`**: When using the `vmem` feature, ensure your buffer size is a multiple of the system's page size to prevent panics.
4.  **Graceful Shutdown**: The buffer is deallocated only when the last of its iterators is dropped. Ensure all iterators are dropped for proper cleanup.
