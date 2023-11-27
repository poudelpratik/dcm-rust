use crate::modules::dependency_resolver::DependencyUsageDetail;
use crate::modules::source_code_analyzer::types::RustItemCommonProperties;
use crate::modules::util;
use syn::spanned::Spanned;
use syn::visit::{
    visit_expr_call, visit_expr_method_call, visit_expr_path, visit_expr_struct, visit_fn_arg,
    visit_item_fn, visit_item_impl, visit_item_struct, visit_return_type, visit_type, Visit,
};
use syn::{
    ExprCall, ExprMethodCall, ExprPath, ExprStruct, GenericArgument, ItemFn, ItemImpl, ItemStruct,
    Path, PathArguments, Type,
};

pub struct RustItemAstVisitor<'a> {
    pub rust_item: &'a RustItemCommonProperties,
    pub dependencies: &'a mut Vec<DependencyUsageDetail>,
}

impl<'a> RustItemAstVisitor<'a> {
    pub fn new(
        rust_item: &'a RustItemCommonProperties,
        dependencies: &'a mut Vec<DependencyUsageDetail>,
    ) -> Self {
        Self {
            rust_item,
            dependencies,
        }
    }

    fn is_within_line_range(&self, span: proc_macro2::Span) -> bool {
        let line = span.start().line;
        line >= self.rust_item.position.start_line && line <= self.rust_item.position.end_line
    }
}

impl<'a, 'ast> Visit<'ast> for RustItemAstVisitor<'a> {
    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        if let syn::Expr::Path(ref expr_path) = *node.func {
            let module_hierarchy = extract_module_hierarchy(&expr_path.path);
            if let Some(last_segment) = expr_path.path.segments.last() {
                let function_name = last_segment.ident.to_string();
                let func_span = last_segment.ident.span();
                let line = func_span.start().line as u32;
                let column = func_span.start().column as u32;

                // Condition for skipping recursive call
                if function_name != self.rust_item.name {
                    self.dependencies.push(DependencyUsageDetail {
                        file_path: self.rust_item.file_path.clone(),
                        module_hierarchy,
                        line,
                        column,
                    });
                }
            }
        }

        visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        let method_name = node.method.to_string();
        let method_span = node.method.span();
        let line = method_span.start().line as u32;
        let column = method_span.start().column as u32;
        let module_hierarchy = Vec::new();

        // Condition for skipping recursive call
        if method_name != self.rust_item.name {
            self.dependencies.push(DependencyUsageDetail {
                file_path: self.rust_item.file_path.clone(),
                module_hierarchy,
                line,
                column,
            });
        }

        visit_expr_method_call(self, node);
    }

    fn visit_expr_path(&mut self, node: &'ast ExprPath) {
        let module_hierarchy = extract_module_hierarchy(&node.path);
        if let Some(last_segment) = node.path.segments.last() {
            let item_span = last_segment.ident.span();
            let line = item_span.start().line as u32;
            let column = item_span.start().column as u32;

            self.dependencies.push(DependencyUsageDetail {
                file_path: self.rust_item.file_path.clone(),
                module_hierarchy,
                line,
                column,
            });
        }

        visit_expr_path(self, node);
    }

    fn visit_expr_struct(&mut self, node: &'ast ExprStruct) {
        let module_hierarchy = extract_module_hierarchy(&node.path);
        if let Some(last_segment) = node.path.segments.last() {
            let span = last_segment.ident.span();
            let line = span.start().line as u32;
            let column = span.start().column as u32;

            self.dependencies.push(DependencyUsageDetail {
                file_path: self.rust_item.file_path.clone(),
                module_hierarchy,
                line,
                column,
            });
        }

        visit_expr_struct(self, node);
    }

    fn visit_fn_arg(&mut self, node: &'ast syn::FnArg) {
        if let syn::FnArg::Typed(pat_type) = node {
            if let Type::Path(type_path) = &*pat_type.ty {
                if let Some(last_segment) = type_path.path.segments.last() {
                    let type_name = last_segment.ident.to_string();
                    let type_span = last_segment.ident.span();
                    let line = type_span.start().line as u32;
                    let column = type_span.start().column as u32;

                    // Add the type to dependencies, if it's not a primitive type
                    if !util::is_primitive(&type_name) {
                        self.dependencies.push(DependencyUsageDetail {
                            file_path: self.rust_item.file_path.clone(),
                            module_hierarchy: Vec::new(),
                            line,
                            column,
                        });
                    }

                    // If there are generics, visit them as well
                    if let PathArguments::AngleBracketed(angle_bracketed_param) =
                        &last_segment.arguments
                    {
                        for generic_arg in &angle_bracketed_param.args {
                            if let GenericArgument::Type(generic_type) = generic_arg {
                                self.visit_type(generic_type);
                            }
                        }
                    }
                }
            }
        }

        visit_fn_arg(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        if self.is_within_line_range(node.span()) {
            visit_item_fn(self, node);
        }
    }

    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        if self.is_within_line_range(node.span()) {
            if let Type::Path(type_path) = &*node.self_ty {
                let module_hierarchy = extract_module_hierarchy(&type_path.path);
                if let Some(last_segment) = type_path.path.segments.last() {
                    let line = last_segment.ident.span().start().line as u32;
                    let column = last_segment.ident.span().start().column as u32;

                    self.dependencies.push(DependencyUsageDetail {
                        file_path: self.rust_item.file_path.clone(),
                        module_hierarchy,
                        line,
                        column,
                    });
                }

                visit_item_impl(self, node);
            }
        }
    }

    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        if self.is_within_line_range(node.span()) {
            visit_item_struct(self, node);
        }
    }

    fn visit_return_type(&mut self, node: &'ast syn::ReturnType) {
        if let syn::ReturnType::Type(_, type_box) = node {
            if let Type::Path(type_path) = &**type_box {
                let module_hierarchy = extract_module_hierarchy(&type_path.path);
                if let Some(last_segment) = type_path.path.segments.last() {
                    let type_name = last_segment.ident.to_string();
                    let type_span = last_segment.ident.span();
                    let line = type_span.start().line as u32;
                    let column = type_span.start().column as u32;

                    // Add the type to dependencies if it's not a primitive type
                    if !util::is_primitive(&type_name) {
                        self.dependencies.push(DependencyUsageDetail {
                            file_path: self.rust_item.file_path.clone(),
                            module_hierarchy,
                            line,
                            column,
                        });
                    }

                    // If there are generics, visit them as well
                    if let PathArguments::AngleBracketed(angle_bracketed_param) =
                        &last_segment.arguments
                    {
                        for generic_arg in &angle_bracketed_param.args {
                            if let GenericArgument::Type(generic_type) = generic_arg {
                                self.visit_type(generic_type);
                            }
                        }
                    }
                }
            }
        }

        visit_return_type(self, node);
    }

    fn visit_type(&mut self, node: &'ast Type) {
        if let Type::Path(type_path) = node {
            let module_hierarchy = extract_module_hierarchy(&type_path.path);
            if let Some(last_segment) = type_path.path.segments.last() {
                let type_name = last_segment.ident.to_string();
                let type_span = last_segment.ident.span();
                let line = type_span.start().line as u32;
                let column = type_span.start().column as u32;

                // Add the type to dependencies, if it's not a primitive type
                if !util::is_primitive(&type_name) {
                    self.dependencies.push(DependencyUsageDetail {
                        file_path: self.rust_item.file_path.clone(),
                        module_hierarchy,
                        line,
                        column,
                    });
                }

                // If there are generics, visit them as well
                if let PathArguments::AngleBracketed(angle_bracketed_param) =
                    &last_segment.arguments
                {
                    for generic_arg in &angle_bracketed_param.args {
                        if let GenericArgument::Type(generic_type) = generic_arg {
                            self.visit_type(generic_type);
                        }
                    }
                }
            }
        }

        visit_type(self, node);
    }
}

fn extract_module_hierarchy(path: &Path) -> Vec<String> {
    let mut segments: Vec<String> = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect();

    if !segments.is_empty() {
        match segments[0].as_str() {
            // If the path starts with `crate`, `self`, or `super`, remove this segment
            "crate" | "self" | "super" => {
                segments.remove(0);
            }
            _ => {}
        }
    }

    // Omit the last segment which is the item name itself, not part of the module hierarchy
    segments.pop();

    segments
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::source_code_analyzer::types::rust_function::RustFunction;
    use syn::parse_str;

    #[tokio::test]
    async fn test_function_visitor() {
        let source_code = r#"
        fn example_function(x: i32, s: &str) {
            let y = x + 1;
            let z: i32 = y * 2;
            another_function(y, z);
            test_module::test_function();
            let test1 = Test::new();
            test1.set_a(2);
            
            let test2 = Test::from_values(1, "Hello".to_string());
            
            let test3 = Test {
                a: 1,
                b: "Hello".to_string(),
            };
        }
        "#;

        let syntax_tree: syn::File = parse_str(source_code).unwrap();
        let mut func = RustFunction::default();
        func.properties.position.start_line = 2;
        func.properties.position.end_line = 16;
        let mut dependencies: Vec<DependencyUsageDetail> = Vec::new();
        let mut visitor = RustItemAstVisitor::new(&func.properties, &mut dependencies);
        visitor.visit_file(&syntax_tree);
        println!("Function calls: {:?}", visitor.dependencies);
    }
}
