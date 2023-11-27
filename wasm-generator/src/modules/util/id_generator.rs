use crate::modules::application::traits::fragment::Fragment;
use log::error;
use std::collections::{HashMap, HashSet};
use std::process::exit;

pub fn check_duplicates<T: Fragment>(fragments: &mut [T]) {
    // check for duplicate IDs in the executable_fragments
    let mut id_counts: HashMap<String, usize> = HashMap::new();
    for id in fragments.iter().map(|x| x.get_id()) {
        if !id.is_empty() {
            *id_counts.entry(id).or_insert(0) += 1;
        }
    }
    let duplicate_ids: Vec<String> = id_counts
        .into_iter()
        .filter_map(|(id, count)| if count > 1 { Some(id) } else { None })
        .collect();
    if !duplicate_ids.is_empty() {
        error!("Duplicate IDs found for fragment: {:?}", duplicate_ids);
        exit(1);
    }
}

pub fn assign_missing_ids<T: Fragment>(
    fragments: &mut [T],
    id_generator: &mut FragmentIdGenerator,
) {
    // Initialize ids with all currently present IDs for next id generation
    let ids: HashSet<String> = fragments.iter().map(|f| f.get_id()).collect();
    id_generator.id_registry.extend(ids);
    for fragment in fragments.iter_mut() {
        if fragment.get_id() == "" {
            let id = id_generator.generate(fragment);
            fragment.set_id(id);
        } else {
            if let Some(first_char) = fragment.get_id().chars().next() {
                if first_char.to_string().parse::<u8>().is_ok() {
                    let id = format!("a_{}", fragment.get_id());
                    fragment.set_id(id);
                }
            }
            id_generator.id_registry.insert(fragment.get_id());
        }
    }
}

#[derive(Default)]
pub struct NumericIdGenerator {
    id_registry: HashSet<u32>,
}

impl NumericIdGenerator {
    pub fn next_id(&mut self) -> u32 {
        let id_set: HashSet<u32> = self.id_registry.iter().cloned().collect();
        let next_id = (1..).find(|&id| !id_set.contains(&id)).unwrap();
        self.id_registry.insert(next_id);
        next_id
    }
}

#[derive(Default)]
pub struct FragmentIdGenerator {
    id_registry: HashSet<String>,
}

impl FragmentIdGenerator {
    pub fn generate(&mut self, fragment: &impl Fragment) -> String {
        let mut id = fragment.get_name();
        let mut counter = 1;

        // If the ID already exists, append a number to make it unique
        while self.id_registry.contains(&id) {
            id = format!("{}{}", fragment.get_name(), counter);
            counter += 1;
        }
        self.id_registry.insert(id.clone());
        id
    }
}

#[cfg(test)]
mod tests {
    use crate::modules::util::id_generator::NumericIdGenerator;

    #[test]
    pub fn test_generate_next_id() {
        let mut id_generator = NumericIdGenerator::default();
        id_generator.id_registry.extend(vec![1, 2, 4, 5, 8]);

        let next_id = id_generator.next_id();
        assert_eq!(next_id, 3);
        id_generator.id_registry.insert(next_id);

        let next_id = id_generator.next_id();
        assert_eq!(next_id, 6);
        id_generator.id_registry.insert(next_id);

        let next_id = id_generator.next_id();
        assert_eq!(next_id, 7);
        id_generator.id_registry.insert(next_id);

        let next_id = id_generator.next_id();
        assert_eq!(next_id, 9);
        id_generator.id_registry.insert(next_id);

        let next_id = id_generator.next_id();
        assert_eq!(next_id, 10);
        id_generator.id_registry.insert(next_id);
    }
}
