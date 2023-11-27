// use crate::modules::application::dependency_fragment::DependencyFragment;
// use crate::modules::application::traits::executable_fragment::ExecutableFragment;
// use crate::modules::application::MobileFragments;
// use itertools::Itertools;
// use std::collections::{HashMap, HashSet};
//
// pub fn resolve_indirect_dependencies<T: ExecutableFragment>(
//     executables: &mut [T],
//     dependencies: &[DependencyFragment],
// ) {
//     // Create a map combining dependencies from DependencyFragments
//     let mut all_fragments_map: HashMap<String, Vec<String>> = HashMap::new();
//
//     for df in dependencies.iter() {
//         all_fragments_map.insert(df.id.clone(), df.dependencies.clone().into_iter().collect());
//     }
//
//     for executable_fragment in executables.iter_mut() {
//         let mut visited = HashSet::new();
//
//         // Find all dependencies of the current executable fragment
//         let all_dependencies = find_all_dependencies(
//             &executable_fragment
//                 .get_dependency_ids()
//                 .into_iter()
//                 .collect(),
//             &all_fragments_map,
//             &mut visited,
//         );
//
//         executable_fragment.set_dependency_ids(all_dependencies.into_iter().sorted().collect());
//     }
// }
//
// fn find_all_dependencies(
//     deps: &Vec<String>,
//     all_fragments_map: &HashMap<String, Vec<String>>,
//     visited: &mut HashSet<String>,
// ) -> HashSet<String> {
//     let mut result = HashSet::new();
//
//     for dep_id in deps {
//         // Avoid revisiting dependencies
//         if visited.contains(dep_id) {
//             continue;
//         }
//         visited.insert(dep_id.clone());
//
//         result.insert(dep_id.clone());
//
//         if let Some(sub_deps) = all_fragments_map.get(dep_id) {
//             result.extend(find_all_dependencies(sub_deps, all_fragments_map, visited));
//         }
//     }
//
//     result
// }
//
// pub fn resolve(mobile_fragments: &mut MobileFragments) {
//     resolve_indirect_dependencies(
//         &mut mobile_fragments.functions,
//         &mobile_fragments.dependencies,
//     );
//     resolve_indirect_dependencies(&mut mobile_fragments.impls, &mobile_fragments.dependencies);
// }
