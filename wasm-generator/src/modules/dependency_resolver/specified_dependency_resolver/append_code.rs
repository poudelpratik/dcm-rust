// use crate::modules::application::dependency_fragment::DependencyFragment;
// use crate::modules::application::fragment_type::RustItemType;
// use crate::modules::application::traits::executable_fragment::IsExecutable;
// use crate::modules::application::traits::fragment::Fragment;
// use crate::modules::application::MobileFragments;
// use crate::modules::source_code_analyzer::cargo_toml::{CargoPackageInformation, ProjectCargoToml};
// use std::collections::HashMap;
// use std::sync::Arc;
//
// pub fn extend_code_from_dependencies<T: IsExecutable + Fragment>(
//     final_fragment_data: &mut [T],
//     dependency_fragments: &[DependencyFragment],
// ) {
//     for executable_fragment in final_fragment_data.iter_mut() {
//         let mut dependency_code = String::new();
//         let filtered_dependency_fragments =
//             get_dependency_fragments(executable_fragment, dependency_fragments);
//         for dependency_fragment in filtered_dependency_fragments.iter() {
//             dependency_code.push_str(format!("{}\n\n", dependency_fragment.code).as_str());
//         }
//         executable_fragment.set_code(format!(
//             "{}\n{}",
//             executable_fragment.get_code(),
//             dependency_code
//         ));
//     }
// }
//
// fn get_dependency_fragments<T: IsExecutable>(
//     final_fragment_data: &mut T,
//     dependency_fragments: &[DependencyFragment],
// ) -> Vec<DependencyFragment> {
//     dependency_fragments
//         .iter()
//         .filter(|&dependency_fragment| {
//             final_fragment_data
//                 .get_dependency_ids()
//                 .contains(&dependency_fragment.id)
//         })
//         .cloned()
//         .collect::<Vec<DependencyFragment>>()
// }
//
// pub fn extend_crates_from_dependencies<T: IsExecutable>(
//     final_fragment_data: &mut [T],
//     dependency_fragments: &[DependencyFragment],
// ) {
//     for executable_fragment in final_fragment_data.iter_mut() {
//         let mut crates = executable_fragment.get_crates().clone();
//         let filtered_dependency_fragments =
//             get_dependency_fragments(executable_fragment, dependency_fragments);
//         for dependency_fragment in filtered_dependency_fragments.iter() {
//             crates.extend(dependency_fragment.crates.clone());
//         }
//         crates.sort();
//         crates.dedup();
//         executable_fragment.set_crates(crates);
//     }
// }
//
// pub fn set_cargo_toml<T: IsExecutable>(
//     fragments: &mut [T],
//     project_cargo_toml: Arc<ProjectCargoToml>,
// ) {
//     for fragment in fragments.iter_mut() {
//         let default_deps: HashMap<String, toml::Value> = [
//             (
//                 "rmp-serde".to_string(),
//                 toml::Value::String("1.1.2".to_string()),
//             ),
//             (
//                 "serde_derive".to_string(),
//                 toml::Value::String("1.0.163".to_string()),
//             ),
//             (
//                 "serde".to_string(),
//                 toml::Value::String("1.0.163".to_string()),
//             ),
//             (
//                 "serde_json".to_string(),
//                 toml::Value::String("1.0.104".to_string()),
//             ),
//         ]
//         .iter()
//         .cloned()
//         .collect();
//
//         let mut cargo_dependencies: toml::Table =
//             project_cargo_toml
//                 .dependencies
//                 .as_ref()
//                 .map_or(toml::Table::new(), |deps| {
//                     deps.iter()
//                         .filter(|(k, _)| fragment.get_crates().contains(k))
//                         .map(|(k, v)| (k.clone(), v.clone()))
//                         .collect()
//                 });
//         cargo_dependencies.extend(default_deps);
//
//         let cargo_package = CargoPackageInformation::new(
//             fragment.get_package_name(),
//             project_cargo_toml.package.version.clone(),
//             project_cargo_toml.package.authors.clone(),
//             project_cargo_toml.package.edition.clone(),
//         );
//
//         let deps = match project_cargo_toml.dependencies.clone() {
//             None => None,
//             Some(mut dependencies) => {
//                 dependencies.retain(|_, value| {
//                     if let toml::Value::Table(ref table) = value {
//                         !table.contains_key("path")
//                     } else {
//                         true
//                     }
//                 });
//                 Some(dependencies)
//             }
//         };
//         let mut cargo_toml = ProjectCargoToml {
//             package: cargo_package,
//             dependencies: cargo_dependencies.into(),
//             lib: toml::Table::new().into(),
//         };
//
//         // Add crate-type to the Cargo.toml
//         cargo_toml.lib.as_mut().map(|lib| {
//             lib.insert(
//                 "crate-type".to_string(),
//                 toml::Value::Array(vec![toml::Value::String("cdylib".to_string())]),
//             )
//         });
//
//         fragment.set_cargo_toml(cargo_toml);
//     }
// }
//
// // pub fn link_impls_with_structs(mobile_fragments: &mut MobileFragments) {
// //     for impl_fragment in mobile_fragments.impls.iter_mut() {
// //         let mut struct_found = false;
// //         for struct_fragment in mobile_fragments.structs.iter_mut() {
// //             if struct_fragment.dependencies.contains(&impl_fragment.id) {
// //                 impl_fragment.rust_struct = struct_fragment.rust_struct.clone();
// //                 // impl_fragment.rust_struct.properties.code = format!(
// //                 //     "#[derive(Serialize, Deserialize)]\n{}",
// //                 //     impl_fragment.rust_struct.properties.code
// //                 // );
// //                 impl_fragment.code = format!(
// //                     "{}\n\n{}",
// //                     impl_fragment.rust_struct.properties.code, impl_fragment.code
// //                 );
// //                 struct_found = true;
// //                 break;
// //             }
// //         }
// //         if !struct_found {
// //             panic!("No struct found for impl fragment: {}", impl_fragment.name);
// //         }
// //     }
// //
// //     for dependency_fragment in mobile_fragments.dependencies.iter_mut() {
// //         if dependency_fragment.fragment_type == RustItemType::Struct {
// //             dependency_fragment.code = format!(
// //                 "#[derive(Serialize, Deserialize, Debug)]\n{}",
// //                 dependency_fragment.code
// //             );
// //         }
// //     }
// // }
