//! # Configuration Management
//!
//! This module handles configuration loading from JSON files and environment variables.
//! It supports a hierarchical configuration system where environment variables override
//! JSON file settings, which in turn override default values.
//!
//! ## Configuration Sources (in order of precedence)
//!
//! 1. Environment variables (highest priority)
//! 2. JSON configuration file (`config.json`)
//! 3. Default values (lowest priority)
//!
//! ## Environment Variables
//!
//! - `HOST`: Server bind address
//! - `PORT`: Server port number
//! - `AWS_REGION`: AWS region for Cloud Map operations
//! - `CLOUDMAP_NAMESPACE`: Specific namespace to filter (optional)

use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub aws_region: Option<String>,
    /// Specific Cloud Map namespace to discover
    /// If None, discovers all namespaces
    /// Set via config file or CLOUDMAP_NAMESPACE environment variable
    pub cloudmap_namespace: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3030,
            aws_region: None,
            cloudmap_namespace: None,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        // Try to read config from file
        let mut config = match fs::read_to_string("config.json") {
            Ok(content) => match serde_json::from_str::<Config>(&content) {
                Ok(config) => {
                    info!("📄 Loaded config from config.json");
                    config
                }
                Err(e) => {
                    warn!("⚠️  Failed to parse config.json: {}, using defaults", e);
                    Config::default()
                }
            },
            Err(_) => {
                info!("📄 No config.json found, using defaults");
                Config::default()
            }
        };

        // Override with environment variables if present
        if let Ok(host) = std::env::var("HOST") {
            info!("🌍 HOST environment variable found, overriding config");
            config.host = host;
        }

        if let Ok(port_str) = std::env::var("PORT") {
            if let Ok(port) = port_str.parse::<u16>() {
                info!("🔌 PORT environment variable found, overriding config");
                config.port = port;
            } else {
                warn!("⚠️  Invalid PORT environment variable: {}", port_str);
            }
        }

        if let Ok(region) = std::env::var("AWS_REGION") {
            info!("🌍 AWS_REGION environment variable found, overriding config");
            config.aws_region = Some(region);
        }

        if let Ok(namespace) = std::env::var("CLOUDMAP_NAMESPACE") {
            info!("🗂️  CLOUDMAP_NAMESPACE environment variable found, overriding config");
            config.cloudmap_namespace = Some(namespace);
        }

        config
    }

    pub fn parse_host(&self) -> Result<[u8; 4], String> {
        let parts: Vec<&str> = self.host.split('.').collect();

        if parts.len() != 4 {
            return Err(format!(
                "Invalid IP format: expected 4 parts, got {}",
                parts.len()
            ));
        }

        let mut result = [0u8; 4];
        for (i, part) in parts.iter().enumerate() {
            match part.parse::<u8>() {
                Ok(num) => result[i] = num,
                Err(_) => return Err(format!("Invalid IP part: '{}' is not a valid number", part)),
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();

        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 3030);
        assert_eq!(config.aws_region, None);
        assert_eq!(config.cloudmap_namespace, None);
    }

    #[test]
    fn test_parse_host_valid_ip() {
        let config = Config {
            host: "192.168.1.1".to_string(),
            port: 8080,
            aws_region: None,
            cloudmap_namespace: None,
        };

        let result = config.parse_host().unwrap();
        assert_eq!(result, [192, 168, 1, 1]);
    }

    #[test]
    fn test_parse_host_localhost() {
        let config = Config {
            host: "127.0.0.1".to_string(),
            port: 3000,
            aws_region: None,
            cloudmap_namespace: None,
        };

        let result = config.parse_host().unwrap();
        assert_eq!(result, [127, 0, 0, 1]);
    }

    #[test]
    fn test_parse_host_all_interfaces() {
        let config = Config {
            host: "0.0.0.0".to_string(),
            port: 3030,
            aws_region: None,
            cloudmap_namespace: None,
        };

        let result = config.parse_host().unwrap();
        assert_eq!(result, [0, 0, 0, 0]);
    }

    #[test]
    fn test_parse_host_invalid_format() {
        let config = Config {
            host: "192.168.1".to_string(), // Missing fourth octet
            port: 3030,
            aws_region: None,
            cloudmap_namespace: None,
        };

        let result = config.parse_host();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid IP format"));
    }

    #[test]
    fn test_parse_host_invalid_number() {
        let config = Config {
            host: "192.168.1.256".to_string(), // 256 is out of range for u8
            port: 3030,
            aws_region: None,
            cloudmap_namespace: None,
        };

        let result = config.parse_host();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid IP part"));
    }

    #[test]
    fn test_parse_host_non_numeric() {
        let config = Config {
            host: "192.168.1.abc".to_string(),
            port: 3030,
            aws_region: None,
            cloudmap_namespace: None,
        };

        let result = config.parse_host();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid IP part"));
    }

    #[test]
    fn test_config_clone() {
        let config = Config {
            host: "10.0.0.1".to_string(),
            port: 8080,
            aws_region: Some("us-east-1".to_string()),
            cloudmap_namespace: Some("test-namespace".to_string()),
        };

        let cloned = config.clone();
        assert_eq!(config.host, cloned.host);
        assert_eq!(config.port, cloned.port);
        assert_eq!(config.aws_region, cloned.aws_region);
        assert_eq!(config.cloudmap_namespace, cloned.cloudmap_namespace);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config {
            host: "192.168.1.100".to_string(),
            port: 9090,
            aws_region: Some("eu-west-1".to_string()),
            cloudmap_namespace: Some("production".to_string()),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(config.host, deserialized.host);
        assert_eq!(config.port, deserialized.port);
        assert_eq!(config.aws_region, deserialized.aws_region);
        assert_eq!(config.cloudmap_namespace, deserialized.cloudmap_namespace);
    }

    // Note: Testing Config::load() with actual file I/O and env vars would require
    // more complex setup with temporary files and env var manipulation.
    // For now, we test the individual components that make up the load functionality.
}
