// use crate::shared::playground1::Test;
// use quote::quote;
//
// pub type CustomString = String;
//
// /// @mobile
// pub fn square(num: f64) -> f64 {
//     num * num
// }
//
// /// @mobile
// pub fn area_of_circle(radius: f64) -> f64 {
//     PI * square(radius)
// }
//
// const PI: f64 = std::f64::consts::PI;
//
// /// @mobile(crates = ["quote"])
// pub fn generate_struct_code() -> CustomString {
//     let tokens = quote! {
//         struct GeneratedStruct {
//             data: String,
//         }
//     };
//     tokens.to_string()
// }
//
// /// @mobile
// pub fn create_test_struct() -> Test {
//     Test::new()
// }
