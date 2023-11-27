use crate::modules::application::function_fragment::FunctionFragment;
use crate::modules::application::object_fragment::ObjectFragment;
use crate::modules::application::traits::fragment::Fragment;
use crate::modules::source_code_analyzer::types::rust_function::{RustFunction, RustFunctionType};

// Helper function to generate parameter values
fn generate_deserialize_param_values(rust_function: &RustFunction) -> String {
    let mut param_values = Vec::new();
    for (idx, param) in rust_function.parameters.iter().enumerate() {
        let param_indexed_name = format!("param{}", idx);
        param_values.push(format!(
            "let {}: {} = rmp_serde::from_slice(&parameters[{}]).unwrap();",
            param_indexed_name, param.rust_type, idx
        ));
    }
    param_values.join("\n    ")
}

// Helper function to generate function call
fn generate_function_call(rust_function: &RustFunction) -> String {
    let args = (0..rust_function.parameters.len())
        .map(|i| format!("param{}", i))
        .collect::<Vec<_>>()
        .join(", ");

    match rust_function.function_type {
        RustFunctionType::FreeFunction => {
            format!("let result = {}({});", rust_function.properties.name, args)
        }
        RustFunctionType::Method => {
            format!(
                "let mut self_instance: {} = rmp_serde::from_slice(&parameters[parameters.len()-1]).unwrap(); \
                 let result: {} = {{ self_instance.{}({}) }};
                ",
                rust_function.struct_name.as_ref().unwrap(),
                rust_function.return_type.rust_type,
                rust_function.properties.name, args
            )
        }
        RustFunctionType::AssociatedFunction => format!(
            "let result = {}::{}({});",
            rust_function.struct_name.as_ref().unwrap(),
            rust_function.properties.name,
            args
        ),
    }
}

fn generate_add_state(function_type: RustFunctionType) -> String {
    if function_type != RustFunctionType::Method {
        return "".to_string();
    }
    r#"map.insert("state".to_string(), serde_json::json!(self_instance));"#.to_string()
}

pub fn generate_wrapper(function: &RustFunction) -> String {
    let function_name = format!("execute__{}", function.properties.name);
    let param_values = generate_deserialize_param_values(function);
    let function_call = generate_function_call(function);
    let wrapper_code = format!(
        r#"
#[no_mangle]
pub extern "C" fn {}(params_ptr: *const u8, parameter_count: usize) -> *const u8 {{
    let parameters = extract_parameters(params_ptr, parameter_count);
    {}
    {}
    let mut map = serde_json::Map::new();
    {}
    map.insert("result".to_string(), serde_json::json!(result));
    serialize_result(map)
}}
"#,
        function_name,
        param_values,
        function_call,
        generate_add_state(function.function_type.clone()),
    );
    wrapper_code
}

pub fn generate_wrapper_for_free_functions(function_fragments: &mut [FunctionFragment]) {
    for fragment in function_fragments.iter_mut() {
        let wrapper_code = generate_wrapper(&fragment.rust_function);
        fragment.rust_function.properties.code = format!(
            "{}\n\n{}",
            wrapper_code, fragment.rust_function.properties.code
        );
    }
}

pub fn generate_wrapper_for_impls(impl_fragments: &mut [ObjectFragment]) {
    for fragment in impl_fragments.iter_mut() {
        let mut final_wrapper_code = String::new();
        for function in fragment.rust_impl.functions.iter_mut() {
            // reference types can be used internally but cannot be exposed outside of WebAssembly module
            if function.return_type.is_reference
                || function.parameters.iter().any(|p| p.is_reference)
            {
                continue;
            }
            let wrapper_code = generate_wrapper(function);
            final_wrapper_code = format!("{}\n\n{}", final_wrapper_code, wrapper_code);
        }
        fragment.set_code(format!("{}\n\n{}", final_wrapper_code, fragment.get_code()));
    }
}
