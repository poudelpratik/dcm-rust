use serde::Serialize;
use serde_derive::Deserialize;

#[derive(Serialize, Deserialize)]
pub(crate) struct Claims {
    pub uuid: String,
    user_agent: String,
    ip_address: String,
    pub exp: usize,
}

impl Claims {
    pub fn new(uuid: String, user_agent: String, ip_address: String, exp: usize) -> Self {
        Self {
            uuid,
            user_agent,
            ip_address,
            exp,
        }
    }
}
