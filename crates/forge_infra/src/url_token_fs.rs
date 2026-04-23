//! File system-based URL token repository
//!
//! This module provides a file-based implementation of UrlTokenRepository
//! that stores tokens in a JSON file on disk.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use forge_domain::{UrlToken, UrlTokenId};
use tokio::sync::RwLock;

use crate::UrlTokenRepository;

/// File system based token repository
///
/// Stores tokens in a JSON file on disk. All operations are
/// protected by an async read-write lock for thread safety.
pub struct FsUrlTokenRepository {
    storage_path: PathBuf,
    cache: Arc<RwLock<HashMap<UrlTokenId, UrlToken>>>,
}

impl FsUrlTokenRepository {
    /// Create a new file system token repository
    ///
    /// # Arguments
    /// * `storage_path` - Path to the JSON file for token storage
    ///
    /// # Returns
    /// A new repository instance, loading existing tokens if present
    pub async fn new(storage_path: PathBuf) -> anyhow::Result<Self> {
        let cache = Arc::new(RwLock::new(HashMap::new()));
        
        // Ensure parent directory exists
        if let Some(parent) = storage_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let repo = Self {
            storage_path,
            cache,
        };

        // Load existing tokens if file exists
        repo.load().await?;

        Ok(repo)
    }

    /// Load tokens from disk
    async fn load(&self) -> anyhow::Result<()> {
        if !self.storage_path.exists() {
            return Ok(());
        }

        let content = tokio::fs::read_to_string(&self.storage_path).await?;
        let tokens: HashMap<UrlTokenId, UrlToken> = serde_json::from_str(&content)?;
        
        let mut cache = self.cache.write().await;
        *cache = tokens;

        Ok(())
    }

    /// Save tokens to disk
    async fn save_to_disk(&self) -> anyhow::Result<()> {
        let cache = self.cache.read().await;
        let content = serde_json::to_string_pretty(&*cache)?;
        
        // Write atomically using temp file
        let temp_path = self.storage_path.with_extension("tmp");
        tokio::fs::write(&temp_path, content).await?;
        tokio::fs::rename(&temp_path, &self.storage_path).await?;

        Ok(())
    }
}

#[async_trait]
impl UrlTokenRepository for FsUrlTokenRepository {
    async fn save(&self, token: &UrlToken) -> anyhow::Result<()> {
        let mut cache = self.cache.write().await;
        cache.insert(token.id.clone(), token.clone());
        drop(cache);
        
        self.save_to_disk().await
    }

    async fn find_by_id(&self, id: &UrlTokenId) -> anyhow::Result<Option<UrlToken>> {
        let cache = self.cache.read().await;
        Ok(cache.get(id).cloned())
    }

    async fn find_by_token(&self, token_str: &str) -> anyhow::Result<Option<UrlToken>> {
        let cache = self.cache.read().await;
        Ok(cache
            .values()
            .find(|t| t.token == token_str)
            .cloned())
    }

    async fn list_all(&self) -> anyhow::Result<HashMap<UrlTokenId, UrlToken>> {
        let cache = self.cache.read().await;
        Ok(cache.clone())
    }

    async fn delete(&self, id: &UrlTokenId) -> anyhow::Result<()> {
        let mut cache = self.cache.write().await;
        cache.remove(id);
        drop(cache);
        
        self.save_to_disk().await
    }

    async fn update(&self, token: &UrlToken) -> anyhow::Result<()> {
        // Save operation is the same as update for this implementation
        self.save(token).await
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use forge_test_kit::TempDir;
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn test_fs_token_repository_save_and_find() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("tokens.json");
        let repo = FsUrlTokenRepository::new(storage_path).await.unwrap();

        let token = UrlToken::new(Duration::hours(24), Some("Test".to_string()));
        repo.save(&token).await.unwrap();

        let actual = repo.find_by_id(&token.id).await.unwrap();

        assert_eq!(actual, Some(token));
    }

    #[tokio::test]
    async fn test_fs_token_repository_find_by_token() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("tokens.json");
        let repo = FsUrlTokenRepository::new(storage_path).await.unwrap();

        let token = UrlToken::new(Duration::hours(24), None);
        let token_str = token.token.clone();
        repo.save(&token).await.unwrap();

        let actual = repo.find_by_token(&token_str).await.unwrap();

        assert_eq!(actual.unwrap().id, token.id);
    }

    #[tokio::test]
    async fn test_fs_token_repository_delete() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("tokens.json");
        let repo = FsUrlTokenRepository::new(storage_path).await.unwrap();

        let token = UrlToken::new(Duration::hours(24), None);
        repo.save(&token).await.unwrap();
        repo.delete(&token.id).await.unwrap();

        let actual = repo.find_by_id(&token.id).await.unwrap();

        assert_eq!(actual, None);
    }

    #[tokio::test]
    async fn test_fs_token_repository_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("tokens.json");

        // Save token
        {
            let repo = FsUrlTokenRepository::new(storage_path.clone()).await.unwrap();
            let token = UrlToken::new(Duration::hours(24), Some("Persistent".to_string()));
            repo.save(&token).await.unwrap();
        }

        // Load from new instance
        {
            let repo = FsUrlTokenRepository::new(storage_path).await.unwrap();
            let tokens = repo.list_all().await.unwrap();
            assert_eq!(tokens.len(), 1);
            assert_eq!(
                tokens.values().next().unwrap().description,
                Some("Persistent".to_string())
            );
        }
    }
}
