use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SignupInputs {
    pub email: String,
    pub password: String
}

#[derive(Serialize, Deserialize)]
pub struct SignupResponse {
    pub message: String
}
pub struct User {
    pub email: String,
    pub password: String
}