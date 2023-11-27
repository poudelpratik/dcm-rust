use crate::modules::application::traits::fragment::Fragment;
use crate::modules::application::MobileFragments;

pub fn generate_helper(mobile_fragments: &mut MobileFragments) {
    for executable_fragment in mobile_fragments.functions.iter_mut() {
        executable_fragment.rust_function.properties.code = format!(
            "{}\n{}",
            HELPER_FUNCTIONS, executable_fragment.rust_function.properties.code
        );
    }
    for executable_fragment in mobile_fragments.impls.iter_mut() {
        executable_fragment.set_code(format!(
            "{}\n{}",
            HELPER_FUNCTIONS,
            executable_fragment.get_code()
        ));
    }
}

const HELPER_FUNCTIONS: &str = r#"
use serde_derive::{Deserialize, Serialize};

#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut u8 {
    let layout = std::alloc::Layout::from_size_align(size, 4).unwrap();
    let ptr = unsafe { std::alloc::alloc(layout) };
    ptr
}

#[no_mangle]
pub extern "C" fn dealloc(ptr: *mut u8, size: usize) {
    let layout = std::alloc::Layout::from_size_align(size, 4).unwrap();
    unsafe { std::alloc::dealloc(ptr, layout) };
}

fn extract_parameters(params_ptr: *const u8, parameter_count: usize) -> Vec<Vec<u8>> {
    let mut parameters: Vec<Vec<u8>> = Vec::new();
    let mut current_ptr = params_ptr;
    unsafe {
        {
            for _ in 0..parameter_count {
                let len_ptr = current_ptr as *const u32;
                let len = *len_ptr as u32;
                current_ptr = current_ptr.add(4);
                let bytes = std::slice::from_raw_parts(current_ptr, len as usize);
                parameters.push(bytes.to_vec());
                current_ptr = current_ptr.add(len as usize);
                while (current_ptr as usize) % 4 != 0 {
                    current_ptr = current_ptr.add(1);
                }
            }
        }
    }
    parameters
}

fn serialize_result(map: serde_json::Map<String, serde_json::Value>) -> *const u8 {
    let result_json = serde_json::Value::Object(map);
    let result_json_string = serde_json::to_string(&result_json).unwrap();
    let result_bytes = rmp_serde::to_vec_named(&result_json_string).unwrap();
    let len_bytes = (result_bytes.len() as u32).to_le_bytes();

    let mut combined = Vec::new();
    combined.extend_from_slice(&len_bytes);
    combined.extend(&result_bytes);

    let leaked = combined.leak();
    leaked.as_ptr()
}
"#;
