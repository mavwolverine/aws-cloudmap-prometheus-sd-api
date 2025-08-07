use aws_sdk_servicediscovery::Client as ServiceDiscoveryClient;
use log::{info, debug};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Config {
    #[allow(dead_code)]
    pub region: Option<String>,
    pub namespace: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct PrometheusTarget {
    pub targets: Vec<String>,
    pub labels: HashMap<String, String>,
}

#[derive(Clone)]
pub struct Discovery {
    client: ServiceDiscoveryClient,
    config: Config,
}

impl Discovery {
    pub fn new(client: ServiceDiscoveryClient, config: Config) -> Self {
        Self { client, config }
    }

    pub async fn discover_targets(&self) -> Result<Vec<PrometheusTarget>, Box<dyn std::error::Error + Send + Sync>> {
        let mut targets = Vec::new();

        // List namespaces
        let namespaces_resp = self.client.list_namespaces().send().await?;

        for namespace in namespaces_resp.namespaces() {
            let namespace_name = namespace.name().unwrap_or("unknown");
            let namespace_id = namespace.id().unwrap_or("");

            // Skip if namespace filter is set and doesn't match
            if let Some(ref filter) = self.config.namespace {
                if namespace_name != filter {
                    continue;
                }
            }

            info!("🔍 Discovering services in namespace: {}", namespace_name);

            // List services in this namespace
            let service_filter = aws_sdk_servicediscovery::types::ServiceFilter::builder()
                .name(aws_sdk_servicediscovery::types::ServiceFilterName::NamespaceId)
                .values(namespace_id)
                .build()?;

            let services_resp = self.client
                .list_services()
                .filters(service_filter)
                .send()
                .await?;

            for service in services_resp.services() {
                debug!("🔍 Complete service object: {:?}", service);

                let service_name = service.name().unwrap_or("unknown");
                let service_id = service.id().unwrap_or("");

                info!("📋 Found service: {} in namespace: {}", service_name, namespace_name);

                // Get instances for this service
                let instances_resp = self.client
                    .list_instances()
                    .service_id(service_id)
                    .send()
                    .await?;

                let mut service_targets = Vec::new();
                for instance in instances_resp.instances() {
                    debug!("🔍 Complete instance object: {:?}", instance);

                    if let Some(attributes) = instance.attributes() {
                        debug!("🔍 Instance attributes: {:?}", attributes);
                        // Look for IP addresses in common attribute names
                        for ip_attr in ["AWS_INSTANCE_IPV4", "IPv4", "ip", "address"] {
                            if let Some(ip) = attributes.get(ip_attr) {
                                debug!("✅ Found IP {} in attribute {}", ip, ip_attr);
                                service_targets.push(ip.clone());
                                break;
                            }
                        }
                    } else {
                        debug!("⚠️  Instance has no attributes");
                    }
                }

                if !service_targets.is_empty() {
                    let mut labels = HashMap::new();
                    labels.insert(
                        "__meta_cloudmap_namespace_name".to_string(),
                        namespace_name.to_string(),
                    );
                    labels.insert(
                        "__meta_cloudmap_service_name".to_string(),
                        service_name.to_string(),
                    );

                    targets.push(PrometheusTarget {
                        targets: service_targets,
                        labels,
                    });
                }
            }
        }

        info!("✅ Successfully discovered {} target groups", targets.len());
        Ok(targets)
    }

    // Helper method for creating Prometheus targets from service instances
    #[cfg(test)]
    pub fn create_prometheus_target(
        &self,
        namespace_name: &str,
        service_name: &str,
        instance_ips: Vec<String>,
    ) -> PrometheusTarget {
        let mut labels = HashMap::new();
        labels.insert(
            "__meta_cloudmap_namespace_name".to_string(),
            namespace_name.to_string(),
        );
        labels.insert(
            "__meta_cloudmap_service_name".to_string(),
            service_name.to_string(),
        );

        PrometheusTarget {
            targets: instance_ips,
            labels,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_discovery() -> Discovery {
        let config = Config {
            region: Some("us-west-2".to_string()),
            namespace: None,
        };

        let aws_config = aws_config::SdkConfig::builder()
            .behavior_version(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new("us-west-2"))
            .build();
        let client = ServiceDiscoveryClient::new(&aws_config);
        Discovery::new(client, config)
    }

    #[test]
    fn test_create_prometheus_target() {
        let discovery = create_test_discovery();

        let target = discovery.create_prometheus_target(
            "test-namespace",
            "test-service",
            vec!["192.168.1.1".to_string(), "192.168.1.2".to_string()],
        );

        assert_eq!(target.targets, vec!["192.168.1.1", "192.168.1.2"]);
        assert_eq!(
            target.labels.get("__meta_cloudmap_namespace_name"),
            Some(&"test-namespace".to_string())
        );
        assert_eq!(
            target.labels.get("__meta_cloudmap_service_name"),
            Some(&"test-service".to_string())
        );
    }

    #[test]
    fn test_create_prometheus_target_with_empty_ips() {
        let discovery = create_test_discovery();

        let target = discovery.create_prometheus_target(
            "test-namespace",
            "test-service",
            vec![],
        );

        assert!(target.targets.is_empty());
        assert_eq!(
            target.labels.get("__meta_cloudmap_namespace_name"),
            Some(&"test-namespace".to_string())
        );
        assert_eq!(
            target.labels.get("__meta_cloudmap_service_name"),
            Some(&"test-service".to_string())
        );
    }

    #[test]
    fn test_create_prometheus_target_with_port() {
        let discovery = create_test_discovery();

        let target = discovery.create_prometheus_target(
            "ns1",
            "svc1",
            vec!["192.168.0.1:8080".to_string()],
        );

        assert_eq!(target.targets, vec!["192.168.0.1:8080"]);
        assert_eq!(
            target.labels.get("__meta_cloudmap_namespace_name"),
            Some(&"ns1".to_string())
        );
        assert_eq!(
            target.labels.get("__meta_cloudmap_service_name"),
            Some(&"svc1".to_string())
        );
    }

    #[test]
    fn test_prometheus_target_serialization() {
        let mut labels = HashMap::new();
        labels.insert("__meta_cloudmap_namespace_name".to_string(), "ns1".to_string());
        labels.insert("__meta_cloudmap_service_name".to_string(), "svc1".to_string());

        let target = PrometheusTarget {
            targets: vec!["192.168.0.1:8080".to_string()],
            labels,
        };

        let json = serde_json::to_string(&target).unwrap();
        let deserialized: PrometheusTarget = serde_json::from_str(&json).unwrap();

        assert_eq!(target, deserialized);
    }

    #[test]
    fn test_config_creation() {
        let config = Config {
            region: Some("us-east-1".to_string()),
            namespace: Some("production".to_string()),
        };

        assert_eq!(config.region, Some("us-east-1".to_string()));
        assert_eq!(config.namespace, Some("production".to_string()));
    }

    #[test]
    fn test_config_with_none_values() {
        let config = Config {
            region: None,
            namespace: None,
        };

        assert_eq!(config.region, None);
        assert_eq!(config.namespace, None);
    }

    #[test]
    fn test_config_clone() {
        let config = Config {
            region: Some("us-west-2".to_string()),
            namespace: Some("test".to_string()),
        };

        let cloned_config = config.clone();
        assert_eq!(config.region, cloned_config.region);
        assert_eq!(config.namespace, cloned_config.namespace);
    }

    #[test]
    fn test_prometheus_target_with_multiple_labels() {
        let discovery = create_test_discovery();

        let target = discovery.create_prometheus_target(
            "production-namespace",
            "web-service",
            vec!["10.0.1.1".to_string(), "10.0.1.2".to_string(), "10.0.1.3".to_string()],
        );

        assert_eq!(target.targets.len(), 3);
        assert!(target.targets.contains(&"10.0.1.1".to_string()));
        assert!(target.targets.contains(&"10.0.1.2".to_string()));
        assert!(target.targets.contains(&"10.0.1.3".to_string()));

        assert_eq!(target.labels.len(), 2);
        assert_eq!(
            target.labels.get("__meta_cloudmap_namespace_name"),
            Some(&"production-namespace".to_string())
        );
        assert_eq!(
            target.labels.get("__meta_cloudmap_service_name"),
            Some(&"web-service".to_string())
        );
    }
}
