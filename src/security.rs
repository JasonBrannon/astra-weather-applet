// SPDX-License-Identifier: GPL-3.0-or-later

//! Secure API key storage and management using system keyring
//!
//! This module provides secure credential storage using the freedesktop.org Secret Service
//! API (via D-Bus). On COSMIC/GNOME systems, this integrates with gnome-keyring.
//!
//! **Backend**: Uses `sync-secret-service` for persistent storage across reboots.
//! The Secret Service API stores credentials persistently (survives reboot/logout).
//!
//! The keyring crate automatically handles:
//! - D-Bus communication with Secret Service (gnome-keyring, KWallet)
//! - Persistent encrypted storage (survives reboot)
//! - Cross-platform compatibility (Linux, Windows, macOS)

use crate::constants::*;
use keyring::{Entry, Error as KeyringError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("API key not found")]
    ApiKeyNotFound,
    #[error("Keyring access failed: {0}")]
    KeyringError(String),
    #[error("API key validation failed: {0}")]
    ValidationError(String),
}

pub type SecurityResult<T> = Result<T, SecurityError>;

/// Secure API key manager using system keyring via D-Bus Secret Service
pub struct ApiKeyManager {
    entry: Entry,
}

impl ApiKeyManager {
    /// Create a new API key manager
    ///
    /// This connects to the system's Secret Service via D-Bus (on Linux/COSMIC)
    /// and creates a credential entry for the Tempest API key.
    pub fn new() -> Self {
        // Create keyring entry for Tempest API key
        // service: "com.slagmine.astra"
        // username: "api-key"
        let entry =
            Entry::new(KEYRING_SERVICE, KEYRING_USERNAME).expect("Failed to create keyring entry");

        Self { entry }
    }

    /// Store API key securely in system keyring
    ///
    /// On COSMIC/GNOME: Stored in gnome-keyring via Secret Service D-Bus API
    /// **Persistent**: Survives reboot/logout (uses sync-secret-service backend)
    pub fn store_api_key(&self, api_key: &str) -> SecurityResult<()> {
        tracing::debug!("store_api_key called with key length: {}", api_key.len());

        self.validate_api_key(api_key)?;
        tracing::debug!("API key validation passed");

        tracing::info!(
            "Attempting to store API key via keyring (service: {}, username: {})",
            KEYRING_SERVICE,
            KEYRING_USERNAME
        );

        self.entry.set_password(api_key).map_err(|e| match e {
            KeyringError::NoStorageAccess(inner) => {
                tracing::error!("Secret Service unavailable, storage failed: {}", inner);
                SecurityError::KeyringError(format!("Storage access error: {}", inner))
            }
            KeyringError::PlatformFailure(inner) => {
                tracing::error!("Platform error during keyring storage: {}", inner);
                SecurityError::KeyringError(format!("Platform error: {}", inner))
            }
            _ => {
                tracing::error!("Unknown keyring error: {}", e);
                SecurityError::KeyringError(format!("Failed to store API key: {}", e))
            }
        })?;

        tracing::info!("✓ API key stored successfully in system keyring");

        // Immediately verify storage
        match self.entry.get_password() {
            Ok(retrieved) => {
                if retrieved == api_key {
                    tracing::info!("✓ API key storage verified - retrieval successful");
                } else {
                    tracing::error!("✗ API key verification failed - retrieved key doesn't match!");
                }
            }
            Err(e) => {
                tracing::error!("✗ API key verification failed - cannot retrieve: {}", e);
            }
        }

        Ok(())
    }

    /// Retrieve API key from system keyring
    pub fn retrieve_api_key(&self) -> SecurityResult<String> {
        tracing::debug!(
            "Attempting to retrieve API key from keyring (service: {}, username: {})",
            KEYRING_SERVICE,
            KEYRING_USERNAME
        );

        let api_key = self.entry.get_password().map_err(|e| match e {
            KeyringError::NoEntry => {
                tracing::debug!("API key not found in keyring (NoEntry)");
                SecurityError::ApiKeyNotFound
            }
            KeyringError::NoStorageAccess(inner) => {
                tracing::error!("Storage access error during retrieval: {}", inner);
                SecurityError::KeyringError(format!("Storage access error: {}", inner))
            }
            KeyringError::PlatformFailure(inner) => {
                tracing::error!("Platform error during retrieval: {}", inner);
                SecurityError::KeyringError(format!("Platform error: {}", inner))
            }
            _ => {
                tracing::error!("Unknown keyring error during retrieval: {}", e);
                SecurityError::KeyringError(format!("Failed to retrieve API key: {}", e))
            }
        })?;

        tracing::debug!(
            "✓ API key retrieved from keyring, length: {}",
            api_key.len()
        );

        self.validate_api_key(&api_key)?;
        tracing::debug!("✓ Retrieved API key passed validation");

        Ok(api_key)
    }

    /// Remove API key from system keyring
    pub fn remove_api_key(&self) -> SecurityResult<()> {
        match self.entry.delete_credential() {
            Ok(()) => {
                tracing::info!("API key removed from system keyring");
                Ok(())
            }
            Err(KeyringError::NoEntry) => {
                // Already deleted, consider this success
                tracing::debug!("API key already removed from keyring");
                Ok(())
            }
            Err(KeyringError::NoStorageAccess(inner)) => Err(SecurityError::KeyringError(format!(
                "Storage access error: {}",
                inner
            ))),
            Err(KeyringError::PlatformFailure(inner)) => Err(SecurityError::KeyringError(format!(
                "Platform error: {}",
                inner
            ))),
            Err(e) => Err(SecurityError::KeyringError(format!(
                "Failed to remove API key: {}",
                e
            ))),
        }
    }

    /// Check if API key exists in keyring
    pub fn has_api_key(&self) -> bool {
        self.retrieve_api_key().is_ok()
    }

    /// Validate API key format
    fn validate_api_key(&self, api_key: &str) -> SecurityResult<()> {
        if api_key.len() < MIN_API_KEY_LENGTH {
            return Err(SecurityError::ValidationError(format!(
                "API key too short (minimum {} characters)",
                MIN_API_KEY_LENGTH
            )));
        }

        if api_key.len() > MAX_API_KEY_LENGTH {
            return Err(SecurityError::ValidationError(format!(
                "API key too long (maximum {} characters)",
                MAX_API_KEY_LENGTH
            )));
        }

        // API key should be alphanumeric and possibly contain dashes/underscores
        if !api_key
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(SecurityError::ValidationError(
                "API key contains invalid characters".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for ApiKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_validation() {
        let manager = ApiKeyManager::new();

        // Valid key
        assert!(manager.validate_api_key("valid-api-key-123").is_ok());

        // Too short
        assert!(manager.validate_api_key("short").is_err());

        // Invalid characters
        assert!(manager.validate_api_key("invalid@key!").is_err());
    }
}
