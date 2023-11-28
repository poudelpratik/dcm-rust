use crate::modules::error::ApplicationError;
use crate::modules::source_code_analyzer::types::RustItemPosition;
use crate::modules::source_code_analyzer::FilePath;
use async_trait::async_trait;
use lsp_types::{Location, Position, Range};
use serde_derive::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LspFilePath(pub String);

impl LspFilePath {
    pub fn get_as_string(&self) -> String {
        self.0.clone()
    }
}

impl From<FilePath> for LspFilePath {
    fn from(file_path: FilePath) -> Self {
        Self(format!(
            "file://{}",
            file_path.get_absolute_path_as_string()
        ))
    }
}

impl From<Location> for RustItemLocation {
    fn from(lsp_item_location: Location) -> Self {
        Self {
            file_path: LspFilePath(lsp_item_location.uri.to_string()).into(),
            position: lsp_item_location.range.into(),
        }
    }
}

// Range is based on Lsp's Position, where both line and column are 0-based.
// RustItemPosition is based on Rust's Span (use by syn crate), where line is 1-based and column is 0-based.
// That is why addition and subtraction of 1 is done only on line for the conversion implementations.
// https://docs.rs/proc-macro2/1.0.69/proc_macro2/struct.LineColumn.html
impl From<RustItemPosition> for Range {
    fn from(position: RustItemPosition) -> Self {
        Self {
            start: Position {
                line: (position.start_line - 1) as u32,
                character: (position.start_column) as u32,
            },
            end: Position {
                line: (position.end_line - 1) as u32,
                character: (position.end_column) as u32,
            },
        }
    }
}

impl From<Range> for RustItemPosition {
    fn from(position: Range) -> Self {
        Self {
            start_line: (position.start.line + 1) as usize,
            start_column: (position.start.character) as usize,
            end_line: (position.end.line + 1) as usize,
            end_column: (position.end.character) as usize,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RustItemLocation {
    pub file_path: FilePath,
    pub position: RustItemPosition,
}

#[async_trait]
pub trait LspClient {
    async fn initialize(&mut self, project_root_url: LspFilePath) -> Result<(), ApplicationError>;

    async fn get_definition_location(
        &mut self,
        file_path: LspFilePath,
        line: u32,
        column: u32,
    ) -> Result<RustItemLocation, ApplicationError>;

    async fn get_implementation_location(
        &mut self,
        file_path: LspFilePath,
        line: u32,
        column: u32,
    ) -> Result<RustItemLocation, ApplicationError>;

    async fn get_document_highlight_positions(
        &mut self,
        file_path: LspFilePath,
        line: u32,
        column: u32,
    ) -> Result<Vec<RustItemPosition>, ApplicationError>;

    async fn shutdown(&mut self) -> Result<(), ApplicationError>;
}
