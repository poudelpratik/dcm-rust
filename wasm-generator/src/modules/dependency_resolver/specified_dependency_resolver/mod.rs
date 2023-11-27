// use crate::modules::application::MobileFragments;
// use crate::modules::source_code_analyzer::cargo_toml::ProjectCargoToml;
// use std::sync::Arc;
// 
// pub mod append_code;
// pub mod resolve_indirect_dependencies;
// 
// pub fn run(mobile_fragments: &mut MobileFragments, project_cargo_toml: Arc<ProjectCargoToml>) {
//     // append_code::link_impls_with_structs(mobile_fragments);
//     // resolve_indirect_dependencies::resolve(mobile_fragments);
//     // extend_crates_from_dependencies(mobile_fragments);
//     // set_cargo_toml(mobile_fragments, project_cargo_toml.clone());
//     append_dependency_code(mobile_fragments);
// }
// 
// pub fn append_dependency_code(mobile_fragments: &mut MobileFragments) {
//     append_code::extend_code_from_dependencies(
//         &mut mobile_fragments.functions,
//         &mobile_fragments.dependencies,
//     );
//     append_code::extend_code_from_dependencies(
//         &mut mobile_fragments.impls,
//         &mobile_fragments.dependencies,
//     );
// }
// 
// pub fn extend_crates_from_dependencies(mobile_fragments: &mut MobileFragments) {
//     append_code::extend_crates_from_dependencies(
//         &mut mobile_fragments.functions,
//         &mobile_fragments.dependencies,
//     );
//     append_code::extend_crates_from_dependencies(
//         &mut mobile_fragments.impls,
//         &mobile_fragments.dependencies,
//     );
// }
// 
// fn set_cargo_toml(
//     mobile_fragments: &mut MobileFragments,
//     project_cargo_toml: Arc<ProjectCargoToml>,
// ) {
//     append_code::set_cargo_toml(&mut mobile_fragments.functions, project_cargo_toml.clone());
//     append_code::set_cargo_toml(&mut mobile_fragments.impls, project_cargo_toml.clone());
// }
