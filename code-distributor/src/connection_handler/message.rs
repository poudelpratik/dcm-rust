use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Message<T> {
    pub message_id: String,
    pub message_type: Events,
    pub data: T,
}

impl<T> Message<T> {
    pub fn new(request_id: String, event_type: Events, data: T) -> Self {
        Self {
            message_id: request_id,
            message_type: event_type,
            data,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) enum Events {
    ExecuteFunction,
    UpdateFragments,
}

impl FromStr for Events {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ExecuteFunction" => Ok(Events::ExecuteFunction),
            "UpdateFragments" => Ok(Events::UpdateFragments),
            _ => Err(()),
        }
    }
}
