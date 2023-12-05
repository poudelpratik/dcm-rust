use thiserror::Error;

// use wasmer::ExportError;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("FileNotWritable")]
    FileNotWritable(#[from] std::io::Error),

    #[error("DataEncodeError")]
    DataEncodeError {
        #[from]
        source: rmp_serde::encode::Error,
    },

    // #[error("WasmRuntimeError")]
    // WasmRuntimeError(#[from] wasmer::RuntimeError),
    // #[error("WasmExportError")]
    // WasmExportError(#[from] ExportError),
    // #[error("WasmerMemoryAccessError")]
    // WasmerMemoryAccessError(#[from] wasmer::MemoryAccessError),
    // #[error("WasmerMemoryError")]
    // WasmerMemoryError(#[from] wasmer::MemoryError),
    #[error("WasmRuntimeError")]
    WasmtimeError(#[from] wasmtime::Error),
    #[error("WasmtimeMemoryAccessError")]
    WasmerMemoryAccessError(#[from] wasmtime::MemoryAccessError),
    #[error("message: {message}")]
    WasmError { message: String },
    #[error("WebSocketError")]
    WebSocketError(#[from] warp::Error),
    #[error("DecondingError")]
    DecodingError(#[from] rmp_serde::decode::Error),
}
