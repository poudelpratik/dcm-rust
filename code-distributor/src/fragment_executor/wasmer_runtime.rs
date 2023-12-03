use std::path::PathBuf;

use async_trait::async_trait;
use byteorder::{LittleEndian, ReadBytesExt};
use wasmer::{Imports, Instance, MemoryView, Module, Store, Value, WasmSlice};

use crate::fragment_executor::FragmentExecutor;
use crate::util::error::ApplicationError;

pub(crate) struct WasmerRuntime {
    pub(crate) fragments_dir: PathBuf,
}

impl WasmerRuntime {
    pub(crate) fn new(fragments_dir: String) -> Self {
        Self {
            fragments_dir: PathBuf::from(fragments_dir),
        }
    }
}

#[async_trait]
impl FragmentExecutor for WasmerRuntime {
    async fn execute(
        &self,
        fragment_id: &str,
        function_name: &str,
        params: &[serde_json::Value],
    ) -> Result<String, ApplicationError> {
        let fragment_path = self
            .fragments_dir
            .join(format!("{}.wasm", fragment_id).as_str());
        let function_name = format!("execute__{}", function_name);
        WasmerInstance::new(fragment_path, function_name).execute(params)
    }
}

const WASM_PAGE_SIZE: usize = 64 * 1024; // 64KiB

pub struct WasmerInstance {
    pub store: Store,
    pub instance: Instance,
    pub function_name: String,
}

impl WasmerInstance {
    pub fn new(fragment_path: PathBuf, function_name: String) -> Self {
        let mut store = Store::default();
        let module = Module::from_file(&store, fragment_path).unwrap();
        let instance = Instance::new(&mut store, &module, &Imports::new()).unwrap();
        Self {
            store,
            instance,
            function_name,
        }
    }

    pub fn execute(&mut self, params: &[serde_json::Value]) -> Result<String, ApplicationError> {
        let func = self.instance.exports.get_function(&self.function_name)?;
        let memory = self.instance.exports.get_memory("memory")?;

        let mut args: Vec<u8> = Vec::new();
        for param in params.iter() {
            let param_as_bytes = rmp_serde::to_vec_named(param)?; // Serialize the parameter to MessagePack format
            let bytes_len = (param_as_bytes.len() as u32).to_le_bytes();

            // Pad args to the next 4-byte boundary
            while args.len() % 4 != 0 {
                args.push(0);
            }

            args.extend_from_slice(&bytes_len); // Extend args with the length bytes
            args.extend(&param_as_bytes); // Extend args with the parameter bytes
        }

        // grow memory if needed
        let total_length = args.len();
        let required_pages = (total_length + WASM_PAGE_SIZE - 1) / WASM_PAGE_SIZE;
        memory.grow(&mut self.store, required_pages as u32)?;

        // Allocate memory in WebAssembly and get the pointer
        let alloc_func = self.instance.exports.get_function("alloc")?;
        let ptr_value = alloc_func.call(&mut self.store, &[Value::I32(total_length as i32)])?;
        let ptr = match ptr_value.first() {
            Some(Value::I32(ptr)) => ptr,
            _ => Err(ApplicationError::WasmError {
                message: "Unable to get pointer from WebAssembly".to_string(),
            })?,
        };

        // Get the memory view, write bytes to memory, and call the function
        let view = memory.view(&self.store);
        view.write(*ptr as u64, &args)?;
        let result = func.call(
            &mut self.store,
            &[Value::I32(*ptr), Value::I32(params.len() as i32)],
        )?;

        match result.first() {
            Some(Value::I32(pointer)) => {
                let view = memory.view(&self.store);
                let output_len = {
                    let bytes = read(&view, *pointer as u64, 4)?;
                    bytes.as_slice().read_u32::<LittleEndian>()?
                };
                let output_bytes = read(&view, (*pointer as u64) + 4, output_len as u64)?;
                // Deserialize the output bytes to a string
                let output_str: String = rmp_serde::from_slice(output_bytes.as_slice())?;
                Ok(output_str)
            }
            _ => Err(ApplicationError::WasmError {
                message: "Unable to get result from WebAssembly".to_string(),
            }),
        }
    }
}

fn read(view: &MemoryView<'_>, offset: u64, length: u64) -> Result<Vec<u8>, ApplicationError> {
    Ok(WasmSlice::new(view, offset, length)?.read_to_vec()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    const FRAGMENT_PATH: &str = "fragments";

    #[test]
    fn test_execute_object() {
        let mut params: Vec<serde_json::Value> = Vec::new();
        params.push(serde_json::Value::from(
            r#"5bb7703f-8fe3-43fb-883a-a6d81fa4d58e"#,
        ));
        params.push(
            serde_json::Value::from_str(
                r#"{
    "orders": [
        {
            "archived": true,
            "id": "5bb7703f-8fe3-43fb-883a-a6d81fa4d58e",
            "products": [
                {
                    "id": "e6857e0b-49a3-46ed-9c58-3c8069a95c48",
                    "name": "Wanduhr",
                    "price": 8,
                    "quantity": 2
                },
                {
                    "id": "592e1747-85bc-4c91-8de5-a42cf1a05303",
                    "name": "Stuhl",
                    "price": 21.99,
                    "quantity": 1
                }
            ],
            "starred": true,
            "total": 37.989999999999995
        }
    ]
}"#,
            )
            .unwrap(),
        );

        let fragment_path = PathBuf::from(FRAGMENT_PATH).join("OrderManager.wasm");
        let function_name = "execute__toggle_starred".to_string();
        let result = WasmerInstance::new(fragment_path, function_name).execute(&params);
        println!("Result: {:?}", result.unwrap());
    }

    #[test]
    fn test_execute_wasm_fibonacci() {
        let mut params: Vec<serde_json::Value> = Vec::new();
        params.push(serde_json::Value::from(10));
        let fragment_path = PathBuf::from(FRAGMENT_PATH).join("fibonacci.wasm");
        let function_name = "execute__fibonacci".to_string();
        let result = WasmerInstance::new(fragment_path, function_name).execute(&params);
        println!("Result: {}", result.unwrap());
    }
}
