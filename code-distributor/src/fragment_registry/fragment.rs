use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct Fragment {
    pub(crate) id: String,
    pub(crate) execution_location: ExecutionLocation,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) enum ExecutionLocation {
    Client,
    Server,
}
