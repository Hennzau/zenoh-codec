// use criterion::Criterion;

// use crate::field::ZField;

// pub(super) fn criterion_benchmark(c: &mut Criterion) {
//     let mut data = [0u8; 16];
//     c.bench_function("Encode u64", |b| {
//         b.iter(|| {
//             let mut w = &mut data.as_mut_slice();

//             u64::MAX.z_encode(&mut w).unwrap();

//             let mut r = data.as_slice();
//             let _: u64 = <u64 as ZField>::z_decode(&mut r).unwrap();
//         })
//     });
// }

// #[test]
// #[ignore]
// fn bench() {
//     let mut c = Criterion::default().with_output_color(true).without_plots();

//     criterion_benchmark(&mut c);

//     Criterion::default().final_summary();
// }
