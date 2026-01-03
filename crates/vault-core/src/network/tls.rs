//! # TLS Configuration Module
//!
//! Provides TLS configuration and helpers for secure network communication.
//! TLS is disabled by default but can be enabled via configuration.

use std::path::PathBuf;
use thiserror::Error;

/// TLS-related errors
#[derive(Error, Debug)]
pub enum TlsError {
    #[error("TLS is not enabled")]
    NotEnabled,

    #[error("Certificate file not found: {0}")]
    CertificateNotFound(PathBuf),

    #[error("Key file not found: {0}")]
    KeyNotFound(PathBuf),

    #[error("Invalid certificate: {0}")]
    InvalidCertificate(String),

    #[error("Invalid key: {0}")]
    InvalidKey(String),

    #[error("TLS initialization failed: {0}")]
    InitializationFailed(String),
}

/// TLS configuration for secure network communication.
///
/// By default, TLS is disabled. To enable TLS, set `enabled` to true
/// and provide paths to the certificate and key files.
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Whether TLS is enabled
    pub enabled: bool,

    /// Path to the TLS certificate file (PEM format)
    pub cert_path: Option<PathBuf>,

    /// Path to the TLS private key file (PEM format)
    pub key_path: Option<PathBuf>,

    /// Path to the CA certificate file for client verification (optional)
    pub ca_cert_path: Option<PathBuf>,

    /// Whether to verify client certificates (mTLS)
    pub verify_client: bool,

    /// Minimum TLS version to accept (1.2 or 1.3)
    pub min_tls_version: TlsVersion,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cert_path: None,
            key_path: None,
            ca_cert_path: None,
            verify_client: false,
            min_tls_version: TlsVersion::Tls12,
        }
    }
}

impl TlsConfig {
    /// Create a new TLS configuration with TLS disabled
    pub fn disabled() -> Self {
        Self::default()
    }

    /// Create a new TLS configuration with TLS enabled
    pub fn enabled(cert_path: PathBuf, key_path: PathBuf) -> Self {
        Self {
            enabled: true,
            cert_path: Some(cert_path),
            key_path: Some(key_path),
            ca_cert_path: None,
            verify_client: false,
            min_tls_version: TlsVersion::Tls12,
        }
    }

    /// Create a new TLS configuration with mutual TLS (mTLS)
    pub fn mtls(cert_path: PathBuf, key_path: PathBuf, ca_cert_path: PathBuf) -> Self {
        Self {
            enabled: true,
            cert_path: Some(cert_path),
            key_path: Some(key_path),
            ca_cert_path: Some(ca_cert_path),
            verify_client: true,
            min_tls_version: TlsVersion::Tls12,
        }
    }

    /// Set the minimum TLS version
    pub fn with_min_version(mut self, version: TlsVersion) -> Self {
        self.min_tls_version = version;
        self
    }

    /// Validate the TLS configuration
    pub fn validate(&self) -> Result<(), TlsError> {
        if !self.enabled {
            return Ok(());
        }

        // Check certificate path
        if let Some(cert_path) = &self.cert_path {
            if !cert_path.exists() {
                return Err(TlsError::CertificateNotFound(cert_path.clone()));
            }
        } else {
            return Err(TlsError::InvalidCertificate("Certificate path not provided".to_string()));
        }

        // Check key path
        if let Some(key_path) = &self.key_path {
            if !key_path.exists() {
                return Err(TlsError::KeyNotFound(key_path.clone()));
            }
        } else {
            return Err(TlsError::InvalidKey("Key path not provided".to_string()));
        }

        // Check CA certificate path if mTLS is enabled
        if self.verify_client {
            if let Some(ca_cert_path) = &self.ca_cert_path {
                if !ca_cert_path.exists() {
                    return Err(TlsError::CertificateNotFound(ca_cert_path.clone()));
                }
            } else {
                return Err(TlsError::InvalidCertificate("CA certificate path not provided for mTLS".to_string()));
            }
        }

        Ok(())
    }

    /// Check if TLS is enabled and valid
    pub fn is_ready(&self) -> bool {
        self.enabled && self.validate().is_ok()
    }
}

/// Supported TLS versions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsVersion {
    /// TLS 1.2
    Tls12,
    /// TLS 1.3 (recommended for new deployments)
    Tls13,
}

impl TlsVersion {
    /// Get the version string
    pub fn as_str(&self) -> &'static str {
        match self {
            TlsVersion::Tls12 => "TLS 1.2",
            TlsVersion::Tls13 => "TLS 1.3",
        }
    }
}

/// TLS status information for monitoring
#[derive(Debug, Clone)]
pub struct TlsStatus {
    /// Whether TLS is enabled
    pub enabled: bool,

    /// Whether TLS is properly configured
    pub configured: bool,

    /// Minimum TLS version
    pub min_version: Option<TlsVersion>,

    /// Whether mTLS (mutual TLS) is enabled
    pub mtls_enabled: bool,

    /// Any configuration errors
    pub error: Option<String>,
}

impl TlsStatus {
    /// Create a TLS status from a configuration
    pub fn from_config(config: &TlsConfig) -> Self {
        let error = match config.validate() {
            Ok(()) => None,
            Err(e) => Some(e.to_string()),
        };

        Self {
            enabled: config.enabled,
            configured: error.is_none() && config.enabled,
            min_version: if config.enabled { Some(config.min_tls_version) } else { None },
            mtls_enabled: config.verify_client,
            error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_default_tls_disabled() {
        let config = TlsConfig::default();
        assert!(!config.enabled);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_enabled_without_paths_fails() {
        let config = TlsConfig {
            enabled: true,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_tls_version_string() {
        assert_eq!(TlsVersion::Tls12.as_str(), "TLS 1.2");
        assert_eq!(TlsVersion::Tls13.as_str(), "TLS 1.3");
    }

    #[test]
    fn test_tls_status_from_disabled_config() {
        let config = TlsConfig::disabled();
        let status = TlsStatus::from_config(&config);
        assert!(!status.enabled);
        assert!(!status.configured);
        assert!(!status.mtls_enabled);
        assert!(status.error.is_none());
    }
}
