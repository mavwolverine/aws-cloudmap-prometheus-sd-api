//! # AWS Cloud Map Prometheus Service Discovery API
//!
//! A Rust-based HTTP API service discovery adapter for Prometheus that integrates with AWS Cloud Map.
//! This service provides an HTTP endpoint that serves service discovery data in Prometheus-compatible
//! JSON format, allowing you to dynamically discover targets registered in AWS Cloud Map.
//!
//! ## Features
//!
//! - HTTP API endpoint at `/cloudmap_sd`
//! - Real-time discovery from AWS Cloud Map
//! - Optional namespace filtering
//! - Prometheus-compatible JSON output
//! - Configurable via JSON file and environment variables
//! - Structured logging with configurable levels
//!
//! ## Usage
//!
//! ```bash
//! # Run with default configuration
//! cargo run
//!
//! # Run with environment overrides
//! AWS_REGION=us-east-1 PORT=8080 cargo run
//!
//! # Test the endpoint
//! curl http://localhost:3030/cloudmap_sd
//! ```

mod config;
mod discovery;
mod handlers;

use config::Config;
use discovery::Discovery;
use handlers::cloudmap_sd_handler;
use log::{info, warn};
use warp::Filter;
use aws_sdk_servicediscovery::Client as ServiceDiscoveryClient;

#[tokio::main]
async fn main() {
    // Initialize the logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    // Load configuration
    let config = Config::load();

    // Initialize AWS SDK
    let aws_config = match config.aws_region.as_ref() {
        Some(region) => {
            info!("🌍 Using AWS region from config: {}", region);
            aws_config::defaults(aws_config::BehaviorVersion::latest())
                .region(aws_config::Region::new(region.clone()))
                .load()
                .await
        }
        None => {
            info!("🌍 Using default AWS region from environment/profile");
            aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await
        }
    };

    let servicediscovery_client = ServiceDiscoveryClient::new(&aws_config);
    
    // Log the actual region being used
    if let Some(region) = aws_config.region() {
        info!("🌍 AWS SDK initialized with region: {}", region);
    } else {
        warn!("⚠️  No AWS region configured!");
    }
    
    // Create discovery instance
    let discovery_config = discovery::Config {
        region: config.aws_region.clone(),
        namespace: config.cloudmap_namespace.clone(),
    };
    let discovery = Discovery::new(servicediscovery_client, discovery_config);

    // Single route for Cloud Map service discovery
    let cloudmap_route = warp::path("cloudmap_sd")
        .and(warp::get())
        .and_then(move || {
            let discovery = discovery.clone();
            cloudmap_sd_handler(discovery)
        })
        .with(warp::log("api"));

    let host = match config.parse_host() {
        Ok(host_array) => host_array,
        Err(e) => {
            warn!("⚠️  Failed to parse host '{}': {}, using 0.0.0.0", config.host, e);
            [0, 0, 0, 0]
        }
    };
    let addr = (host, config.port);
    
    info!("🚀 Server starting...");
    info!("📡 Listening on http://{}:{}", config.host, config.port);
    info!("📋 Available endpoint:");
    info!("  GET /cloudmap_sd - AWS Cloud Map service discovery for Prometheus");
    info!("🔗 Try: http://localhost:{}/cloudmap_sd", config.port);
    warn!("Press Ctrl+C to stop the server");

    warp::serve(cloudmap_route)
        .run(addr)
        .await;
}
