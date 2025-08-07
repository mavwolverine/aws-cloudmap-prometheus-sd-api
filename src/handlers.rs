//! # HTTP Request Handlers
//!
//! This module contains HTTP request handlers for the service discovery API.
//! It provides the main endpoint for Prometheus service discovery integration.
//!
//! ## Endpoints
//!
//! - `GET /cloudmap_sd`: Returns Prometheus-compatible service discovery JSON
//!
//! ## Error Handling
//!
//! All AWS API errors are caught and converted to HTTP 500 responses with
//! appropriate logging for debugging purposes.

use crate::discovery::Discovery;
use log::error;
use warp::{Rejection, Reply};

/// Custom error type for Cloud Map discovery failures
///
/// This error is returned when the service discovery process fails,
/// typically due to AWS API errors or network issues.
#[derive(Debug)]
pub struct CloudMapError;
impl warp::reject::Reject for CloudMapError {}

/// HTTP handler for the `/cloudmap_sd` endpoint
///
/// This handler performs AWS Cloud Map service discovery and returns
/// the results in Prometheus-compatible JSON format.
///
/// # Arguments
///
/// * `discovery` - Discovery client configured with AWS credentials and settings
///
/// # Returns
///
/// * `Ok(impl Reply)` - JSON response with discovered targets
/// * `Err(Rejection)` - HTTP error response (500 for discovery failures)
///
/// # Response Format
///
/// Returns a JSON array of target objects:
/// ```json
/// [
///   {
///     "targets": ["192.168.1.1", "192.168.1.2"],
///     "labels": {
///       "__meta_cloudmap_namespace_name": "production",
///       "__meta_cloudmap_service_name": "web-service"
///     }
///   }
/// ]
/// ```
pub async fn cloudmap_sd_handler(discovery: Discovery) -> Result<impl Reply, Rejection> {
    match discovery.discover_targets().await {
        Ok(targets) => Ok(warp::reply::json(&targets)),
        Err(e) => {
            error!("❌ Failed to discover Cloud Map targets: {:?}", e);
            error!("❌ Error details: {}", e);
            Err(warp::reject::custom(CloudMapError))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::{Config, PrometheusTarget};
    use std::collections::HashMap;

    #[test]
    fn test_cloudmap_error_debug() {
        let error = CloudMapError;
        let debug_str = format!("{:?}", error);
        assert_eq!(debug_str, "CloudMapError");
    }

    #[test]
    fn test_prometheus_target_creation() {
        // Test that we can create a PrometheusTarget (used in handlers)
        let mut labels = HashMap::new();
        labels.insert(
            "__meta_cloudmap_namespace_name".to_string(),
            "test-ns".to_string(),
        );
        labels.insert(
            "__meta_cloudmap_service_name".to_string(),
            "test-svc".to_string(),
        );

        let target = PrometheusTarget {
            targets: vec!["192.168.1.1:8080".to_string()],
            labels,
        };

        assert_eq!(target.targets.len(), 1);
        assert_eq!(target.labels.len(), 2);
    }

    #[test]
    fn test_discovery_config_for_handler() {
        // Test creating a discovery config that would be used by handlers
        let config = Config {
            region: Some("us-west-2".to_string()),
            namespace: Some("production".to_string()),
        };

        assert_eq!(config.region, Some("us-west-2".to_string()));
        assert_eq!(config.namespace, Some("production".to_string()));
    }

    // Note: Testing the actual cloudmap_sd_handler function would require
    // mocking the AWS SDK client, which is complex. The handler logic is
    // simple - it calls discovery.discover_targets() and handles the result.
    // The real testing happens in the discovery module tests.
}
