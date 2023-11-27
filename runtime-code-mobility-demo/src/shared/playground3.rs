// // use crate::shared::playground1::{get_struct_test, Test};
// use crate::shared::playground1::get_struct_test;
// use crate::shared::playground2::{area_of_circle, CustomString};
// use crate::shared::playground4::hello_world;
//
// pub mod whatever {
//     pub fn hello() {
//         println!("Hello from whatever module");
//     }
// }
//
// /// @mobile
// pub fn test_imported_from_another_module() -> CustomString {
//     let area = area_of_circle(2.0);
//     let a = CustomString::from(format!("Hello, world! Area: {}", area));
//     a
// }
//
// /// @mobile(crates = ["itertools"])
// pub fn invoke_itertools_demo() {
//     crate::shared::playground1::itertools_demo();
//     whatever::hello();
// }
//
// /// @mobile
// pub fn use_test_struct() -> String {
//     let t = get_struct_test();
//     hello_world();
//     t.b
// }
