use crate::common_def;
use oneringbuf::ORBIterator;

common_def!();

macro_rules! test_buf {
    ($v: tt, $prod: tt) => {
        #[cfg(not(all(feature = "vmem", unix)))]
        let b = {
            let (h, t) = $prod.get_mut_slice_avail().unwrap();
            h.iter().chain(t.iter()).map(|e| *e).collect::<Vec<usize>>()
        };

        #[cfg(all(feature = "vmem", unix))]
        let b = $prod.get_mut_slice_avail().unwrap();

        for (be, ve) in b.iter().zip($v.iter()) {
            assert_eq!(*ve, *be);
        }
    };
}

#[test]
fn test_new_buf() {
    let v: [usize; BUFFER_SIZE] = (0..BUFFER_SIZE).collect::<Vec<usize>>().try_into().unwrap();

    #[cfg(all(feature = "alloc", feature = "vmem", unix))]
    {
        use oneringbuf::{LocalVmemRB, SharedVmemRB};
        let (mut prod, _) = SharedVmemRB::from(v.clone().to_vec()).split();
        test_buf!(v, prod);

        let (mut prod, _) = LocalVmemRB::from(v.clone().to_vec()).split();
        test_buf!(v, prod);
    }

    #[cfg(all(feature = "alloc", not(all(feature = "vmem", unix))))]
    {
        use oneringbuf::{LocalHeapRB, SharedHeapRB};
        let (mut prod, _) = SharedHeapRB::from(v.clone().to_vec()).split();
        test_buf!(v, prod);

        let (mut prod, _) = LocalHeapRB::from(v.clone().to_vec()).split();
        test_buf!(v, prod);
    }

    #[cfg(all(not(feature = "alloc"), not(all(feature = "vmem", unix))))]
    {
        use oneringbuf::{LocalStackRB, SharedStackRB};

        let mut buf = SharedStackRB::from(v.clone());
        let (mut prod, _) = buf.split();
        test_buf!(v, prod);

        let mut buf = LocalStackRB::from(v.clone());
        let (mut prod, _) = buf.split();
        test_buf!(v, prod);
    }
}
