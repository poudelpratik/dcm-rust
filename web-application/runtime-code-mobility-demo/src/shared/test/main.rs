// use crate::shared::test::stats;
// use crate::shared::test::stats::Number;
// use itertools::Itertools;
//
// /// @mobile(crates = ["itertools"])
// fn get_processed_data(data: Vec<Number>) -> Vec<Number> {
//     process_data(&data)
// }
//
// fn process_data(data: &[Number]) -> Vec<Number> {
//     let mean_val = stats::mean(data);
//     let var_val = stats::variance(data);
//
//     data.iter()
//         .cloned()
//         .filter(|&x| x < mean_val)
//         .map(|x| x + var_val)
//         .sorted()
//         .collect()
// }
