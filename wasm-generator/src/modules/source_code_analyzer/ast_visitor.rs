use crate::modules::application::fragment_type::RustItemType;
use crate::modules::application::function_fragment::FunctionFragment;
use crate::modules::application::object_fragment::ObjectFragment;
use crate::modules::application::MobileFragments;
use crate::modules::source_code_analyzer::rust_file::RustFile;
use crate::modules::source_code_analyzer::traits::rust_item::RustItem;
use crate::modules::source_code_analyzer::types::rust_const::RustConst;
use crate::modules::source_code_analyzer::types::rust_function::RustFunction;
use crate::modules::source_code_analyzer::types::rust_impl::RustImpl;
use crate::modules::source_code_analyzer::types::rust_static::RustStatic;
use crate::modules::source_code_analyzer::types::rust_struct::RustStruct;
use crate::modules::source_code_analyzer::types::rust_type_definition::RustTypeDefinition;
use crate::modules::source_code_analyzer::types::rust_use::RustUse;
use crate::modules::source_code_analyzer::FilePath;
use syn::visit::Visit;
use syn::{ItemConst, ItemImpl, ItemMod, ItemUse};

pub struct AstVisitor<'a> {
    pub rust_file: RustFile,
    source_code: String,
    module_hierarchy: Vec<String>,
    mobile_fragments: &'a mut MobileFragments,
}

impl<'a> AstVisitor<'a> {
    pub fn new(
        file_path: FilePath,
        source_code: String,
        mobile_fragments: &'a mut MobileFragments,
    ) -> Self {
        Self {
            rust_file: file_path.into(),
            source_code,
            module_hierarchy: Vec::new(),
            mobile_fragments,
        }
    }

    pub fn fill_common_properties(&self, rust_item: &mut impl RustItem) {
        let common_properties = rust_item.get_common_properties_mut();
        common_properties.set_module_hierarchy(self.module_hierarchy.clone());
        common_properties.set_item_code_from_source(self.source_code.as_str());
        common_properties.set_file_path(FilePath::from(
            self.rust_file
                .absolute_filepath
                .to_str()
                .unwrap()
                .to_string()
                .clone(),
        ));
    }
}

impl<'ast, 'a> Visit<'ast> for AstVisitor<'a> {
    fn visit_item_const(&mut self, node: &'ast ItemConst) {
        let mut rust_item: RustConst = node.clone().into();
        self.fill_common_properties(&mut rust_item);
        rust_item.properties.item_type = RustItemType::Const;
        self.rust_file.consts.push(rust_item.clone());
        syn::visit::visit_item_const(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let mut rust_item: RustFunction = node.clone().into();
        self.fill_common_properties(&mut rust_item);
        rust_item.properties.item_type = RustItemType::Function;
        self.rust_file.functions.push(rust_item.clone());

        if let Some(function_fragment) = FunctionFragment::try_create_from_attributes(
            node.attrs.clone(),
            rust_item.clone(),
            self.module_hierarchy.clone(),
        ) {
            self.mobile_fragments.functions.push(function_fragment);
        }

        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        let mut rust_item: RustImpl = node.clone().into();
        self.fill_common_properties(&mut rust_item);
        rust_item.properties.item_type = RustItemType::Impl;
        rust_item.functions.iter_mut().for_each(|f| {
            self.fill_common_properties(f);
        });
        self.rust_file.impls.push(rust_item.clone());
        if let Some(impl_fragment) = ObjectFragment::try_create_from_attributes(
            node.attrs.clone(),
            rust_item.clone(),
            self.module_hierarchy.clone(),
        ) {
            self.mobile_fragments.impls.push(impl_fragment);
        }
    }

    fn visit_item_mod(&mut self, node: &'ast ItemMod) {
        self.module_hierarchy.push(node.ident.to_string());
        if let Some(item) = node.content.as_ref() {
            for item in item.1.iter() {
                self.visit_item(item);
            }
        }
        self.module_hierarchy.pop();
    }

    fn visit_item_static(&mut self, node: &'ast syn::ItemStatic) {
        let mut rust_item: RustStatic = node.clone().into();
        self.fill_common_properties(&mut rust_item);
        rust_item.properties.item_type = RustItemType::Static;
        self.rust_file.statics.push(rust_item.clone());
        syn::visit::visit_item_static(self, node);
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        let mut rust_item: RustStruct = node.clone().into();
        self.fill_common_properties(&mut rust_item);
        rust_item.properties.item_type = RustItemType::Struct;
        self.rust_file.structs.push(rust_item.clone());
        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_type(&mut self, node: &'ast syn::ItemType) {
        let mut rust_item: RustTypeDefinition = node.clone().into();
        self.fill_common_properties(&mut rust_item);
        rust_item.properties.item_type = RustItemType::Type;
        self.rust_file.type_definitions.push(rust_item.clone());
        syn::visit::visit_item_type(self, node);
    }

    fn visit_item_use(&mut self, node: &'ast ItemUse) {
        let mut rust_item: RustUse = node.clone().into();
        self.fill_common_properties(&mut rust_item);
        rust_item.properties.item_type = RustItemType::Use;
        self.rust_file.uses.push(rust_item.clone());
        syn::visit::visit_item_use(self, node);
    }
}
