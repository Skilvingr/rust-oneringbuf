use divan::black_box;
use oneringbuf::LocalStackRB;

const BUFFER_SIZE: usize = 4096;
const BATCH_SIZE: usize = 100;

fn main() {
    divan::main();
}

#[divan::bench(sample_size = 100000)]
fn push_pop_x100(b: divan::Bencher) {
    let mut buf = LocalStackRB::<i32, BUFFER_SIZE>::default();

    let (mut prod, mut cons) = buf.split();

    prod.push_slice(&[1; BUFFER_SIZE / 2]);

    b.bench_local(|| {
        for _ in 0..BATCH_SIZE {
            prod.push(1).unwrap();
        }
        for _ in 0..BATCH_SIZE {
            black_box(cons.pop().unwrap());
        }
    });
}
