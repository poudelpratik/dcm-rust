// // use crate::shared::playground2::{area_of_circle, square, CustomString};
// // use crate::shared::playground4::fibonacci;
// use crate::shared::playground2::CustomString;
// use itertools::Itertools;
//
// // /// @mobile
// // pub fn generate_random_number() -> u32 {
// //     let num = square(2.0);
// //     let fib = fibonacci(5);
// //     (num as u32) * fib
// // }
// //
// /// @mobile
// pub struct Test {
//     a: i32,
//     pub(crate) b: CustomString,
//     c: Vec<i32>,
// }
// //
// // /// @mobile
// pub fn get_struct_test() -> Test {
//     Test {
//         a: 1,
//         b: CustomString::from("Hello"),
//         c: vec![1, 2, 3],
//     }
// }
//
// pub fn itertools_demo() {
//     let nums1 = vec![1, 2, 3];
//     let products = nums1
//         .iter()
//         .cartesian_product(nums1.iter())
//         .map(|(&a, &b)| a * b)
//         .collect::<Vec<_>>();
//     println!("Products: {:?}", products);
// }
//
// /// @mobile
// impl Test {
//     pub fn new() -> Self {
//         Self {
//             a: 1,
//             b: "Hello".into(),
//             c: vec![1, 2, 3],
//         }
//     }
//
//     pub fn set_a(&mut self, a: i32) {
//         self.a = a;
//     }
//
//     pub fn combine_with_another(&self, other: Test) -> Test {
//         Test {
//             a: self.a + other.a,
//             b: format!("{}{}", self.b, other.b),
//             c: [&self.c[..], &other.c[..]].concat(),
//         }
//     }
// }
//
// pub struct Test {
//     a: i32,
//     b: String,
//     c: Vec<i32>,
// }
//
// /// @mobile
// impl Test {
//     pub fn new() -> Test {
//         Test {
//             a: 1,
//             b: "Hello".to_string(),
//             c: vec![1, 2, 3],
//         }
//     }
//
//     pub fn from_values(a: i32, b: String, c: Vec<i32>) -> Test {
//         Test { a, b, c }
//     }
//
//     pub fn get_a(&self) -> i32 {
//         self.a
//     }
//
//     pub fn set_a(&mut self, a: i32) {
//         self.a = a;
//     }
// }
