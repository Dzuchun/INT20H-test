use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub name: String,
    pub pass: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {/* probably the same as `LoginResponse`? */}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub name: String,
    pub pass: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {/* some auth-providing fields */}
