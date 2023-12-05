// use std::path::PathBuf;
//
// use async_trait::async_trait;
// use byteorder::{LittleEndian, ReadBytesExt};
// use figment::value::Map;
// use wasmer::{
//     Engine, Imports, Instance, MemoryView, Module, NativeEngineExt, Store, Value, WasmSlice,
// };
//
// use crate::fragment_executor::FragmentExecutor;
// use crate::fragment_registry::FragmentRegistry;
// use crate::util::error::ApplicationError;
//
// pub(crate) struct WasmerRuntime {
//     pub(crate) fragments: Map<String, Module>,
// }
//
// impl WasmerRuntime {
//     pub(crate) fn new(fragment_registry: &FragmentRegistry, fragments_dir: String) -> Self {
//         let mut fragments: Map<String, Module> = Map::new();
//         for fragment in fragment_registry.fragments.iter() {
//             let fragment_path = PathBuf::from(&fragments_dir).join(format!("{}.wasm", fragment.id));
//             let store = Store::default();
//             let module = Module::from_file(&store, fragment_path).unwrap();
//             fragments.insert(fragment.id.clone(), module);
//         }
//         Self { fragments }
//     }
// }
//
// #[async_trait]
// impl FragmentExecutor for WasmerRuntime {
//     async fn execute(
//         &self,
//         fragment_id: &str,
//         function_name: &str,
//         params: &[serde_json::Value],
//     ) -> Result<String, ApplicationError> {
//         let module = self.fragments.get(fragment_id).unwrap();
//         let function_name = format!("execute__{}", function_name);
//         execute(module.clone(), function_name, params)
//     }
// }
//
// const WASM_PAGE_SIZE: usize = 64 * 1024; // 64KiB
//
// pub fn execute(
//     module: Module,
//     function_name: String,
//     params: &[serde_json::Value],
// ) -> Result<String, ApplicationError> {
//     let mut store = Store::new(Engine::headless());
//     let instance = Instance::new(&mut store, &module, &Imports::new()).unwrap();
//     let func = instance.exports.get_function(&function_name)?;
//     let memory = instance.exports.get_memory("memory")?;
//
//     let mut args: Vec<u8> = Vec::new();
//     for param in params.iter() {
//         let param_as_bytes = rmp_serde::to_vec_named(param)?; // Serialize the parameter to MessagePack format
//         let bytes_len = (param_as_bytes.len() as u32).to_le_bytes();
//
//         // Pad args to the next 4-byte boundary
//         while args.len() % 4 != 0 {
//             args.push(0);
//         }
//
//         args.extend_from_slice(&bytes_len); // Extend args with the length bytes
//         args.extend(&param_as_bytes); // Extend args with the parameter bytes
//     }
//
//     // grow memory if needed
//     let total_length = args.len();
//     let required_pages = (total_length + WASM_PAGE_SIZE - 1) / WASM_PAGE_SIZE;
//     memory.grow(&mut store, required_pages as u32)?;
//
//     // Allocate memory in WebAssembly and get the pointer
//     let alloc_func = instance.exports.get_function("alloc")?;
//     let ptr_value = alloc_func.call(&mut store, &[Value::I32(total_length as i32)])?;
//     let ptr = match ptr_value.first() {
//         Some(Value::I32(ptr)) => ptr,
//         _ => Err(ApplicationError::WasmError {
//             message: "Unable to get pointer from WebAssembly".to_string(),
//         })?,
//     };
//
//     // Get the memory view, write bytes to memory, and call the function
//     let view = memory.view(&store);
//     view.write(*ptr as u64, &args)?;
//     let result = func.call(
//         &mut store,
//         &[Value::I32(*ptr), Value::I32(params.len() as i32)],
//     )?;
//
//     let dealloc_func = instance.exports.get_function("dealloc")?;
//     dealloc_func.call(
//         &mut store,
//         &[Value::I32(*ptr), Value::I32(total_length as i32)],
//     )?;
//
//     match result.first() {
//         Some(Value::I32(pointer)) => {
//             let view = memory.view(&store);
//             let output_len = {
//                 let bytes = read(&view, *pointer as u64, 4)?;
//                 bytes.as_slice().read_u32::<LittleEndian>()?
//             };
//             let output_bytes = read(&view, (*pointer as u64) + 4, output_len as u64)?;
//             // Deserialize the output bytes to a string
//             let output_str: String = rmp_serde::from_slice(output_bytes.as_slice())?;
//             Ok(output_str)
//         }
//         _ => Err(ApplicationError::WasmError {
//             message: "Unable to get result from WebAssembly".to_string(),
//         }),
//     }
// }
//
// fn read(view: &MemoryView<'_>, offset: u64, length: u64) -> Result<Vec<u8>, ApplicationError> {
//     Ok(WasmSlice::new(view, offset, length)?.read_to_vec()?)
// }
//
// #[cfg(test)]
// mod tests {
//     use std::str::FromStr;
//
//     use super::*;
//
//     const FRAGMENT_PATH: &str = "benches/resources/wasm";
//
//     #[test]
//     fn test_execute_object() {
//         let mut params: Vec<serde_json::Value> = Vec::new();
//         params.push(serde_json::Value::from(
//             r#"5bb7703f-8fe3-43fb-883a-a6d81fa4d58e"#,
//         ));
//         params.push(
//             serde_json::Value::from_str(
//                 r#"{
//     "orders": [
//         {
//             "archived": true,
//             "id": "5bb7703f-8fe3-43fb-883a-a6d81fa4d58e",
//             "products": [
//                 {
//                     "id": "e6857e0b-49a3-46ed-9c58-3c8069a95c48",
//                     "name": "Wanduhr",
//                     "price": 8,
//                     "quantity": 2
//                 },
//                 {
//                     "id": "592e1747-85bc-4c91-8de5-a42cf1a05303",
//                     "name": "Stuhl",
//                     "price": 21.99,
//                     "quantity": 1
//                 }
//             ],
//             "starred": true,
//             "total": 37.989999999999995
//         }
//     ]
// }"#,
//             )
//             .unwrap(),
//         );
//
//         let fragment_path = PathBuf::from(FRAGMENT_PATH).join("OrderManager.wasm");
//         let store = Store::default();
//         let module = Module::from_file(&store, fragment_path).unwrap();
//         let function_name = "execute__toggle_starred".to_string();
//         let result = execute(module, function_name, &params);
//         println!("Result: {:?}", result.unwrap());
//     }
//
//     #[test]
//     fn test_execute_wasm_fibonacci() {
//         let mut params: Vec<serde_json::Value> = Vec::new();
//         params.push(serde_json::Value::from(10));
//         let fragment_path = PathBuf::from(FRAGMENT_PATH).join("fibonacci.wasm");
//         let store = Store::default();
//         let module = Module::from_file(&store, fragment_path).unwrap();
//         let function_name = "execute__fibonacci".to_string();
//         let result = execute(module, function_name, &params);
//         println!("Result: {}", result.unwrap());
//     }
//
//     #[test]
//     fn test_execute_wasm_factorial() {
//         let mut params: Vec<serde_json::Value> = Vec::new();
//         params.push(serde_json::Value::from(12));
//         let fragment_path = PathBuf::from(FRAGMENT_PATH).join("factorial.wasm");
//         let store = Store::default();
//         let module = Module::from_file(&store, fragment_path).unwrap();
//         let function_name = "execute__factorial".to_string();
//         let result = execute(module, function_name, &params);
//         println!("Result: {}", result.unwrap());
//     }
// }
//
// mod benchmarks {
//     use super::*;
//     use wasmer::Engine;
//
//     fn execute_fibonacci(module: Module) -> Result<String, ApplicationError> {
//         let mut params: Vec<serde_json::Value> = Vec::new();
//         params.push(serde_json::Value::from(10));
//         let function_name = "execute__fibonacci".to_string();
//         execute(module, function_name, &params)
//     }
//
//     fn execute_factorial(module: Module) -> Result<String, ApplicationError> {
//         let mut params: Vec<serde_json::Value> = Vec::new();
//         params.push(serde_json::Value::from(12));
//         let function_name = "execute__factorial".to_string();
//         execute(module, function_name, &params)
//     }
//
//     #[test]
//     fn test_repeated_execution() {
//         const FRAGMENT_PATH: &str = "benches/resources/wasm";
//         let fragment_path = PathBuf::from(FRAGMENT_PATH).join("fibonacci.wasm");
//         let fib_module = Module::from_file(&Engine::default(), fragment_path).unwrap();
//         let fragment_path = PathBuf::from(FRAGMENT_PATH).join("factorial.wasm");
//         let fact_module = Module::from_file(&Engine::default(), fragment_path).unwrap();
//         for _ in 0..10 {
//             let fibonacci_result = execute_fibonacci(fib_module.clone());
//             assert!(
//                 fibonacci_result.is_ok(),
//                 "Fibonacci execution failed: {:?}",
//                 fibonacci_result.err()
//             );
//             let factorial_result = execute_factorial(fact_module.clone());
//             assert!(
//                 factorial_result.is_ok(),
//                 "Factorial execution failed: {:?}",
//                 factorial_result.err()
//             );
//         }
//         println!("Repeated execution completed successfully.");
//     }
// }
