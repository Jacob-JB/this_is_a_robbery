use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct JoinGameRequest {
    pub username: String,
}
