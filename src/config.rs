use serde::{Deserialize, Serialize};
use log::{info, warn};
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub aws_region: Option<String>,
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
            Ok(content) => {
                match serde_json::from_str::<Config>(&content) {
                    Ok(config) => {
                        info!("📄 Loaded config from config.json");
                        config
                    }
                    Err(e) => {
                        warn!("⚠️  Failed to parse config.json: {}, using defaults", e);
                        Config::default()
                    }
                }
            }
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
            return Err(format!("Invalid IP format: expected 4 parts, got {}", parts.len()));
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
