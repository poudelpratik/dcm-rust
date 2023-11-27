use crate::modules::application::function_fragment::FunctionFragment;
use crate::modules::application::object_fragment::ObjectFragment;
use crate::modules::application::traits::fragment::Fragment;
use crate::modules::application::MobileFragments;
use crate::modules::source_code_analyzer::types::rust_function::{RustFunction, RustFunctionType};
use crate::modules::util;

// Function to generate JavaScript function or method based on RustFunction
fn generate_js_function(
    fragment_id: &String,
    function: &RustFunction,
    function_type: RustFunctionType,
) -> String {
    let mut js_function_code = String::new();

    let function_name = &function.properties.name;
    let params = &function.parameters;
    let param_names = params
        .clone()
        .into_iter()
        .map(|param| param.name)
        .collect::<Vec<_>>();
    let json_return_type = &function.return_type.js_type;

    // Create parameter list
    let param_names_str = param_names.join(", ");

    // Create jsDoc lines
    let jsdoc_lines: Vec<String> = params
        .iter()
        .enumerate()
        .map(|(_, param)| format!("   * @param {{{}}} {}", param.js_type, param.name))
        .collect();

    let return_jsdoc_line = format!(
        "   * @returns {{Promise<{}>}} - The return value",
        json_return_type
    );
    let jsdoc = format!(
        "  /**\n{}\n{} \n   */",
        jsdoc_lines.join("\n"),
        return_jsdoc_line
    );

    // Generate the function or method signature
    let mut function_sig = String::new();
    match function_type {
        RustFunctionType::FreeFunction => {
            function_sig.push_str(&format!(
                "export async function {}({}) {{\n",
                fragment_id, param_names_str
            ));
        }
        RustFunctionType::Method => {
            function_sig.push_str(&format!(
                "async {}({}) {{\n",
                function_name, param_names_str
            ));
        }
        RustFunctionType::AssociatedFunction => {
            function_sig.push_str(&format!(
                "static async {}({}) {{\n",
                function_name, param_names_str
            ));
        }
    }

    // Generate the logic for executing the Rust function
    let mut execute_call = String::new();
    match function_type {
        RustFunctionType::FreeFunction => {
            let execute_call_base = format!(
                "    let result = await cdm.execute(\"{}\", \"{}\", [{}]);\n",
                fragment_id, function_name, param_names_str
            );
            let return_logic = String::from("    return result.result;\n");

            execute_call.push_str(&execute_call_base);
            execute_call.push_str(&return_logic);
        }
        RustFunctionType::Method => {
            let execute_call_base = format!(
                "    let result = await cdm.execute(\"{}\", \"{}\", [{}, this]);\n Object.assign(this, result.state);\n",
                fragment_id, function_name, param_names_str
            );

            let return_logic = String::from("    return result.result;\n");

            execute_call.push_str(&execute_call_base);
            execute_call.push_str(&return_logic);
        }
        RustFunctionType::AssociatedFunction => {
            let struct_name = function.struct_name.as_ref().unwrap();
            let return_type = &function.return_type.rust_type;
            let is_constructor = return_type.contains("Self")
                || return_type.contains(struct_name)
                || return_type.contains(format!("Result<{},", struct_name).as_str())
                || return_type.contains(format!("Option<{}>", struct_name).as_str())
                || return_type.contains("Option<Self>")
                || return_type.contains("Result<Self,");

            let execute_call_base = format!(
                "      let res = await cdm.execute(\"{}\", \"{}\", [{}]);\n",
                fragment_id, function_name, param_names_str
            );

            let return_logic = if is_constructor {
                String::from(
                    "      const result = res.result;\n   if (result !== null && result !== undefined) {\n        const newInstance = new this();\n        Object.assign(newInstance, result);\n        return newInstance;\n    }\n    return null;\n"
                )
            } else {
                String::from("    return result;\n")
            };

            execute_call.push_str(&execute_call_base);
            execute_call.push_str(&return_logic);
        }
    }

    // Combine all parts to form the complete JavaScript function or method
    js_function_code.push_str(&jsdoc);
    js_function_code.push('\n');
    js_function_code.push_str(&function_sig);
    js_function_code.push_str("    ensureInitialized();\n");
    js_function_code.push_str(&execute_call);
    js_function_code.push_str("}\n\n");

    js_function_code
}

fn generate_js_glue_for_free_functions(function_fragments: &Vec<FunctionFragment>) -> String {
    let mut js_code = String::new();

    // Handle free functions
    for fragment in function_fragments {
        js_code.push_str(&generate_js_function(
            &fragment.id,
            &fragment.rust_function,
            fragment.rust_function.function_type.clone(),
        ));
    }

    js_code
}

fn generate_js_glue_for_impl_block(impl_fragments: &Vec<ObjectFragment>) -> String {
    let mut js_code = String::new();

    // Handle impl blocks
    for impl_fragment in impl_fragments {
        let class_name = &impl_fragment.get_name();
        js_code.push_str(&format!("export class {} {{\n", class_name));

        // Generate JavaScript properties based on struct_data
        for field in &impl_fragment.rust_struct.fields {
            js_code.push_str(&format!("  /** @type {{{}}} */\n", field.js_type));
            js_code.push_str(&format!(
                "  {} = {};\n",
                field.name,
                util::get_default_value_for_js_type(&field.js_type)
            ));
        }

        for function in &impl_fragment.rust_impl.functions {
            // reference types can be used internally but cannot be exposed outside of WebAssembly module
            if function.return_type.is_reference
                || function.parameters.iter().any(|p| p.is_reference)
            {
                continue;
            }
            js_code.push_str(&generate_js_function(
                &impl_fragment.id,
                function,
                function.function_type.clone(),
            ));
        }

        js_code.push_str("}\n\n");
    }

    js_code
}

pub fn run(mobile_fragments: &MobileFragments) -> String {
    let mut js_code = String::new();
    js_code.push_str("import CodeDistributionManager from './index.js';\n\nlet cdm = null;\n\n");
    js_code.push_str(
        r#"
export async function initialize(config) {
    if (cdm === null) {
        cdm = new CodeDistributionManager(config);
        await cdm.init();
    }
}
    
function ensureInitialized() {
    if (cdm === null) {
        throw new Error('Initialize function must be called before using the exported functions.');
    }
}
        "#,
    );

    // Generate JS glue code for free functions
    js_code.push_str(&generate_js_glue_for_free_functions(
        &mobile_fragments.functions,
    ));

    // Generate JS glue code for impl blocks
    js_code.push_str(&generate_js_glue_for_impl_block(&mobile_fragments.impls));

    js_code
}
