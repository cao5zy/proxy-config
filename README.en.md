
# micro_proxy

[![Crates.io](https://img.shields.io/crates/v/micro_proxy)](https://crates.io/crates/micro_proxy)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://www.rust-lang.org)](https://www.rust-lang.org)

A tool for managing micro-applications, supporting Docker image building, container management, Nginx reverse proxy configuration, and more.

For detailed instructions on micro-application development, please refer to **[Micro-Application Development Guide](docs/micro-app-development.md)**.

[Home](https://www.craftaidhub.com)

## Table of Contents

This document covers the following topics to help you quickly understand and use micro_proxy:

- [Features](#features) - Core features and advantages
- [Installation](#installation) - How to install micro_proxy
- [Quick Start](#quick-start) - Five-minute getting started guide
- [Commands](#commands) - Detailed description of all available commands
- [Configuration](#configuration) - Configuration file details and best practices
- [SSL Certificate Configuration (Optional)](#ssl-certificate-configuration-optional) - HTTPS certificate setup guide
- [Micro-Application Development](#micro-application-development) - Development standards and requirements
- [Troubleshooting](#troubleshooting) - Common issues and solutions
- [Project Structure](#project-structure) - Source code directory organization
- [Tech Stack](#tech-stack) - Technologies and dependencies used
- [License](#license) - Open source license
- [Contributing](#contributing) - How to participate in the project

---

## Features

- **Auto-discovery of Micro-Applications** - Supports multiple scan directories, automatically discovers micro-applications containing `micro-app.yml` and `Dockerfile`
- **Docker Image Building** - Automatically builds Docker images for micro-applications with environment variable support
- **Container Lifecycle Management** - Start, stop, and clean up containers
- **Nginx Reverse Proxy** - Automatically generates nginx configuration as a unified entry point
- **Docker Compose Integration** - Generates docker-compose.yml files
- **State Management** - Hash-based directory change detection to determine if rebuilds are needed
- **Network Management** - Unified Docker network management, supporting inter-service communication
- **Script Support** - Supports pre-build (setup.sh) and cleanup (clean.sh) scripts
- **Network Address List** - Generates network address lists for connectivity troubleshooting
- **Internal Service Support** - Supports internal services like Redis and MySQL that don't need nginx proxying
- **SSL Certificate Support** - Supports Let's Encrypt certificate issuance with automatic ACME verification (optional)
- **Volumes Mapping Support** - Supports Docker volumes mapping for micro-applications to enable data persistence

## Installation

### Install from crates.io (Recommended)

```bash
cargo install micro_proxy
```

### Build from Source

```bash
# Clone the repository
git clone https://github.com/cao5zy/proxy-config
cd proxy-config

# Build
cargo build --release

# Install
cargo install --path .
```

## Quick Start

### 1. Create Configuration File

Copy the example configuration file and modify as needed:

```bash
cp proxy-config.yml.example proxy-config.yml
```

### 2. Prepare Micro-Applications

Create a `micro-app.yml` configuration file in each micro-application directory:

```bash
cp micro-app.yml.example ./micro-apps/my-app/micro-app.yml
```

### 3. Start Micro-Applications

```bash
# Start all micro-applications
micro_proxy start

# Force rebuild all images
micro_proxy start --force-rebuild

# Show verbose logs
micro_proxy start -v
```

### 4. Access Applications

All applications are accessed through the Nginx unified entry point, with the default port being 80 (configurable via the `nginx_host_port` field in `proxy-config.yml`):

```bash
# Access the main application
curl http://localhost/

# Access the API service
curl http://localhost/api
```

## Commands

### start - Start Micro-Applications

```bash
micro_proxy start [options]
```

Options:
- `-c, --config <path>`: Specify configuration file path (default: ./proxy-config.yml)
- `--force-rebuild`: Force rebuild all images

### stop - Stop Micro-Applications

```bash
micro_proxy stop [options]
```

Options:
- `-c, --config <path>`: Specify configuration file path (default: ./proxy-config.yml)

### clean - Clean Up Micro-Applications

```bash
micro_proxy clean [options]
```

Options:
- `-c, --config <path>`: Specify configuration file path (default: ./proxy-config.yml)
- `--force`: Force cleanup without confirmation
- `--network`: Also clean up Docker network

### status - View Status

```bash
micro_proxy status [options]
```

Options:
- `-c, --config <path>`: Specify configuration file path (default: ./proxy-config.yml)

### network - View Network Addresses

```bash
micro_proxy network [options]
```

Options:
- `-c, --config <path>`: Specify configuration file path (default: ./proxy-config.yml)
- `-o, --output <path>`: Specify output file path (overrides configuration file setting)

## Configuration

### Main Configuration File (proxy-config.yml)

```yaml
# Scan directory list (for discovering micro-app.yml)
scan_dirs:
  - "./micro-apps"

# Dynamic apps configuration storage path
# This file is auto-generated by micro_proxy, do not edit manually
apps_config_path: "./apps-config.yml"

# Nginx configuration file output path
nginx_config_path: "./nginx.conf"

# Docker Compose configuration file output path
compose_config_path: "./docker-compose.yml"

# State file path
state_file_path: "./proxy-config.state"

# Network address list output path
network_list_path: "./network-addresses.txt"

# Docker network name
network_name: "proxy-network"

# Nginx host port (unified entry point)
nginx_host_port: 80

# Web root directory (optional)
web_root: "/var/www/html"

# Certificate directory (optional)
cert_dir: "/etc/nginx/certs"

# Domain name (optional)
domain: "example.com"
```

### Micro-Application Configuration File (micro-app.yml)

Each micro-application directory must contain a `micro-app.yml` file to configure the application's properties:

```yaml
# Access paths (required for static/api types)
routes: ["/", "/api"]

# Docker container name (required, globally unique)
container_name: "my-container"

# Container internal port (required)
container_port: 80

# Application type (required): static, api, internal
app_type: "static"

# Application description (optional)
description: "Application description"

# Docker volumes mapping (optional)
docker_volumes:
  - "./data:/app/data"           # Read-write mount
  - "./config:/app/config:ro"    # Read-only mount

# Extra nginx configuration (optional, only effective for static and api types)
nginx_extra_config: |
  add_header 'X-Custom-Header' 'value';

# Proxy timeout settings (optional, only effective for api type, unit: seconds, default: 60)
proxy_connect_timeout: 60
proxy_read_timeout: 60
proxy_send_timeout: 60
```

**Detailed configuration instructions** are available in **[Micro-Application Development Guide](docs/micro-app-development.md)**.

### SSL Certificate Configuration

> **Full Guide**: For detailed information on SSL certificate configuration methods, how it works, and FAQ, please refer to **[SSL Configuration Complete Guide](docs/ssl-configuration.md)**.

micro_proxy supports Let's Encrypt certificate issuance, automatically verifying domain ownership through the ACME protocol. Here is a brief overview:

#### Three Required Configuration Items

| Configuration | Purpose | Default Value |
|---------------|---------|---------------|
| `web_root` | Directory for storing ACME verification files; Let's Encrypt verifies domain ownership through this directory | `/var/www/html` |
| `cert_dir` | Directory for storing SSL certificates and private keys; automatically mounted to the Nginx container | `/etc/nginx/certs` |
| `domain` | Domain name, used to derive certificate file paths and Nginx configuration | None (optional) |

#### Quick Configuration Steps

```bash
# 1. Configure the following three items in proxy-config.yml
web_root: "/var/www/html"
cert_dir: "/etc/nginx/certs"
domain: "your-domain.com"

# 2. Ensure directories exist with write permissions
sudo mkdir -p /var/www/html
sudo mkdir -p /etc/nginx/certs

# 3. Issue certificate using acme.sh
acme.sh --issue -d your-domain.com --webroot /var/www/html

# 4. Deploy certificate
acme.sh --install-cert -d your-domain.com \
  --key-file /etc/nginx/certs/your-domain.com.key \
  --fullchain-file /etc/nginx/certs/your-domain.com.cer

# 5. Start services
micro_proxy start
```

> **See More**: [SSL Configuration Complete Guide](docs/ssl-configuration.md) includes detailed FAQ, error troubleshooting, and best practices.

### Port Configuration

micro_proxy uses Docker port mapping to map host ports to container internal ports.

| Configuration | Purpose | Example | Description |
|---------------|---------|---------|-------------|
| `nginx_host_port` | Host port | 80 | The port users access, mapped to the container internal port via Docker |
| `listen` in `nginx.conf` | Container internal port | 80 | Fixed value, auto-generated by micro_proxy, no manual modification needed |

**Port Mapping Notes**:
- HTTP: Fixed at 80
- HTTPS: Fixed at 443
- If the host port is already in use, modify `nginx_host_port`

### Scan Directory Notes

The `scan_dirs` configuration specifies the directory list for scanning micro-applications:

- Only scans first-level directories, does not scan recursively
- Only directories containing both `micro-app.yml` and `Dockerfile` are recognized as micro-applications
- The directory name is used as the default micro-application name (`app.name`)
- All micro-application `container_name` values must be globally unique

## Micro-Application Development

For detailed instructions on micro-application development, please refer to **[Micro-Application Development Guide](docs/micro-app-development.md)**.

### Application Type Overview

micro_proxy supports three application types:

| Type | Description | Access Method |
|------|-------------|---------------|
| **Static** | Static application (frontend pages) | Served externally via Nginx reverse proxy |
| **API** | API service (backend endpoints) | Served externally via Nginx reverse proxy |
| **Internal** | Internal service (databases, etc.) | Only for internal communication between micro-applications |

### Standard Micro-Application Directory Structure

```
micro-apps/
└── my-app/                    # Micro-application directory
    ├── micro-app.yml          # Micro-application configuration (required)
    ├── Dockerfile             # Docker build file (required)
    ├── nginx.conf             # Nginx configuration (required for SPA)
    ├── setup.sh               # Pre-build script (optional)
    ├── clean.sh               # Cleanup script (optional)
    ├── .env                   # Environment variables (optional)
    └── src/                   # Source code directory
```

## Troubleshooting

### View Logs

```bash
# Show verbose logs
micro_proxy start -v

# View container logs
docker logs <container-name>

# View nginx logs
docker logs proxy-nginx
```

### View Network Addresses

```bash
# Generate and view network address list
micro_proxy network

# View the generated file
cat network-addresses.txt
```

### Check Container Status

```bash
# View all container statuses
micro_proxy status

# Use docker command to check
docker ps -a
```

### Port Conflict Issues

```bash
# Check port usage
sudo lsof -i :80
sudo lsof -i :8080

# Modify nginx_host_port in proxy-config.yml
nginx_host_port: 8080  # Change to another available port
```

### Volumes Mounting Issues

```bash
# Check if host path exists
ls -la ./data

# Check mount point inside the container
docker exec <container-name> ls -la /app/data

# View container details
docker inspect <container-name> | grep -A 10 Mounts
```

### SSL Certificate Issues

```bash
# Check if certificate files exist
ls -la /etc/nginx/certs/

# Validate nginx configuration
docker exec proxy-nginx nginx -t

# View nginx error logs
docker logs proxy-nginx | grep -i ssl

# Manually test HTTPS connection
curl -k https://your-domain.com
```

> **More Help**: For SSL configuration troubleshooting, see [SSL Configuration Complete Guide](docs/ssl-configuration.md#troubleshooting)

### Micro-Application Configuration Issues

```bash
# Check if micro-app.yml exists
ls -la ./micro-apps/my-app/micro-app.yml

# Check if Dockerfile exists
ls -la ./micro-apps/my-app/Dockerfile

# Validate micro-app.yml format
cat ./micro-apps/my-app/micro-app.yml

# Check for duplicate container_name
grep -r "container_name:" ./micro-apps/*/micro-app.yml
```

## Project Structure

```
proxy-config/
├── docs/
│   ├── ssl-configuration.md     # SSL Configuration Complete Guide
│   ├── micro-app-development.md # Micro-Application Development Guide
│   └── ...
├── src/
│   ├── main.rs          # Main entry point
│   ├── lib.rs           # Library entry point
│   ├── cli.rs           # Command-line interface
│   ├── config.rs        # Configuration management
│   ├── discovery.rs     # Application discovery
│   ├── micro_app_config.rs  # Micro-application configuration parsing
│   └── ...
├── Cargo.toml           # Project configuration
├── proxy-config.yml.example  # Configuration file example
├── micro-app.yml.example     # Micro-application configuration example
└── README.md            # Project documentation
```

## Tech Stack

- **Rust** - Primary programming language
- **Docker** - Containerization
- **Nginx** - Reverse proxy
- **Docker Compose** - Container orchestration

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

If you encounter any issues while using this tool, feel free to submit an Issue.

If you'd like to follow the latest project updates or read related technical articles, feel free to follow my WeChat Official Account:
![WeChat QR Code](./assets/wechat-id.png)]
