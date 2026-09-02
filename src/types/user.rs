use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct SignupInputs {
    pub email: String,
    pub password: String
}

#[derive(Serialize, Deserialize)]
pub struct SignupResponse {
    pub message: String
}

#[derive(Serialize, Deserialize)]
pub struct SignInResponse {
    pub message: String,
    pub token: Option<String>
}

pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password: String
}

#[derive(Serialize, Deserialize)]
pub struct Cliams {
    pub sub: Uuid,
    pub exp: u64,
}

#[derive(Serialize, Deserialize)]
pub struct BalanceResponse {
    pub usd_balance: u32,
    pub stock_balance: HashMap<String, u32>
}

#[derive(Serialize, Deserialize)]
pub struct OnRampInput {
    pub amount: u32
}

#[derive(Serialize, Deserialize)]
pub struct AuthErrorRepsonse {
    pub message: String
}