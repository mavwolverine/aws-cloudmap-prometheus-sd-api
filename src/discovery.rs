use aws_sdk_servicediscovery::Client as ServiceDiscoveryClient;
use log::info;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Config {
    #[allow(dead_code)]
    pub region: Option<String>,
    pub namespace: Option<String>,
}

#[derive(Serialize, Debug)]
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
                    if let Some(attributes) = instance.attributes() {
                        // Look for IP addresses in common attribute names
                        for ip_attr in ["AWS_INSTANCE_IPV4", "IPv4", "ip", "address"] {
                            if let Some(ip) = attributes.get(ip_attr) {
                                service_targets.push(ip.clone());
                                break;
                            }
                        }
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
}
