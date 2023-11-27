use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("FileNotWritable")]
    IOError(#[from] std::io::Error),

    #[error("String error: {0}")]
    StrError(String),

    #[error("error executing command - {command}: {error_message}")]
    CommandExecutionError {
        command: String,
        error_message: String,
    },

    #[error("yaml parse error")]
    YamlParseError(#[from] serde_yaml::Error),
    #[error("syn parse error")]
    SynParseError(#[from] syn::Error),

    #[error("jsonrpc error")]
    TransportError(#[from] jsonrpc::Error),

    #[error("type conversion error: {message}")]
    TypeConversionError { message: String },

    #[error("serde json error")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("utf8 yaml error")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("UnknownError")]
    UnknownError,
}

impl From<&str> for ApplicationError {
    fn from(s: &str) -> Self {
        ApplicationError::StrError(s.to_string())
    }
}
