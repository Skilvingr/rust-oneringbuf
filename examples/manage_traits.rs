use oneringbuf::{
    IntoRef, ORBIterator, OneRB, OneRingBuf, SharedStackRBMut,
    iterators::{ProdIter, async_iterators::AsyncIterator},
    iters_components::SharedCompMut,
    storage_components::StackStorage,
};

/// `OneRB` is implemented by every buffer.
fn call_buffer_common_method(buf: &impl OneRB) -> usize {
    buf.len()
}

/// `ORBIterator` is implemented by every iterator.
fn call_iter_common_method(iter: &impl ORBIterator) -> u8 {
    iter.alive_iters()
}

fn call_iter_spec_method(iter: &mut ProdIter<impl IntoRef + OneRB<Item = usize>>) {
    let _ = iter.push(1);
}
fn call_iter_spec_method_2(iter: &mut ProdIter<SharedStackRBMut<usize, 1024>>) {
    let _ = iter.push(1);
}
fn call_iter_spec_method_3(
    iter: &mut ProdIter<OneRingBuf<StackStorage<usize, 1024>, SharedCompMut>>,
) {
    let _ = iter.push(1);
}

async fn async_fn() {
    let mut buf = oneringbuf::AsyncStackRB::<usize, 10>::default();

    let (prod, cons) = buf.split_async();

    let mut sync_prod = prod.into_sync();
    sync_prod.push(1).unwrap();

    let mut as_prod = sync_prod.into_async();
    as_prod.push(2).await.unwrap();

    let mut sync_cons = cons.into_sync();
    assert_eq!(sync_cons.pop().unwrap(), 1);

    let mut as_cons = sync_cons.into_async();
    assert_eq!(as_cons.pop().await.unwrap(), 2);
}

#[tokio::main]
async fn main() {
    let mut buf = SharedStackRBMut::<usize, 1024>::default();

    call_buffer_common_method(&buf);

    let (mut prod, _work, _cons) = buf.split_mut();

    call_iter_common_method(&prod);
    call_iter_spec_method(&mut prod);
    call_iter_spec_method_2(&mut prod);
    call_iter_spec_method_3(&mut prod);

    async_fn().await;
}
