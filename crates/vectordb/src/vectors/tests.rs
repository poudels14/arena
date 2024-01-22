// #[cfg(test)]
// mod tests {
//   use glam::Vec4;

//   #[test]
//   fn test_glam() {
//     let v1: Vec<f32> = vec![0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.2, 0.2];
//     let v2: Vec<f32> = vec![1., 1., 1., 1., 1., 1., 1., 1.];

//     let glam_dot = v1
//       .chunks_exact(4)
//       .map(Vec4::from_slice)
//       .zip(v2.chunks_exact(4).map(Vec4::from_slice))
//       .map(|(a, b)| a.dot(b))
//       .sum::<f32>();

//     let simple_dot_product =
//       v1.iter().zip(&v2).map(|(a, b)| a * b).sum::<f32>();

//     // yolo comparing floats
//     assert_eq!(glam_dot, 1.0);
//     assert_eq!(simple_dot_product, 1.0);
//   }
// }
