use std::time::Duration;

use async_scoped::TokioScope;
use oneringbuf::iterators::async_iterators::AsyncIterator;

use crate::common_def;

common_def!();

#[tokio::test(flavor = "multi_thread")]
async fn test_full() {
    #[cfg(all(not(feature = "alloc"), not(all(feature = "vmem", unix))))]
    let mut buf = oneringbuf::AsyncStackRB::from([0; BUFFER_SIZE]);
    #[cfg(all(feature = "alloc", not(all(feature = "vmem", unix))))]
    let buf = oneringbuf::AsyncHeapRB::from(vec![0; BUFFER_SIZE]);
    #[cfg(all(feature = "alloc", feature = "vmem", unix))]
    let buf = oneringbuf::AsyncVmemRB::from(vec![0; BUFFER_SIZE]);

    let (mut as_prod, mut as_cons) = buf.split_async();

    TokioScope::scope_and_block(|s| {
        let slice: Vec<i32> = (0..BUFFER_SIZE as i32 - 1).collect();

        let clone = slice.clone();
        s.spawn(async move {
            as_prod.push_slice(&clone).await;
        });

        s.spawn(async move {
            tokio::time::sleep(Duration::from_secs(2)).await;
            let _ = as_cons.peek_available().await;
            unsafe {
                as_cons.advance(BUFFER_SIZE - 1);
            }
            println!("ADVANCED");
        });
    });
}

#[tokio::test]
async fn test_push_work_pop_single_and_slice() {
    #[cfg(all(not(feature = "alloc"), not(all(feature = "vmem", unix))))]
    let mut buf = oneringbuf::AsyncStackRBMut::from([0; BUFFER_SIZE]);
    #[cfg(all(feature = "alloc", not(all(feature = "vmem", unix))))]
    let buf = oneringbuf::AsyncHeapRBMut::from(vec![0; BUFFER_SIZE]);
    #[cfg(all(feature = "alloc", feature = "vmem", unix))]
    let buf = oneringbuf::AsyncVmemRBMut::from(vec![0; BUFFER_SIZE]);

    let (mut as_prod, mut as_work, mut as_cons) = buf.split_async_mut();

    as_prod.push(1).await;

    if let Some(res) = as_work.get_mut().await {
        *res += 1;
        unsafe {
            as_work.advance(1);
        }
    }

    let res = as_cons.peek_ref().await.unwrap();
    assert_eq!(res, &2);
    unsafe {
        as_cons.advance(1);
    }

    let slice: Vec<i32> = (0..BUFFER_SIZE as i32 / 2).collect();
    as_prod.push_slice(&slice).await;

    #[cfg(not(all(feature = "vmem", unix)))]
    if let Some((h, t)) = as_work.get_mut_slice_avail().await {
        let len = h.len() + t.len();

        for x in h.iter_mut().chain(t) {
            *x += 1;
        }

        unsafe {
            as_work.advance(len);
        }
    }

    #[cfg(all(feature = "vmem", unix))]
    if let Some(r) = as_work.get_mut_slice_avail().await {
        let len = r.len();

        for x in r {
            *x += 1;
        }

        unsafe {
            as_work.advance(len);
        }
    }

    #[cfg(not(all(feature = "vmem", unix)))]
    if let Some((h, t)) = as_cons.peek_available().await {
        for (x, y) in h.iter().chain(t).zip(&slice) {
            assert_eq!(*x, y + 1);
        }
    }

    #[cfg(all(feature = "vmem", unix))]
    if let Some(r) = as_cons.peek_available().await {
        for (x, y) in r.iter().zip(&slice) {
            assert_eq!(*x, y + 1);
        }
    }

    drop(as_prod);
    assert_eq!(as_cons.alive_iters(), 2);
    drop(as_work);
    assert_eq!(as_cons.alive_iters(), 1);
}
