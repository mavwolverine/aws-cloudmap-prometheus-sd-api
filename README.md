# aws-cloudmap-prometheus-sd-api

A Rust-based HTTP API service discovery adapter for Prometheus that integrates with [AWS Cloud Map](https://aws.amazon.com/cloud-map/). This provides an HTTP endpoint that serves service discovery data in Prometheus-compatible JSON format, allowing you to dynamically discover targets registered in AWS Cloud Map without static configuration.

It is based on https://github.com/awslabs/aws-cloudmap-prometheus-sd, adding an API instead of saving it to a file.

AWS Cloud Map is a cloud resource discovery service. With Cloud Map, you can define custom names for your application resources, and it maintains the updated location of these dynamically changing resources. This increases your application availability because your web service always discovers the most up-to-date locations of its resources.

## Features

- **HTTP API**: Serves discovery data via REST endpoint instead of file-based approach
- **Real-time Discovery**: Dynamically discovers services from AWS Cloud Map
- **Namespace Filtering**: Optional filtering by specific Cloud Map namespaces
- **Prometheus Compatible**: Returns JSON in Prometheus HTTP service discovery format
- **Configurable**: JSON configuration with environment variable overrides
- **Comprehensive Logging**: Structured logging with configurable levels

## Usage

### 1. Clone and Build

```bash
git clone <repository-url>
cd aws-cloudmap-prometheus-sd-api
cargo build --release
```

### 2. Configuration

Create a `config.json` file:

```json
{
  "host": "0.0.0.0",
  "port": 3030,
  "aws_region": "us-west-2",
  "cloudmap_namespace": null
}
```

**Configuration Options:**

- `host`: IP address to bind the server (default: "0.0.0.0")
- `port`: Port to listen on (default: 3030)
- `aws_region`: AWS region (optional, will auto-detect if not specified)
- `cloudmap_namespace`: Specific namespace to discover (optional, discovers all if null)

**Environment Variable Overrides:**

- `HOST`: Override the host configuration
- `PORT`: Override the port configuration
- `AWS_REGION`: Override the AWS region
- `CLOUDMAP_NAMESPACE`: Override the namespace filter

### Namespace Filtering

You can filter discovery to a specific Cloud Map namespace in three ways:

#### 1. Config File
```json
{
  "host": "0.0.0.0",
  "port": 3030,
  "aws_region": "us-west-2",
  "cloudmap_namespace": "production"
}
```

#### 2. Environment Variable
```bash
CLOUDMAP_NAMESPACE=production cargo run
```

#### 3. Both (environment variable takes precedence)
```bash
# Config has "staging", but env var overrides to "production"
CLOUDMAP_NAMESPACE=production cargo run
```

If no namespace is specified, the service will discover all namespaces in your AWS account.

### 3. AWS Credentials

Ensure your AWS credentials are configured via one of:

- AWS credentials file (`~/.aws/credentials`)
- Environment variables (`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`)
- IAM roles (if running on EC2)
- AWS profiles (`AWS_PROFILE` environment variable)

**Required IAM Permissions:**

```json
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Action": [
                "servicediscovery:ListNamespaces",
                "servicediscovery:ListServices",
                "servicediscovery:ListInstances"
            ],
            "Resource": "*"
        }
    ]
}
```

### 4. Run the Service

```bash
# Using default configuration (discovers all namespaces)
cargo run

# With environment overrides
AWS_REGION=us-east-1 PORT=8080 cargo run

# Filter to specific namespace
CLOUDMAP_NAMESPACE=production cargo run

# With specific AWS profile and namespace
AWS_PROFILE=production CLOUDMAP_NAMESPACE=production cargo run

# With debug logging
RUST_LOG=debug cargo run
```

### 5. Test the Endpoint

```bash
curl http://localhost:3030/cloudmap_sd
```

## Sample Output

The `/cloudmap_sd` endpoint returns JSON compatible with Prometheus HTTP service discovery:

```json
[
    {
        "targets": [
            "192.168.34.115"
        ],
        "labels": {
            "__meta_cloudmap_namespace_name": "production.local",
            "__meta_cloudmap_service_name": "frontend"
        }
    },
    {
        "targets": [
            "192.168.35.13",
            "192.168.78.132"
        ],
        "labels": {
            "__meta_cloudmap_namespace_name": "production.local",
            "__meta_cloudmap_service_name": "backend"
        }
    }
]
```

## Prometheus Configuration

Configure Prometheus to use this service for HTTP-based service discovery:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'cloudmap-discovery'
    http_sd_configs:
      - url: 'http://localhost:3030/cloudmap_sd'
        refresh_interval: 30s
    relabel_configs:
      - source_labels: [__meta_cloudmap_service_name]
        target_label: service
      - source_labels: [__meta_cloudmap_namespace_name]
        target_label: namespace
```

## Docker Usage

### Building the Docker Image

```bash
# Build production image
make docker-build

# Build development image with hot reload
make docker-build-dev
```

### Running with Docker

```bash
# Run the application in Docker
make docker-run

# Or run directly with docker
docker run --rm -p 3030:3030 \
  -e AWS_REGION=us-west-2 \
  -e AWS_ACCESS_KEY_ID=your-key \
  -e AWS_SECRET_ACCESS_KEY=your-secret \
  aws-cloudmap-prometheus-sd-api:latest
```

### Docker Compose

The project includes a comprehensive `docker-compose.yml` with multiple service profiles:

#### Basic Usage
```bash
# Start the API service
make docker-compose-up

# View logs
make docker-compose-logs

# Stop services
make docker-compose-down
```

#### Development with Hot Reload
```bash
# Start development services with file watching
make docker-compose-up-dev
```

#### Full Monitoring Stack
```bash
# Start API + Prometheus + Grafana
make docker-compose-up-monitoring
```

This will start:
- **API**: http://localhost:3030/cloudmap_sd
- **Prometheus**: http://localhost:9090
- **Grafana**: http://localhost:3000 (admin/admin)

### Environment Variables for Docker

```bash
# AWS Configuration
AWS_REGION=us-west-2
AWS_ACCESS_KEY_ID=your-access-key
AWS_SECRET_ACCESS_KEY=your-secret-key
AWS_SESSION_TOKEN=your-session-token  # Optional
CLOUDMAP_NAMESPACE=production          # Optional

# Server Configuration
HOST=0.0.0.0
PORT=3030
RUST_LOG=info
```

### Docker Image Features

- **Multi-stage build** for minimal production image
- **Non-root user** for security
- **Health checks** included
- **SSL/TLS support** with ca-certificates
- **Optimized layers** for faster builds
- **Development variant** with hot reload support

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test discovery::tests
```

### Building for Production

```bash
# Optimized release build
cargo build --release

# The binary will be available at:
./target/release/aws-cloudmap-prometheus-sd-api
```

### Logging Levels

Set the `RUST_LOG` environment variable to control logging:

- `error`: Only error messages
- `warn`: Warnings and errors
- `info`: Informational messages (default)
- `debug`: Detailed debugging information
- `trace`: Very verbose tracing

Example:
```bash
RUST_LOG=debug cargo run
```

## Architecture

The service is built with a modular architecture:

- **`main.rs`**: Application entry point and server setup
- **`config.rs`**: Configuration management with JSON and environment variable support
- **`discovery.rs`**: AWS Cloud Map service discovery logic
- **`handlers.rs`**: HTTP request handlers for the REST API

## Comparison with Go Version

This Rust implementation provides the same functionality as the original Go version but with key differences:

| Feature | Go Version | Rust Version |
|---------|------------|--------------|
| Output | File-based (`cloudmap_sd.json`) | HTTP API endpoint |
| Runtime | Single execution | Long-running service |
| Configuration | Command-line flags | JSON config + env vars |
| Refresh | Manual/cron-based | On-demand via HTTP requests |
| Integration | Prometheus `file_sd_configs` | Prometheus `http_sd_configs` |

## License

This project is licensed under the Apache-2.0 License - see the [LICENSE](LICENSE) file for details.

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Troubleshooting

### Common Issues

**Port already in use:**
```bash
# Check what's using the port
lsof -i :3030

# Use a different port
PORT=3031 cargo run
```

**AWS credentials not found:**
```bash
# Set AWS profile
export AWS_PROFILE=your-profile

# Or set credentials directly
export AWS_ACCESS_KEY_ID=your-key
export AWS_SECRET_ACCESS_KEY=your-secret
```

**No services discovered:**
- Verify your AWS region is correct
- Check IAM permissions for service discovery
- Ensure services are registered in Cloud Map
- Use debug logging: `RUST_LOG=debug cargo run`
