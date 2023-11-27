use crate::util::error::ApplicationError;
use async_trait::async_trait;

pub mod wasmer_runtime;

#[async_trait]
pub(crate) trait FragmentExecutor: Send + Sync {
    async fn execute(
        fragment_id: &str,
        function_name: &str,
        params: &[serde_json::Value],
    ) -> Result<String, ApplicationError>;
}
