use warp::Filter;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    host: String,
    port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3030,
        }
    }
}

fn load_config() -> Config {
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

    config
}

fn parse_host(host_str: &str) -> Result<[u8; 4], String> {
    let parts: Vec<&str> = host_str.split('.').collect();
    
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

#[tokio::main]
async fn main() {
    // Initialize the logger with info as default, but allow RUST_LOG to override
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    // Load configuration
    let config = load_config();
    
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello" / String)
        .map(|name| format!("Hello, {}!", name));

    // Add logging middleware for requests
    let routes = hello.with(warp::log("api"));

    let host = match parse_host(&config.host) {
        Ok(host_array) => host_array,
        Err(e) => {
            warn!("⚠️  Failed to parse host '{}': {}, using 0.0.0.0", config.host, e);
            [0, 0, 0, 0]
        }
    };
    let addr = (host, config.port);
    
    info!("🚀 Server starting...");
    info!("📡 Listening on http://{}:{}", config.host, config.port);
    info!("🔗 Try: http://localhost:{}/hello/world", config.port);
    warn!("Press Ctrl+C to stop the server");

    warp::serve(routes)
        .run(addr)
        .await;
}
