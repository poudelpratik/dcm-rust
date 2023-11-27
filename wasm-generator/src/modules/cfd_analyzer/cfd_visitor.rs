use crate::modules::application::fragment_type::RustItemType;
use crate::modules::application::function_fragment::FunctionFragment;
use crate::modules::application::object_fragment::ObjectFragment;
use crate::modules::application::MobileFragments;
use crate::modules::cfd_analyzer::traits::visit::Visit;
use crate::modules::cfd_analyzer::CodeFragmentDescription;
use crate::modules::source_code_analyzer::rust_file::RustFile;
use crate::modules::source_code_analyzer::traits::rust_item::RustItem;
use std::sync::Arc;

pub struct CfdVisitor<'a> {
    pub rust_files: Arc<Vec<RustFile>>,
    pub mobile_fragments: &'a mut MobileFragments,
    pub cfd_errors: Vec<String>,
}

impl<'a> CfdVisitor<'a> {
    pub fn new(mobile_fragments: &'a mut MobileFragments, rust_files: Arc<Vec<RustFile>>) -> Self {
        Self {
            rust_files,
            mobile_fragments,
            cfd_errors: Vec::new(),
        }
    }
}

impl<'a> Visit for CfdVisitor<'a> {
    fn visit_function(&mut self, cfd_item: &CodeFragmentDescription) {
        if let Some(rf) =
            find_containing_file_in_syntax_tree(&self.rust_files, cfd_item, &mut self.cfd_errors)
        {
            match rf
                .functions
                .iter()
                .find(|item| item.get_common_properties().name == cfd_item.name)
            {
                None => {
                    self.cfd_errors
                        .push(format!("Function `{}` not found.", &cfd_item.name));
                }
                Some(item) => {
                    self.mobile_fragments
                        .functions
                        .push(FunctionFragment::create_from_cfd(
                            item.clone(),
                            cfd_item,
                            item.get_common_properties().module_hierarchy.clone(),
                        ));
                }
            }
        };
    }

    fn visit_impl(&mut self, cfd_item: &CodeFragmentDescription) {
        if let Some(rf) =
            find_containing_file_in_syntax_tree(&self.rust_files, cfd_item, &mut self.cfd_errors)
        {
            match rf
                .impls
                .iter()
                .find(|item| item.get_common_properties().name == cfd_item.name)
            {
                None => {
                    self.cfd_errors
                        .push(format!("Function `{}` not found.", &cfd_item.name));
                }
                Some(item) => {
                    self.mobile_fragments
                        .impls
                        .push(ObjectFragment::create_from_cfd(
                            item.clone(),
                            cfd_item,
                            item.get_common_properties().module_hierarchy.clone(),
                        ));
                }
            }
        };
    }
}

pub fn visit_cfd(visitor: &mut dyn Visit, items: &[CodeFragmentDescription]) {
    for item in items {
        match item.item_type.clone().unwrap_or_default() {
            RustItemType::Function => visitor.visit_function(item),
            RustItemType::Impl => visitor.visit_impl(item),
            _ => {}
        }
    }
}

pub fn find_containing_file_in_syntax_tree(
    rust_files: &[RustFile],
    cfd_item: &CodeFragmentDescription,
    cfd_errors: &mut Vec<String>,
) -> Option<RustFile> {
    let rust_file = rust_files
        .iter()
        .find(|rf| rf.relative_filepath == cfd_item.location.filepath)
        .cloned();
    if rust_file.is_none() {
        cfd_errors.push(format!(
            "File `{}` not found.",
            &cfd_item.location.filepath.display()
        ));
    }
    rust_file
}
