use oneringbuf::ORBIterator;

/// Creates all non-async buffers exposed by `oneringbuf` crate.
/// Returns only local stack-allocated ones.
fn create_buffers<'buf>() -> (
    oneringbuf::OneRingBuf<
        oneringbuf::storage_components::StackStorage<'buf, usize, 100>,
        oneringbuf::iters_components::LocalComp,
    >,
    oneringbuf::OneRingBuf<
        oneringbuf::storage_components::StackStorage<'buf, usize, 100>,
        oneringbuf::iters_components::LocalCompMut,
    >,
) {
    // Mutable and non-mutable stack-allocated buffers.
    let buf = oneringbuf::LocalStackRB::<usize, 100>::default();
    let mut_buf = oneringbuf::LocalStackRBMut::<usize, 100>::default();

    oneringbuf::SharedStackRB::<usize, 100>::default();
    oneringbuf::SharedStackRBMut::<usize, 100>::default();

    // Mutable and non-mutable heap-allocated buffers.
    #[cfg(feature = "alloc")]
    {
        oneringbuf::LocalHeapRB::<usize>::default(100);
        oneringbuf::LocalHeapRBMut::<usize>::default(100);

        oneringbuf::SharedHeapRB::<usize>::default(100);
        oneringbuf::SharedHeapRBMut::<usize>::default(100);
    }

    // Mutable and non-mutable buffers using virtual memory storage.
    #[cfg(all(feature = "alloc", feature = "vmem", unix))]
    {
        oneringbuf::LocalVmemRB::<usize>::default(100);
        oneringbuf::LocalVmemRBMut::<usize>::default(100);

        oneringbuf::SharedVmemRB::<usize>::default(100);
        oneringbuf::SharedVmemRBMut::<usize>::default(100);
    }

    (buf, mut_buf)
}

/// Creates all async buffers exposed by `oneringbuf` crate.
/// Returns only local stack-allocated ones.
#[cfg(feature = "async")]
fn create_async_buffers<'buf>() -> (
    oneringbuf::OneRingBuf<
        oneringbuf::storage_components::StackStorage<'buf, usize, 100>,
        oneringbuf::iters_components::AsyncComp,
    >,
    oneringbuf::OneRingBuf<
        oneringbuf::storage_components::StackStorage<'buf, usize, 100>,
        oneringbuf::iters_components::AsyncCompMut,
    >,
) {
    // Mutable and non-mutable async stack-allocated buffers.
    let buf = oneringbuf::AsyncStackRB::<usize, 100>::default();
    let mut_buf = oneringbuf::AsyncStackRBMut::<usize, 100>::default();

    // Mutable and non-mutable async heap-allocated buffers.
    #[cfg(feature = "alloc")]
    {
        oneringbuf::AsyncHeapRB::<usize>::default(100);
        oneringbuf::AsyncHeapRBMut::<usize>::default(100);
    }

    // Mutable and non-mutable async buffers using virtual memory storage.
    #[cfg(all(feature = "alloc", feature = "vmem", unix))]
    {
        oneringbuf::AsyncVmemRB::<usize>::default(100);
        oneringbuf::AsyncVmemRBMut::<usize>::default(100);
    }

    (buf, mut_buf)
}

#[tokio::main]
async fn main() {
    let (mut buf, mut mut_buf) = create_buffers();

    // Split non-mutable buffer.
    let (_prod, _cons) = buf.split();
    // Split mutable buffer.
    let (mut prod, work, mut cons) = mut_buf.split_mut();

    // Push an element.
    prod.push(1).unwrap();

    // (Optionally, detach the worker and) do something...
    let mut detached = work.detach();
    if let Some(x) = detached.get_mut() {
        *x += 1;
        unsafe {
            detached.advance(1);
        }
    }
    // Re-attach the detached iterator.
    let mut _work = detached.attach();

    // Pop the element by copying it.
    let x = cons.pop().unwrap();
    assert_eq!(x, 2);

    #[cfg(feature = "async")]
    {
        use oneringbuf::iterators::async_iterators::AsyncIterator;

        let (mut buf, mut mut_buf) = create_async_buffers();

        // Split non-mutable buffer.
        let (_prod, _cons) = buf.split_async();
        // Split mutable buffer.
        let (mut prod, work, mut cons) = mut_buf.split_async_mut();

        // Push an element.
        prod.push(1).await.unwrap();

        // (Optionally, detach the worker and) do something...
        let mut detached = work.detach();
        if let Some(x) = detached.get_mut().await {
            *x += 1;
            unsafe {
                detached.advance(1);
            }
        }
        // Re-attach the detached iterator.
        let mut _work = detached.attach();

        // Pop the element by copying it.
        let x = cons.pop().await.unwrap();
        assert_eq!(x, 2);
    };
}
