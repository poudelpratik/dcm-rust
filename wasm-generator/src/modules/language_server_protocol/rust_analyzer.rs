use crate::modules::error::ApplicationError;
use crate::modules::language_server_protocol::traits::lsp_client::{
    LspClient, LspFilePath, RustItemLocation,
};
use crate::modules::source_code_analyzer::types::RustItemPosition;
use crate::modules::util::id_generator::NumericIdGenerator;
use lsp_types::request::{GotoImplementationParams, GotoImplementationResponse};
use lsp_types::{
    ClientCapabilities, DidOpenTextDocumentParams, DocumentHighlight, DocumentHighlightParams,
    GotoDefinitionParams, GotoDefinitionResponse, InitializeParams, Location, Position,
    TextDocumentClientCapabilities, TextDocumentIdentifier, TextDocumentItem,
    TextDocumentPositionParams, TraceValue, Url, WorkspaceFolder,
};
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, ChildStdout, Command};

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<u32>,
    method: String,
    params: Option<Value>,
}

impl JsonRpcRequest {
    pub fn new(method: &str, params: Option<Value>, id: Option<u32>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<u32>,
    result: Option<Value>,
    error: Option<Value>,
}

pub struct RustAnalyzerClient {
    writer: ChildStdin,
    reader: BufReader<ChildStdout>,
    id_generator: NumericIdGenerator,
    open_documents: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcNotification<T> {
    jsonrpc: String,
    method: String,
    params: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerStatusParams {
    health: String,
    quiescent: bool,
    message: Option<String>,
}

impl TryFrom<GotoDefinitionResponse> for RustItemLocation {
    type Error = ApplicationError;

    fn try_from(value: GotoDefinitionResponse) -> Result<Self, ApplicationError> {
        let location: Option<Location> = match value {
            GotoDefinitionResponse::Scalar(location) => Some(location),
            GotoDefinitionResponse::Array(locations) => locations.first().cloned(),
            GotoDefinitionResponse::Link(_) => None,
        };
        let location = location.ok_or("Empty response from LSP.")?;
        Ok(location.into())
    }
}

impl RustAnalyzerClient {
    pub async fn new() -> Result<Self, ApplicationError> {
        let mut child = Command::new("rust-analyzer")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let writer = child.stdin.take().ok_or("Failed to open stdin")?;
        let reader = BufReader::new(child.stdout.take().ok_or("Failed to open stdout")?);
        let id_generator = NumericIdGenerator::default();
        Ok(Self {
            writer,
            reader,
            id_generator,
            open_documents: Vec::new(),
        })
    }

    async fn send_request(
        &mut self,
        jsonrpc_request: JsonRpcRequest,
    ) -> Result<Option<JsonRpcResponse>, ApplicationError> {
        let req = serde_json::to_string(&jsonrpc_request)?;
        let message = format!("Content-Length: {}\r\n\r\n{}", req.len(), req);
        self.writer.write_all(message.as_bytes()).await?;
        self.writer.flush().await?;

        match jsonrpc_request.id {
            Some(id) => loop {
                let raw_response = self.read_response().await?;
                let response: JsonRpcResponse = serde_json::from_str(&raw_response)?;
                if response.id == Some(id) {
                    return Ok(Some(response));
                }
            },
            None => Ok(None),
        }
    }

    async fn read_response(&mut self) -> Result<String, ApplicationError> {
        let mut headers = String::new();
        loop {
            let mut line = String::new();
            self.reader.read_line(&mut line).await?;
            if line == "\r\n" {
                break;
            }
            headers.push_str(&line);
        }

        let content_length_key = "Content-Length: ";
        let length: usize = headers
            .lines()
            .find(|line| line.starts_with(content_length_key))
            .and_then(|line| line.strip_prefix(content_length_key))
            .and_then(|number_str| number_str.parse().ok())
            .ok_or("Missing or invalid Content-Length header")?;

        let mut body = vec![0; length];
        self.reader.read_exact(&mut body).await?;
        let body_str = String::from_utf8(body)?;
        Ok(body_str)
    }

    async fn initialize(&mut self, url: &str) -> Result<JsonRpcResponse, ApplicationError> {
        let message_id = self.id_generator.next_id();

        let workspace_folders = WorkspaceFolder {
            uri: Url::parse(url).unwrap(),
            name: "workspace".to_string(),
        };
        let params = InitializeParams {
            workspace_folders: Some(vec![workspace_folders]),
            root_uri: Some(Url::parse(url).unwrap()),
            trace: Some(TraceValue::Verbose),
            capabilities: ClientCapabilities {
                experimental: Some(json!({ "serverStatusNotification": true})),
                text_document: Some(TextDocumentClientCapabilities {
                    publish_diagnostics: Some(lsp_types::PublishDiagnosticsClientCapabilities {
                        related_information: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        };
        let jsonrpc_request =
            JsonRpcRequest::new("initialize", Some(json!(params)), Some(message_id));
        // Wait for the response using send_request_async
        let response = self.send_request(jsonrpc_request).await?;
        Ok(response.unwrap())
    }

    /// The initialized notification is sent from the client to the server after the client receives the result of the initialize request before the client sends any other request or notification to the server.
    async fn initialized(&mut self) -> Result<(), ApplicationError> {
        let jsonrpc_request = JsonRpcRequest::new("initialized", Some(json!({})), None);
        self.send_request(jsonrpc_request).await?;
        Ok(())
    }

    /// This function waits for the server to be ready, and only then returns ok
    async fn wait_for_server_ready(&mut self) -> Result<(), ApplicationError> {
        loop {
            let response = self.read_response().await?;
            let server_status_notification: JsonRpcNotification<ServerStatusParams> =
                serde_json::from_str(&response)?;
            if server_status_notification.method == "experimental/serverStatus"
                && server_status_notification.params.health == "ok"
                && server_status_notification.params.quiescent
            {
                return Ok(());
            }
        }
    }

    async fn did_open(&mut self, text_document_url: &str) -> Result<(), ApplicationError> {
        let url = Url::parse(text_document_url).unwrap();
        let path = url.to_file_path().unwrap();
        let code = std::fs::read_to_string(path).unwrap();
        let did_open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: url,
                language_id: "rust".to_string(),
                version: 1,
                text: code,
            },
        };
        let jsonrpc_request =
            JsonRpcRequest::new("textDocument/didOpen", Some(json!(did_open_params)), None);
        self.send_request(jsonrpc_request).await?;
        self.open_documents.push(text_document_url.to_string());
        Ok(())
    }

    async fn definition(
        &mut self,
        text_document_url: &str,
        line: u32,
        character: u32,
    ) -> Result<GotoDefinitionResponse, ApplicationError> {
        let message_id = self.id_generator.next_id();

        let go_to_definition_params = GotoDefinitionParams {
            text_document_position_params: TextDocumentPositionParams::new(
                TextDocumentIdentifier::new(Url::parse(text_document_url).unwrap()),
                Position::new(line, character),
            ),
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };
        let jsonrpc_request = JsonRpcRequest::new(
            "textDocument/definition",
            Some(json!(go_to_definition_params)),
            Some(message_id),
        );

        let response = self.send_request(jsonrpc_request).await?.unwrap();
        if let Some(goto_definition_response) = response.result {
            let goto_definition_response: GotoDefinitionResponse =
                serde_json::from_value(goto_definition_response)?;
            Ok(goto_definition_response)
        } else {
            Err(ApplicationError::TypeConversionError {
                message: "No result in goto definition response".to_string(),
            })?
        }
    }

    async fn implementation(
        &mut self,
        text_document_url: &str,
        line: u32,
        character: u32,
    ) -> Result<GotoImplementationResponse, ApplicationError> {
        let message_id = self.id_generator.next_id();

        let go_to_definition_params = GotoImplementationParams {
            text_document_position_params: TextDocumentPositionParams::new(
                TextDocumentIdentifier::new(Url::parse(text_document_url).unwrap()),
                Position::new(line, character),
            ),
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };
        let jsonrpc_request = JsonRpcRequest::new(
            "textDocument/implementation",
            Some(json!(go_to_definition_params)),
            Some(message_id),
        );

        let response = self.send_request(jsonrpc_request).await?.unwrap();
        if let Some(goto_implementation_response) = response.result {
            let goto_implementation_response: GotoImplementationResponse =
                serde_json::from_value(goto_implementation_response)?;
            Ok(goto_implementation_response)
        } else {
            Err(ApplicationError::TypeConversionError {
                message: "No result in goto implementation response".to_string(),
            })?
        }
    }

    // async fn references(
    //     &mut self,
    //     text_document_url: &str,
    //     line: u32,
    //     character: u32,
    // ) -> Result<Vec<Location>, ApplicationError> {
    //     let message_id = self.id_generator.next_id();
    //
    //     let find_references_params = ReferenceParams {
    //         text_document_position: TextDocumentPositionParams::new(
    //             TextDocumentIdentifier::new(Url::parse(text_document_url).unwrap()),
    //             Position::new(line, character),
    //         ),
    //         work_done_progress_params: Default::default(),
    //         partial_result_params: Default::default(),
    //         context: ReferenceContext {
    //             include_declaration: false,
    //         },
    //     };
    //     let jsonrpc_request = JsonRpcRequest::new(
    //         "textDocument/references",
    //         Some(json!(find_references_params)),
    //         Some(message_id),
    //     );
    //
    //     let response = self.send_request(jsonrpc_request).await?.unwrap();
    //     if let Some(find_references_response) = response.result {
    //         let find_references_response: Vec<Location> =
    //             serde_json::from_value(find_references_response)?;
    //         Ok(find_references_response)
    //     } else {
    //         Err(ApplicationError::TypeConversionError {
    //             message: "No result in find references response".to_string(),
    //         })?
    //     }
    // }

    async fn document_highlight(
        &mut self,
        text_document_url: &str,
        line: u32,
        character: u32,
    ) -> Result<Vec<DocumentHighlight>, ApplicationError> {
        let message_id = self.id_generator.next_id();

        let document_highlight_params = DocumentHighlightParams {
            text_document_position_params: TextDocumentPositionParams::new(
                TextDocumentIdentifier::new(Url::parse(text_document_url).unwrap()),
                Position::new(line, character),
            ),
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };
        let jsonrpc_request = JsonRpcRequest::new(
            "textDocument/documentHighlight",
            Some(json!(document_highlight_params)),
            Some(message_id),
        );

        let response = self.send_request(jsonrpc_request).await?.unwrap();
        if let Some(document_highlight_response) = response.result {
            let find_references_response: Vec<DocumentHighlight> =
                serde_json::from_value(document_highlight_response)?;
            Ok(find_references_response)
        } else {
            Err(ApplicationError::TypeConversionError {
                message: "No result in find references response".to_string(),
            })?
        }
    }

    // async fn shutdown(&mut self) -> Result<(), ApplicationError> {
    //     let jsonrpc_request = JsonRpcRequest::new("shutdown", Some(json!({})), None);
    //     self.send_request(jsonrpc_request).await?;
    //     Ok(())
    // }
    //
    // async fn exit(&mut self) -> Result<(), ApplicationError> {
    //     let jsonrpc_request = JsonRpcRequest::new("exit", Some(json!({})), None);
    //     self.send_request(jsonrpc_request).await?;
    //     Ok(())
    // }
}

#[async_trait::async_trait]
impl LspClient for RustAnalyzerClient {
    async fn initialize(&mut self, project_root_url: LspFilePath) -> Result<(), ApplicationError> {
        // Send initialize request
        self.initialize(project_root_url.0.as_str()).await?;
        // After sending the initialize request, initialized notification must be sent
        self.initialized().await?;
        // After sending the initialized notification, we need to wait for the server to be ready, else and request will yield empty responses until then
        self.wait_for_server_ready().await?;
        Ok(())
    }

    async fn get_definition_location(
        &mut self,
        file_path: LspFilePath,
        line: u32,
        column: u32,
    ) -> Result<RustItemLocation, ApplicationError> {
        let file_path = file_path.0.as_str();
        if !self.open_documents.contains(&file_path.to_string()) {
            self.did_open(file_path).await?;
        }
        let goto_definition_response = self.definition(file_path, line, column).await?;
        let rust_item_location: Result<RustItemLocation, ApplicationError> =
            goto_definition_response.try_into().map_err(|_| {
                ApplicationError::TypeConversionError {
                    message: "Failed to convert goto definition response to item location"
                        .to_string(),
                }
            });
        rust_item_location
    }

    async fn get_implementation_location(
        &mut self,
        file_path: LspFilePath,
        line: u32,
        column: u32,
    ) -> Result<RustItemLocation, ApplicationError> {
        let file_path = file_path.0.as_str();
        if !self.open_documents.contains(&file_path.to_string()) {
            self.did_open(file_path).await?;
        }
        let goto_implementation_response = self.implementation(file_path, line, column).await?;
        let rust_item_location: Result<RustItemLocation, ApplicationError> =
            goto_implementation_response.try_into().map_err(|_| {
                ApplicationError::TypeConversionError {
                    message: "Failed to convert goto implementation response to item location"
                        .to_string(),
                }
            });
        rust_item_location
    }

    async fn get_document_highlight_positions(
        &mut self,
        file_path: LspFilePath,
        line: u32,
        column: u32,
    ) -> Result<Vec<RustItemPosition>, ApplicationError> {
        let file_path = file_path.0.as_str();
        if !self.open_documents.contains(&file_path.to_string()) {
            self.did_open(file_path).await?;
        }
        let document_highlight_response = self.document_highlight(file_path, line, column).await?;
        let rust_item_location: Vec<RustItemPosition> = document_highlight_response
            .into_iter()
            .map(|dh| dh.range.into())
            .collect();
        Ok(rust_item_location)
    }
}

#[cfg(test)]
mod tests {
    use crate::modules::language_server_protocol::rust_analyzer::RustAnalyzerClient;
    use crate::modules::language_server_protocol::traits::lsp_client::{LspClient, LspFilePath};
    use tokio_test::{assert_err, assert_ok};

    #[tokio::test]
    async fn test_goto_definition_success() {
        let mut client: Box<dyn LspClient> = Box::new(RustAnalyzerClient::new().await.unwrap());

        // Initialize and wait for the server to be ready.
        client.initialize(LspFilePath("file:///home/cybernetics/Documents/Projects/University/thesis/rust-webassembly/runtime-code-mobility-demo".to_string())).await.unwrap();

        let definition_location = client
            .get_definition_location(LspFilePath("file:///home/cybernetics/Documents/Projects/University/thesis/rust-webassembly/runtime-code-mobility-demo/src/main.rs".to_string()), 23, 24)
            .await
            .unwrap();
        println!(
            "Go to Definition Response: {}",
            serde_json::to_string(&definition_location).unwrap()
        );

        let definition_location = client
            .get_implementation_location(LspFilePath("file:///home/cybernetics/Documents/Projects/University/thesis/rust-webassembly/runtime-code-mobility-demo/src/shared/webshop.rs".to_string()), 157, 11)
            .await
            .unwrap();
        println!(
            "Go to Implementation Response: {}",
            serde_json::to_string(&definition_location).unwrap()
        );

        // let definition_location = client
        //     .get_definition_location(LspFilePath("file:///home/cybernetics/Documents/Projects/University/thesis/rust-webassembly/runtime-code-mobility-demo/src/shared/webshop.rs".to_string()), 18, 24)
        //     .await;
        // assert_ok!(&definition_location);
        // println!(
        //     "Go to Definition Response: {}",
        //     serde_json::to_string(&definition_location.unwrap()).unwrap()
        // );
    }

    #[tokio::test]
    async fn test_document_highlight_success() {
        let mut client: Box<dyn LspClient> = Box::new(RustAnalyzerClient::new().await.unwrap());

        // Initialize and wait for the server to be ready.
        client.initialize(LspFilePath("file:///home/cybernetics/Documents/Projects/University/thesis/rust-webassembly/runtime-code-mobility-demo".to_string())).await.unwrap();

        let location = client
            .get_definition_location(LspFilePath("file:///home/cybernetics/Documents/Projects/University/thesis/rust-webassembly/runtime-code-mobility-demo/src/shared/playground.rs".to_string()), 0, 34)
            .await
            .unwrap();
        println!(
            "Go to Definition Response: {}",
            serde_json::to_string_pretty(&location).unwrap()
        );
        let location = client
            .get_document_highlight_positions(LspFilePath("file:///home/cybernetics/Documents/Projects/University/thesis/rust-webassembly/runtime-code-mobility-demo/src/shared/playground.rs".to_string()), 0, 34)
            .await
            .unwrap();
        println!(
            "Find Document Highlight Response: {}",
            serde_json::to_string_pretty(&location).unwrap()
        );

        let location = client
            .get_definition_location(LspFilePath("file:///home/cybernetics/Documents/Projects/University/thesis/rust-webassembly/runtime-code-mobility-demo/src/shared/playground.rs".to_string()), 1, 16)
            .await;
        println!(
            "Go to Definition Response: {}",
            serde_json::to_string_pretty(&location.unwrap()).unwrap()
        );
        let location = client
            .get_document_highlight_positions(LspFilePath("file:///home/cybernetics/Documents/Projects/University/thesis/rust-webassembly/runtime-code-mobility-demo/src/shared/playground.rs".to_string()), 1, 16)
            .await
            .unwrap();
        println!(
            "Find Document Highlight Response: {}",
            serde_json::to_string_pretty(&location).unwrap()
        );

        let location = client
            .get_definition_location(LspFilePath("file:///home/cybernetics/Documents/Projects/University/thesis/rust-webassembly/runtime-code-mobility-demo/src/shared/playground.rs".to_string()), 1, 46)
            .await;
        println!(
            "Go to Definition Response: {}",
            serde_json::to_string_pretty(&location.unwrap()).unwrap()
        );
        let location = client
            .get_document_highlight_positions(LspFilePath("file:///home/cybernetics/Documents/Projects/University/thesis/rust-webassembly/runtime-code-mobility-demo/src/shared/playground.rs".to_string()), 1, 46)
            .await
            .unwrap();
        println!(
            "Find Document Highlight Response: {}",
            serde_json::to_string_pretty(&location).unwrap()
        );
    }

    #[tokio::test]
    async fn test_failure() {
        let mut client: Box<dyn LspClient> = Box::new(RustAnalyzerClient::new().await.unwrap());

        // Initialize and wait for the server to be ready.
        client.initialize(LspFilePath("file:///home/cybernetics/Documents/Projects/University/thesis/rust-webassembly/runtime-code-mobility-demo".to_string())).await.unwrap();
        let definition_location = client
            .get_definition_location(LspFilePath("file:///home/cybernetics/Documents/Projects/University/thesis/rust-webassembly/runtime-code-mobility-demo/src/shared/playground5.rs".to_string()), 37, 16)
            .await;
        assert_err!(definition_location);
    }

    #[tokio::test]
    #[ignore]
    async fn test_it_old_way() {
        let mut client = RustAnalyzerClient::new().await.unwrap();
        let initialize_response = client.initialize("file:///home/cybernetics/Documents/Projects/University/thesis/rust-webassembly/runtime-code-mobility-demo").await.unwrap();
        println!(
            "Initialize Response: {}",
            serde_json::to_string(&initialize_response).unwrap()
        );
        client.initialized().await.unwrap();
        client.wait_for_server_ready().await.unwrap();
        client
            .did_open("file:///home/cybernetics/Documents/Projects/University/thesis/rust-webassembly/runtime-code-mobility-demo/src/main.rs")
            .await
            .unwrap();
        let definition_response = client
            .definition("file:///home/cybernetics/Documents/Projects/University/thesis/rust-webassembly/runtime-code-mobility-demo/src/main.rs", 23, 24)
            .await
            .unwrap();
        println!(
            "Go to Definition Response: {}",
            serde_json::to_string(&definition_response).unwrap()
        );
        client
            .did_open("file:///home/cybernetics/Documents/Projects/University/thesis/rust-webassembly/runtime-code-mobility-demo/src/shared/playground5.rs")
            .await
            .unwrap();
        let definition_response = client
            .definition("file:///home/cybernetics/Documents/Projects/University/thesis/rust-webassembly/runtime-code-mobility-demo/src/shared/playground5.rs", 19, 33)
            .await
            .unwrap();
        println!(
            "Go to Definition Response: {}",
            serde_json::to_string(&definition_response).unwrap()
        );
        let definition_response = client
            .definition("file:///home/cybernetics/Documents/Projects/University/thesis/rust-webassembly/runtime-code-mobility-demo/src/shared/playground5.rs", 18, 24)
            .await
            .unwrap();
        println!(
            "Go to Definition Response: {}",
            serde_json::to_string(&definition_response).unwrap()
        );
    }

    // #[tokio::test]
    // async fn test_publish_diagnostic() {
    //     let mut client = RustAnalyzerClient::new().await.unwrap();
    //     let initialize_response = client.initialize("file:///home/cybernetics/Documents/Projects/University/thesis/rust-webassembly/runtime-code-mobility-demo").await.unwrap();
    //     println!(
    //         "Initialize Response: {}",
    //         serde_json::to_string(&initialize_response).unwrap()
    //     );
    //     client.initialized().await.unwrap();
    //     client.wait_for_server_ready().await.unwrap();
    //     client
    //         .did_open("file:///home/cybernetics/Documents/Projects/University/thesis/rust-webassembly/runtime-code-mobility-demo/src/main.rs")
    //         .await
    //         .unwrap();
    //     client
    //         .did_open("file:///home/cybernetics/Documents/Projects/University/thesis/rust-webassembly/runtime-code-mobility-demo/src/shared/playground5.rs")
    //         .await
    //         .unwrap();
    //     let definition_response = client
    //         .document_publish_diagnostics("file:///home/cybernetics/Documents/Projects/University/thesis/rust-webassembly/runtime-code-mobility-demo/src/shared/playground5.rs")
    //         .await
    //         .unwrap();
    //     println!(
    //         "Go to Definition Response: {}",
    //         serde_json::to_string(&definition_response).unwrap()
    //     );
    // }
}
