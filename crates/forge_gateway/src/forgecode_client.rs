use serde::{Deserialize, Serialize};
use thiserror::Error;
use reqwest::Client;
use std::time::Duration;

#[derive(Debug, Error)]
pub enum ForgeCodeError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(String),
    #[error("Service unavailable")]
    ServiceUnavailable,
    #[error("Invalid response format: {0}")]
    InvalidResponse(String),
    #[error("Authentication failed")]
    AuthenticationFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteCodeRequest {
    pub code: String,
    pub language: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteCodeResponse {
    pub output: String,
    pub error: Option<String>,
    pub session_id: Option<String>,
    pub execution_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateTokenRequest {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateTokenResponse {
    pub valid: bool,
    pub user_id: Option<String>,
    pub permissions: Vec<String>,
}

#[derive(Clone)]
pub struct ForgeCodeClient {
    client: Client,
    base_url: String,
}

impl ForgeCodeClient {
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            base_url,
        }
    }

    pub async fn execute_code(&self, request: ExecuteCodeRequest) -> Result<ExecuteCodeResponse, ForgeCodeError> {
        let url = format!("{}/api/execute", self.base_url);

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| ForgeCodeError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ForgeCodeError::ServiceUnavailable);
        }

        let response_data: ExecuteCodeResponse = response
            .json()
            .await
            .map_err(|e| ForgeCodeError::InvalidResponse(e.to_string()))?;

        Ok(response_data)
    }

    pub async fn validate_token(&self, token: String) -> Result<ValidateTokenResponse, ForgeCodeError> {
        let url = format!("{}/api/validate-token", self.base_url);
        let request = ValidateTokenRequest { token };

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| ForgeCodeError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ForgeCodeError::AuthenticationFailed);
        }

        let response_data: ValidateTokenResponse = response
            .json()
            .await
            .map_err(|e| ForgeCodeError::InvalidResponse(e.to_string()))?;

        Ok(response_data)
    }

    pub async fn health_check(&self) -> Result<bool, ForgeCodeError> {
        let url = format!("{}/health", self.base_url);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ForgeCodeError::RequestFailed(e.to_string()))?;

        Ok(response.status().is_success())
    }
}