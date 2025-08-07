#!/bin/bash
# Health check script for AWS Cloud Map Prometheus Service Discovery API
#
# This script checks if the service is healthy by making a request to the
# /cloudmap_sd endpoint and verifying it returns valid JSON.

set -e

# Configuration
HOST=${HOST:-localhost}
PORT=${PORT:-3030}
TIMEOUT=${TIMEOUT:-10}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to log messages
log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] $1${NC}"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING: $1${NC}"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $1${NC}"
}

# Main health check function
health_check() {
    local url="http://${HOST}:${PORT}/cloudmap_sd"
    
    log "Performing health check on ${url}"
    
    # Check if the service responds
    if ! curl -f -s --max-time "${TIMEOUT}" "${url}" > /dev/null 2>&1; then
        error "Service is not responding at ${url}"
        return 1
    fi
    
    # Check if the response is valid JSON
    local response
    response=$(curl -f -s --max-time "${TIMEOUT}" "${url}" 2>/dev/null)
    
    if ! echo "${response}" | jq . > /dev/null 2>&1; then
        error "Service returned invalid JSON response"
        return 1
    fi
    
    # Check if the response is an array (expected format)
    if ! echo "${response}" | jq -e 'type == "array"' > /dev/null 2>&1; then
        warn "Service returned JSON but not in expected array format"
        return 1
    fi
    
    log "Health check passed - service is healthy"
    return 0
}

# Run health check
if health_check; then
    exit 0
else
    exit 1
fi
