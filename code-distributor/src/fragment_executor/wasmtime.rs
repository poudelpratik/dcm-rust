use crate::fragment_executor::FragmentExecutor;
use crate::fragment_registry::FragmentRegistry;
use crate::util::error::ApplicationError;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use wasmtime::{Engine, Instance, Module, Store};

pub struct ModuleInfo {
    pub module: Module,
    pub engine: Engine,
}

impl ModuleInfo {
    pub fn new(module: Module, engine: Engine) -> Self {
        Self { module, engine }
    }
}

pub(crate) struct Wasmtime {
    pub(crate) fragments: HashMap<String, ModuleInfo>,
}

impl Wasmtime {
    pub(crate) fn new(fragment_registry: &FragmentRegistry, fragments_dir: String) -> Self {
        let mut fragments: HashMap<String, ModuleInfo> = HashMap::new();
        for fragment in fragment_registry.fragments.iter() {
            let fragment_path = PathBuf::from(&fragments_dir).join(format!("{}.wasm", fragment.id));
            let engine = Engine::default();
            let module = Module::from_file(&engine, fragment_path).unwrap();
            fragments.insert(fragment.id.clone(), ModuleInfo::new(module, engine));
        }
        Self { fragments }
    }
}

const WASM_PAGE_SIZE: usize = 64 * 1024;

#[async_trait]
impl FragmentExecutor for Wasmtime {
    async fn execute(
        &self,
        fragment_id: &str,
        function_name: &str,
        params: &[serde_json::Value],
    ) -> Result<String, ApplicationError> {
        let module_info = self.fragments.get(fragment_id).unwrap();
        let function_name = format!("execute__{}", function_name);
        execute(module_info, function_name, params)
    }
}

pub fn execute(
    module_info: &ModuleInfo,
    function_name: String,
    params: &[serde_json::Value],
) -> Result<String, ApplicationError> {
    let mut store = Store::new(&module_info.engine, 4);
    let instance = Instance::new(&mut store, &module_info.module, &[]).unwrap();
    let func = instance.get_typed_func::<(i32, i32), i32>(&mut store, &function_name)?;
    let memory = instance.get_memory(&mut store, "memory").unwrap();

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
    memory.grow(&mut store, required_pages as u64)?;

    // Allocate memory in WebAssembly and get the pointer
    let alloc_func = instance.get_typed_func::<i32, i32>(&mut store, "alloc")?;
    let ptr = alloc_func.call(&mut store, total_length as i32)?;

    // Get the memory view, write bytes to memory, and call the function
    let view = memory;
    view.write(&mut store, ptr as usize, &args)?;
    let pointer = func.call(&mut store, (ptr, params.len() as i32))?;

    let data = memory.data(&store);
    let output_len = {
        let bytes = read(data, pointer as usize, 4);
        u32::from_le_bytes(bytes.as_slice().try_into().unwrap())
    };
    let output_bytes = read(data, (pointer as usize) + 4, output_len as usize);
    // Deserialize the output bytes to a string
    let output_str: String = rmp_serde::from_slice(output_bytes.as_slice())?;
    Ok(output_str)
}

fn read(data: &[u8], offset: usize, length: usize) -> Vec<u8> {
    data[offset..(offset + length)].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    const FRAGMENT_PATH: &str = "benches/resources/wasm";
    #[test]
    fn test_execute_wasm_fibonacci() {
        let mut params: Vec<serde_json::Value> = Vec::new();
        params.push(serde_json::Value::from(10));
        let fragment_path = PathBuf::from(FRAGMENT_PATH).join("fibonacci.wasm");
        let engine = Engine::default();
        let module = Module::from_file(&engine, fragment_path).unwrap();
        let module_info = ModuleInfo::new(module, engine);
        let function_name = "execute__fibonacci".to_string();
        let result = execute(&module_info, function_name, &params);
        println!("Result: {}", result.unwrap());
    }

    #[test]
    fn test_execute_wasm_factorial() {
        let mut params: Vec<serde_json::Value> = Vec::new();
        params.push(serde_json::Value::from(12));
        let fragment_path = PathBuf::from(FRAGMENT_PATH).join("factorial.wasm");
        let engine = Engine::default();
        let module = Module::from_file(&engine, fragment_path).unwrap();
        let module_info = ModuleInfo::new(module, engine);
        let function_name = "execute__factorial".to_string();
        let result = execute(&module_info, function_name, &params);
        println!("Result: {}", result.unwrap());
    }
}

#[cfg(test)]
mod benchmarks {
    use super::*;

    fn execute_fibonacci(module_info: &ModuleInfo) -> Result<String, ApplicationError> {
        let mut params: Vec<serde_json::Value> = Vec::new();
        params.push(serde_json::Value::from(10));
        let function_name = "execute__fibonacci".to_string();
        execute(module_info, function_name, &params)
    }

    fn execute_factorial(module_info: &ModuleInfo) -> Result<String, ApplicationError> {
        let mut params: Vec<serde_json::Value> = Vec::new();
        params.push(serde_json::Value::from(12));
        let function_name = "execute__factorial".to_string();
        execute(module_info, function_name, &params)
    }

    #[test]
    fn test_repeated_execution() {
        const FRAGMENT_PATH: &str = "benches/resources/wasm";
        let fragment_path = PathBuf::from(FRAGMENT_PATH).join("fibonacci.wasm");
        let fib_engine = Engine::default();
        let fib_module = Module::from_file(&fib_engine, fragment_path).unwrap();
        let fib_module_info = ModuleInfo::new(fib_module, fib_engine);
        let fragment_path = PathBuf::from(FRAGMENT_PATH).join("factorial.wasm");
        let fact_engine = Engine::default();
        let fact_module = Module::from_file(&fact_engine, fragment_path).unwrap();
        let fact_module_info = ModuleInfo::new(fact_module, fact_engine);
        for _ in 0..10 {
            let fibonacci_result = execute_fibonacci(&fib_module_info);
            assert!(
                fibonacci_result.is_ok(),
                "Fibonacci execution failed: {:?}",
                fibonacci_result.err()
            );
            let factorial_result = execute_factorial(&fact_module_info);
            assert!(
                factorial_result.is_ok(),
                "Factorial execution failed: {:?}",
                factorial_result.err()
            );
        }
        println!("Repeated execution completed successfully.");
    }
}
