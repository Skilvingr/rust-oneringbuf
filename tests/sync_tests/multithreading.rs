extern crate alloc;

use crate::{common_def, get_buf};
use oneringbuf::ORBIterator;
use std::thread;
use std::time::Instant;

common_def!();

macro_rules! get_prod {
    ($s: ident, $prod: ident) => {
        $s.spawn(move || {
            let start = Instant::now();

            while start.elapsed().as_millis() < 5 {
                $prod.push_slice(&[0; 3000]);
            }
        })
    };
}
macro_rules! get_work {
    ($s: ident, $work: ident) => {
        $s.spawn(move || {
            let start = Instant::now();

            while start.elapsed().as_millis() < 5 {
                let avail = $work.available();
                unsafe {
                    $work.advance(avail);
                }
            }
        })
    };
}
macro_rules! get_cons {
    ($s: ident, $cons: ident) => {
        $s.spawn(move || {
            let start = Instant::now();

            while start.elapsed().as_millis() < 5 {
                let avail = $cons.available();
                unsafe {
                    $cons.advance(avail);
                }
            }
        })
    };
}

#[test]
fn test_mt_non_mut() {
    let mut buf = get_buf!(Shared);
    let (mut prod, mut cons) = buf.split();

    let mut buf = get_buf!(Shared);

    thread::scope(|s| {
        let producer = get_prod!(s, prod);
        let consumer = get_cons!(s, cons);

        let _ = producer.join();
        let _ = consumer.join();

        let (mut prod, mut cons) = buf.split();

        let producer = get_prod!(s, prod);
        let consumer = get_cons!(s, cons);

        let _ = producer.join();
        let _ = consumer.join();
    });
}

#[test]
fn test_mt_mut() {
    let mut buf = get_buf!(SharedMut);
    let (mut prod, mut work, mut cons) = buf.split_mut();

    let mut buf = get_buf!(SharedMut);

    thread::scope(|s| {
        let producer = get_prod!(s, prod);
        let worker = get_work!(s, work);
        let consumer = get_cons!(s, cons);

        let _ = producer.join();
        let _ = worker.join();
        let _ = consumer.join();

        let (mut prod, mut work, mut cons) = buf.split_mut();

        let producer = get_prod!(s, prod);
        let worker = get_work!(s, work);
        let consumer = get_cons!(s, cons);

        let _ = producer.join();
        let _ = worker.join();
        let _ = consumer.join();
    });
}
