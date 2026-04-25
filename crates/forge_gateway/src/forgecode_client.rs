use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use forge_api::ForgeAPI;
use forge_app::CommandOutput;
use forge_domain::Environment;
use forge_infra::ForgeCommandExecutorService;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandRequest {
    pub command: String,
    pub working_dir: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandResponse {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub success: bool,
}

pub struct ForgeCodeClient {
    executor: Arc<ForgeCommandExecutorService>,
    api: Arc<dyn ForgeAPI>,
    current_dir: Mutex<PathBuf>,
}

impl ForgeCodeClient {
    pub fn new(api: Arc<dyn ForgeAPI>) -> Self {
        let env = Environment::new();
        let executor = Arc::new(ForgeCommandExecutorService::new(env.clone(), Arc::new(forge_infra::console::StdConsoleWriter::default())));

        Self {
            executor,
            api,
            current_dir: Mutex::new(PathBuf::from(".")),
        }
    }

    pub async fn execute_command(&self, request: CommandRequest) -> Result<CommandResponse> {
        let working_dir = match request.working_dir {
            Some(dir) => PathBuf::from(dir),
            None => {
                let current_dir = self.current_dir.lock().await;
                current_dir.clone()
            }
        };

        // Execute the command using the forgecode infrastructure
        let output = self.api.execute_shell_command(&request.command, working_dir.clone()).await?;

        Ok(CommandResponse {
            stdout: output.stdout,
            stderr: output.stderr,
            exit_code: output.exit_code,
            success: output.success(),
        })
    }

    pub async fn execute_command_raw(&self, command: &str) -> Result<std::process::ExitStatus> {
        let working_dir = {
            let current_dir = self.current_dir.lock().await;
            current_dir.clone()
        };

        self.api.execute_shell_command_raw(command).await
    }

    pub async fn list_files(&self, path: Option<String>) -> Result<Vec<String>> {
        let target_path = match path {
            Some(dir) => PathBuf::from(dir),
            None => {
                let current_dir = self.current_dir.lock().await;
                current_dir.clone()
            }
        };

        // Use the forgecode API to discover files
        let files = self.api.discover().await?;

        // Filter files that are in the target directory
        let filtered_files: Vec<String> = files
            .into_iter()
            .filter_map(|file| {
                let file_path = PathBuf::from(&file.path);
                if file_path.parent() == Some(&target_path) {
                    Some(file.name)
                } else {
                    None
                }
            })
            .collect();

        Ok(filtered_files)
    }

    pub async fn change_directory(&self, path: String) -> Result<()> {
        let new_path = PathBuf::from(&path);

        // Validate that the path exists and is a directory
        if !new_path.exists() {
            return Err(anyhow!("Directory does not exist: {}", path));
        }

        if !new_path.is_dir() {
            return Err(anyhow!("Path is not a directory: {}", path));
        }

        let mut current_dir = self.current_dir.lock().await;
        *current_dir = new_path;

        Ok(())
    }

    pub async fn get_current_directory(&self) -> Result<String> {
        let current_dir = self.current_dir.lock().await;
        Ok(current_dir.to_string_lossy().to_string())
    }

    pub async fn get_tools(&self) -> Result<forge_app::dto::ToolsOverview> {
        self.api.get_tools().await
    }

    pub async fn get_models(&self) -> Result<Vec<forge_api::Model>> {
        self.api.get_models().await
    }

    pub async fn chat(&self, request: forge_api::ChatRequest) -> Result<forge_stream::MpscStream<Result<forge_api::ChatResponse>>> {
        self.api.chat(request).await
    }

    pub async fn commit(&self, preview: bool, max_diff_size: Option<usize>, diff: Option<String>, additional_context: Option<String>)
        -> Result<forge_app::CommitResult> {
        self.api.commit(preview, max_diff_size, diff, additional_context).await
    }
}