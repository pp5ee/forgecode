use thiserror::Error;
use reqwest::Client;
use std::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Error)]
pub enum ForgeCodeError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(String),
    #[error("Service unavailable")]
    ServiceUnavailable,
    #[error("Invalid response format: {0}")]
    InvalidResponse(String),
}

#[derive(Debug, Serialize)]
pub struct ExecuteCommandRequest {
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExecuteCommandResponse {
    pub output: String,
    pub success: bool,
    pub exit_code: Option<i32>,
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

    pub async fn execute_command(&self, command: &str, args: &[String]) -> Result<String, ForgeCodeError> {
        let url = format!("{}/api/execute-command", self.base_url);

        let request = ExecuteCommandRequest {
            command: command.to_string(),
            args: args.to_vec(),
            working_dir: None,
        };

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| ForgeCodeError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ForgeCodeError::ServiceUnavailable);
        }

        let response_data: ExecuteCommandResponse = response
            .json()
            .await
            .map_err(|e| ForgeCodeError::InvalidResponse(e.to_string()))?;

        Ok(response_data.output)
    }

    pub async fn health_check(&self) -> Result<bool, ForgeCodeError> {
        let url = format!("{}/health", self.base_url);

        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    // Additional methods for forgecode integration
    pub async fn get_available_commands(&self) -> Result<Vec<String>, ForgeCodeError> {
        let url = format!("{}/api/commands", self.base_url);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ForgeCodeError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ForgeCodeError::ServiceUnavailable);
        }

        let commands: Vec<String> = response
            .json()
            .await
            .map_err(|e| ForgeCodeError::InvalidResponse(e.to_string()))?;

        Ok(commands)
    }

    pub async fn chat(&self, message: &str) -> Result<String, ForgeCodeError> {
        let url = format!("{}/api/chat", self.base_url);

        let request = serde_json::json!({
            "message": message
        });

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| ForgeCodeError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ForgeCodeError::ServiceUnavailable);
        }

        let response_data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ForgeCodeError::InvalidResponse(e.to_string()))?;

        Ok(response_data["response"].as_str().unwrap_or("No response").to_string())
    }
}