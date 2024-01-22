use criterion::{criterion_group, criterion_main, Criterion};
use glam::f32::Vec4;
// use nalgebra::{DMatrix, Matrix1x4};
// use packed_simd_2::f32x4;
use rand::Rng;

fn dot_product_benchmark(c: &mut Criterion) {
  let mut group = c.benchmark_group("dot_product_benchmark");
  const DIM: usize = 384;
  const ITER: usize = 30_000;

  let mut rng = rand::thread_rng();
  let v1: Vec<f32> = (0..DIM).map(|_| rng.gen_range(0.0..100.0)).collect();
  let v2: Vec<Vec<f32>> = (0..100)
    .map(|_| (0..DIM).map(|_| rng.gen_range(0.0..100.0)).collect())
    .collect();

  group.bench_function("simple", |b| {
    b.iter(|| {
      for i in 0..ITER {
        let _ = v1.iter().zip(&v2[i % 100]).map(|(a, b)| a * b).sum::<f32>();
        // assert!(r - result[i] < 1.0 || r - result[i] > -1.);
      }
    });
  });

  group.bench_function("glam", |b| {
    b.iter(|| {
      for i in 0..ITER {
        let _ = v1
          .chunks_exact(4)
          .map(Vec4::from_slice)
          .zip(v2[i % 100].chunks_exact(4).map(Vec4::from_slice))
          .map(|(a, b)| a.dot(b))
          .sum::<f32>();
        // assert!(r - result[i] < 1. || r - result[i] > -1.);
      }
    });
  });

  // group.bench_function("nalgebra column slice", |b| {
  //   b.iter(|| {
  //     for i in 0..ITER {
  //       let _ = DMatrix::from_column_slice(DIM, 1, &v1)
  //         .dot(&DMatrix::from_column_slice(DIM, 1, &v2[i % 100]));
  //       // assert!(r - result[i] < 1. || r - result[i] > -1.);
  //     }
  //   });
  // });

  // group.bench_function("nalgebra row slice", |b| {
  //   b.iter(|| {
  //     for i in 0..ITER {
  //       let _ = DMatrix::from_row_slice(DIM, 1, &v1)
  //         .dot(&DMatrix::from_row_slice(DIM, 1, &v2[i % 100]));
  //       // assert!(r - result[i] < 1. || r - result[i] > -1.);
  //     }
  //   });
  // });

  // group.bench_function("nalgebra chunked", |b| {
  //   b.iter(|| {
  //     for i in 0..ITER {
  //       let _: f32 = v1
  //         .chunks_exact(4)
  //         .map(Matrix1x4::from_row_slice)
  //         .zip(v2[i % 100].chunks_exact(4).map(Matrix1x4::from_row_slice))
  //         .map(|(a, b)| a.dot(&b))
  //         .sum();

  //       // assert!(r - result[i] < 1. || r - result[i] > -1.);
  //     }
  //   });
  // });

  group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = dot_product_benchmark
}

criterion_main!(benches);
