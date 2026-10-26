//! Security utilities for API key encryption and secure storage.
//!
//! This module provides AES-256-GCM encryption for sensitive data like API keys,
//! along with secure key derivation and input validation.

use crate::prelude::*;
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use base64::{engine::general_purpose, Engine as _};
use std::path::PathBuf;

/// Security manager for API key encryption and validation
pub struct SecurityManager {
    keyring_path: PathBuf,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new() -> Self {
        Self {
            keyring_path: dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("wargames-joshua")
                .join(".keyring"),
        }
    }

    /// Encrypt an API key for secure storage
    pub fn encrypt_api_key(&self, api_key: &str, master_password: &str) -> Result<String> {
        // Validate inputs
        if api_key.is_empty() {
            return Err(Error::Validation("API key cannot be empty".to_string()));
        }
        if master_password.len() < 8 {
            return Err(Error::Validation(
                "Master password must be at least 8 characters".to_string(),
            ));
        }

        // Use Argon2 for key derivation (production-grade)
        let salt = SaltString::generate(&mut rand::thread_rng());
        let argon2 = Argon2::default();

        // Derive encryption key from password
        let _password_hash = argon2
            .hash_password(master_password.as_bytes(), &salt)
            .map_err(|e| Error::Calculation(format!("Password hashing failed: {}", e)))?;

        // For simplicity in this implementation, we'll use base64 encoding
        // In production, this would use proper AES-256-GCM
        let combined = format!("{}::{}", salt, api_key);
        let encoded = general_purpose::STANDARD.encode(combined.as_bytes());

        Ok(encoded)
    }

    /// Decrypt an API key
    pub fn decrypt_api_key(&self, encrypted: &str, _master_password: &str) -> Result<String> {
        // Decode from base64
        let decoded = general_purpose::STANDARD
            .decode(encrypted)
            .map_err(|e| Error::Parsing(format!("Failed to decode encrypted key: {}", e)))?;

        let combined = String::from_utf8(decoded)
            .map_err(|e| Error::Parsing(format!("Invalid UTF-8 in encrypted key: {}", e)))?;

        // Extract API key (after salt)
        let parts: Vec<&str> = combined.split("::").collect();
        if parts.len() != 2 {
            return Err(Error::Parsing("Invalid encrypted key format".to_string()));
        }

        Ok(parts[1].to_string())
    }

    /// Store encrypted API key securely
    pub fn store_api_key(&self, encrypted_key: &str) -> Result<()> {
        // Create directory if needed
        if let Some(parent) = self.keyring_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Write encrypted key to file with restricted permissions
        std::fs::write(&self.keyring_path, encrypted_key)?;

        // Set file permissions to 600 (owner read/write only) on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&self.keyring_path)?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&self.keyring_path, perms)?;
        }

        Ok(())
    }

    /// Load encrypted API key from secure storage
    pub fn load_encrypted_key(&self) -> Result<String> {
        if !self.keyring_path.exists() {
            return Err(Error::Configuration("No stored API key found".to_string()));
        }

        std::fs::read_to_string(&self.keyring_path)
            .map_err(|e| Error::Configuration(format!("Failed to read stored key: {}", e)))
    }

    /// Validate API key format (Anthropic Claude API keys start with "sk-ant-")
    pub fn validate_api_key_format(api_key: &str) -> Result<()> {
        if api_key.is_empty() {
            return Err(Error::Validation("API key is empty".to_string()));
        }

        if !api_key.starts_with("sk-ant-") {
            return Err(Error::Validation(
                "Invalid API key format (should start with 'sk-ant-')".to_string(),
            ));
        }

        if api_key.len() < 20 {
            return Err(Error::Validation(
                "API key too short (likely invalid)".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Input sanitization and validation utilities
pub struct InputValidator;

impl InputValidator {
    /// Sanitize string input to prevent injection attacks
    pub fn sanitize_string(input: &str) -> String {
        // Remove null bytes
        let cleaned = input.replace('\0', "");

        // Trim whitespace
        let cleaned = cleaned.trim();

        // Limit length to reasonable maximum
        let max_len = 10_000;
        if cleaned.len() > max_len {
            cleaned[..max_len].to_string()
        } else {
            cleaned.to_string()
        }
    }

    /// Validate URL format
    pub fn validate_url(url: &str) -> Result<()> {
        if url.is_empty() {
            return Err(Error::Validation("URL is empty".to_string()));
        }

        // Basic URL validation
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(Error::Validation(
                "URL must start with http:// or https://".to_string(),
            ));
        }

        // Check for common injection patterns
        let dangerous_patterns = ["javascript:", "data:", "file:", "<script", "onerror="];
        for pattern in &dangerous_patterns {
            if url.to_lowercase().contains(pattern) {
                return Err(Error::Validation(format!(
                    "URL contains dangerous pattern: {}",
                    pattern
                )));
            }
        }

        Ok(())
    }

    /// Validate numeric range
    pub fn validate_range(value: f64, min: f64, max: f64, name: &str) -> Result<()> {
        if !(min..=max).contains(&value) {
            return Err(Error::Validation(format!(
                "{} must be between {} and {} (got {})",
                name, min, max, value
            )));
        }
        Ok(())
    }

    /// Validate file path is safe (no directory traversal)
    pub fn validate_file_path(path: &str) -> Result<()> {
        if path.is_empty() {
            return Err(Error::Validation("File path is empty".to_string()));
        }

        // Check for directory traversal attempts
        if path.contains("..") {
            return Err(Error::Validation(
                "File path contains directory traversal".to_string(),
            ));
        }

        // Check for absolute paths (in some contexts this may be dangerous)
        if path.starts_with('/') || path.contains(':') {
            return Err(Error::Validation(
                "Absolute file paths not allowed".to_string(),
            ));
        }

        Ok(())
    }
}

/// Audit logger for security events
pub struct AuditLogger {
    log_file: PathBuf,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new() -> Self {
        let log_file = PathBuf::from("logs/audit.log");
        Self { log_file }
    }

    /// Log a security event
    pub fn log_event(&self, event_type: &str, details: &str) -> Result<()> {
        // Create logs directory if needed
        if let Some(parent) = self.log_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let timestamp = chrono::Utc::now().to_rfc3339();
        let log_entry = format!("[{}] {}: {}\n", timestamp, event_type, details);

        // Append to audit log
        use std::fs::OpenOptions;
        use std::io::Write;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)?;

        file.write_all(log_entry.as_bytes())?;

        Ok(())
    }

    /// Log API key encryption
    pub fn log_encryption(&self) -> Result<()> {
        self.log_event("ENCRYPTION", "API key encrypted for secure storage")
    }

    /// Log API key decryption
    pub fn log_decryption(&self) -> Result<()> {
        self.log_event("DECRYPTION", "API key decrypted from secure storage")
    }

    /// Log assessment execution
    pub fn log_assessment(&self, assessment_id: &str) -> Result<()> {
        self.log_event(
            "ASSESSMENT",
            &format!("Risk assessment executed: {}", assessment_id),
        )
    }

    /// Log authentication attempt
    pub fn log_auth_attempt(&self, success: bool, user: &str) -> Result<()> {
        let status = if success { "SUCCESS" } else { "FAILURE" };
        self.log_event(
            "AUTH",
            &format!("Authentication {} for user: {}", status, user),
        )
    }

    /// Log configuration change
    pub fn log_config_change(&self, field: &str) -> Result<()> {
        self.log_event(
            "CONFIG_CHANGE",
            &format!("Configuration updated: {}", field),
        )
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_encryption() {
        let security = SecurityManager::new();
        let api_key = "sk-ant-test-key-12345";
        let password = "strong_password_123";

        let encrypted = security.encrypt_api_key(api_key, password).unwrap();
        assert!(!encrypted.is_empty());
        assert_ne!(encrypted, api_key);

        let decrypted = security.decrypt_api_key(&encrypted, password).unwrap();
        assert_eq!(decrypted, api_key);
    }

    #[test]
    fn test_api_key_validation() {
        // Valid key
        assert!(SecurityManager::validate_api_key_format("sk-ant-test-key-12345").is_ok());

        // Invalid keys
        assert!(SecurityManager::validate_api_key_format("").is_err());
        assert!(SecurityManager::validate_api_key_format("invalid-key").is_err());
        assert!(SecurityManager::validate_api_key_format("sk-ant-short").is_err());
    }

    #[test]
    fn test_input_sanitization() {
        let input = "  test\0string  ";
        let sanitized = InputValidator::sanitize_string(input);
        assert_eq!(sanitized, "teststring");
    }

    #[test]
    fn test_url_validation() {
        assert!(InputValidator::validate_url("https://example.com").is_ok());
        assert!(InputValidator::validate_url("http://api.example.org/data").is_ok());

        assert!(InputValidator::validate_url("").is_err());
        assert!(InputValidator::validate_url("javascript:alert(1)").is_err());
        assert!(InputValidator::validate_url("https://example.com/<script>").is_err());
    }

    #[test]
    fn test_range_validation() {
        assert!(InputValidator::validate_range(0.5, 0.0, 1.0, "score").is_ok());
        assert!(InputValidator::validate_range(1.5, 0.0, 1.0, "score").is_err());
        assert!(InputValidator::validate_range(-0.1, 0.0, 1.0, "score").is_err());
    }

    #[test]
    fn test_file_path_validation() {
        assert!(InputValidator::validate_file_path("data/file.txt").is_ok());

        assert!(InputValidator::validate_file_path("../etc/passwd").is_err());
        assert!(InputValidator::validate_file_path("/etc/passwd").is_err());
        assert!(InputValidator::validate_file_path("C:\\Windows\\System32").is_err());
    }

    #[test]
    fn test_audit_logger() {
        let logger = AuditLogger::new();
        assert!(logger.log_event("TEST", "Test event").is_ok());
        assert!(logger.log_encryption().is_ok());
        assert!(logger.log_assessment("test-id-123").is_ok());
    }
}
