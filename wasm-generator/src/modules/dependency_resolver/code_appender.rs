use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
/// This struct is used to append code by keeping track of module hierarchy.
pub struct CodeAppender {
    code: String,
    children: HashMap<String, CodeAppender>,
}

impl CodeAppender {
    pub fn insert(&mut self, hierarchy: &[String], code: &str) {
        if hierarchy.is_empty() {
            self.code.push_str(code);
            self.code.push('\n');
        } else {
            let child_name = &hierarchy[0];
            let child = self.children.entry(child_name.clone()).or_default();
            child.insert(&hierarchy[1..], code);
        }
    }

    pub fn generate_code(&self) -> String {
        let mut result = String::new();
        for (name, child) in &self.children {
            result.push_str(&format!("pub mod {} {{\n", name));
            result.push_str("use super::*;\n");
            result.push_str(&child.generate_code());
            result.push_str("}\n");
        }
        result.push_str(&self.code);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::source_code_analyzer::types::RustItemCommonProperties;

    #[test]
    fn test_append_codes() {
        let items = vec![
            RustItemCommonProperties {
                name: "Item1".to_string(),
                code: "fn function1() {}".to_string(),
                module_hierarchy: vec!["mod1".to_string(), "mod2".to_string()],
                ..Default::default()
            },
            RustItemCommonProperties {
                name: "Item2".to_string(),
                code: "fn function2() {}".to_string(),
                module_hierarchy: vec!["mod1".to_string(), "mod3".to_string()],
                ..Default::default()
            },
            RustItemCommonProperties {
                name: "Item3".to_string(),
                code: "fn function3() {}".to_string(),
                module_hierarchy: vec!["mod1".to_string(), "mod2".to_string()],
                ..Default::default()
            },
        ];

        let mut code_appender = CodeAppender::default();
        for item in items {
            code_appender.insert(&item.module_hierarchy, &item.code);
        }
        println!("combined_code: {}", code_appender.generate_code());
    }
}
