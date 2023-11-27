use crate::modules::application::traits::fragment::Fragment;
use crate::modules::source_code_analyzer::cargo_toml::{CargoPackageInformation, ProjectCargoToml};

const DEFAULT_CRATES: [(&str, &str); 4] = [
    ("serde_json", "1.0.104"),
    ("rmp-serde", "1.1.2"),
    ("serde_derive", "1.0.163"),
    ("serde", "1.0.163"),
];

pub fn set_cargo_toml<T: Fragment>(fragment: &mut T, project_cargo_toml: &ProjectCargoToml) {
    let mut cargo_dependencies: toml::Table =
        project_cargo_toml
            .dependencies
            .as_ref()
            .map_or(toml::Table::new(), |deps| {
                deps.iter()
                    .filter(|(k, v)| {
                        // Keep if the crate exactly matches the string in the list from fragment and does not have a local path.
                        fragment
                            .get_crates()
                            .iter()
                            .any(|crate_name| crate_name == *k)
                            && match v {
                                toml::Value::Table(table) => !table.contains_key("path"),
                                _ => true,
                            }
                    })
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            });

    // add default dependencies
    for (k, v) in DEFAULT_CRATES {
        if !cargo_dependencies.contains_key(k) {
            cargo_dependencies.insert(k.to_string(), toml::Value::String(v.to_string()));
        }
    }

    let cargo_package = CargoPackageInformation::new(
        fragment.get_package_name(),
        project_cargo_toml.package.version.clone(),
        project_cargo_toml.package.authors.clone(),
        project_cargo_toml.package.edition.clone(),
    );

    let mut cargo_toml = ProjectCargoToml {
        package: cargo_package,
        dependencies: cargo_dependencies.into(),
        lib: toml::Table::new().into(),
    };

    // Add crate-type to the Cargo.toml
    cargo_toml.lib.as_mut().map(|lib| {
        lib.insert(
            "crate-type".to_string(),
            toml::Value::Array(vec![toml::Value::String("cdylib".to_string())]),
        )
    });

    fragment.set_cargo_toml(cargo_toml);
}
