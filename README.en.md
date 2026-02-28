# micro_proxy

[![Crates.io](https://img.shields.io/crates/v/micro_proxy)](https://crates.io/crates/micro_proxy)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)

A tool for managing micro-apps, supporting Docker image building, container management, Nginx reverse proxy configuration, and more.
[Home](https://www.craftaidhub.com)

## Features

- 🔍 **Automatic Micro-app Discovery** - Supports multiple scan directories to automatically discover micro-apps containing Dockerfiles
- 🐳 **Docker Image Building** - Automatically builds Docker images for micro-apps with environment variable passing support
- 🔄 **Container Lifecycle Management** - Start, stop, and clean up containers
- 🌐 **Nginx Reverse Proxy** - Automatically generates Nginx configuration as a unified entry point
- 📦 **Docker Compose Integration** - Generates docker-compose.yml files
- 📊 **State Management** - Determines whether rebuild is needed based on directory hash
- 🌍 **Network Management** - Unified Docker network management with inter-micro-app communication support
- 📝 **Script Support** - Supports pre-build (setup.sh) and cleanup (clean.sh) scripts
- 📋 **Network Address List** - Generates network address list for connectivity troubleshooting
- 🔒 **Internal Service Support** - Supports internal services like Redis, MySQL that don't require Nginx proxy
- 🔐 **SSL Certificate Support** - Supports Let's Encrypt certificate requests with automatic ACME validation (optional)

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

### 2. Start Micro-apps

```bash
# Start all micro-apps
micro_proxy start

# Force rebuild all images
micro_proxy start --force-rebuild

# Show detailed logs
micro_proxy start -v
```

### 3. Access Applications

All applications are accessed through the Nginx unified entry point, defaulting to port 80 (configurable via the `nginx_host_port` field in `micro_proxy.yml`):

```bash
# Access main application
curl http://localhost/

# Access API service
curl http://localhost/api
```

## Command Reference

### start - Start Micro-apps

```bash
micro_proxy start [options]
```

Options:
- `-c, --config <path>`: Specify configuration file path (default: ./micro_proxy.yml)
- `--force-rebuild`: Force rebuild all images

### stop - Stop Micro-apps

```bash
micro_proxy stop [options]
```

Options:
- `-c, --config <path>`: Specify configuration file path (default: ./proxy-config.yml)

### clean - Clean Micro-apps

```bash
micro_proxy clean [options]
```

Options:
- `-c, --config <path>`: Specify configuration file path (default: ./proxy-config.yml)
- `--force`: Force cleanup without confirmation
- `--network`: Also clean up Docker network

### status - Check Status

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
# Scan directories list (for Static and Api types)
scan_dirs:
  - "./micro-apps"

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
# Used for storing ACME challenge files, supports Let's Encrypt certificate requests
# Default value: "/var/www/html"
# Can be omitted if HTTPS certificates are not needed
# web_root: "/var/www/html"

# Certificate directory (optional)
# Host directory for storing SSL certificates
# Default value: "/etc/nginx/certs"
# Can be omitted if HTTPS certificates are not needed
# cert_dir: "/etc/nginx/certs"

# Domain name (optional)
# Used for HTTPS configuration. If configured and certificate files exist, Nginx will enable HTTPS
# Certificate file naming: {cert_dir}/{domain}.cer (or .crt)
# Key file naming: {cert_dir}/{domain}.key
# Example:
# domain: "example.com"

# Reverse proxy configuration
apps:
  # Static and Api types: name must match the discovered micro-app folder name
  - name: "app-name"
    routes: ["/", "/api"]          # Access paths
    container_name: "container"    # Container name
    container_port: 80             # Container internal port
    app_type: "static"             # Application type: static, api, or internal
    description: "Application description"  # Optional
    nginx_extra_config: |          # Optional: additional nginx configuration (static and api only)
      add_header 'X-Custom-Header' 'value';

  # Internal type: no nginx reverse proxy needed, only for internal communication
  - name: "redis"
    routes: []                     # Internal type routes should be empty
    container_name: "redis-container"
    container_port: 6379
    app_type: "internal"
    description: "Redis cache service"
    path: "./services/redis"       # Must be configured, pointing to service folder path
```

#### Scan Directory Guidelines

The `scan_dirs` configuration specifies directories to scan for micro-apps, with the following important rules:

**1. Only First-level Directories**
- The system only reads **first-level subdirectories** of specified directories, no recursive scanning
- Each first-level subdirectory is treated as an independent micro-app
- Example:
  ```
  ./micro-apps/
  ├── app1/          # Will be scanned
  ├── app2/          # Will be scanned
  └── nested/
      └── app3/      # Will NOT be scanned (second-level directory)
  ```

**2. Multiple Scan Directories Supported**
- If micro-apps are distributed across multiple directories, specify all in `scan_dirs`
- Example:
  ```yaml
  scan_dirs:
    - "./frontend-apps"
    - "./backend-apps"
    - "./services"
  ```

**3. Directory Name Uniqueness Requirement**
- **No duplicate directory names** allowed across all scan directories
- If duplicate names are found, the system will error and exit
- Example (invalid configuration):
  ```
  ./frontend-apps/
  └── common/        # Conflicts with backend-apps/common
  
  ./backend-apps/
  └── common/        # Conflicts with frontend-apps/common
  ```
  This will cause an error: `Duplicate micro-app name found: common`

**4. Directory Naming Recommendations**
- Use meaningful, unique directory names
- Directory name becomes the micro-app's default name
- Avoid special characters and spaces

## SSL Certificate Configuration (Optional)

> **Important Note: SSL certificate configuration is completely optional!**  
> If HTTPS certificates are not configured, micro_proxy will still work normally, and HTTP (port 80) reverse proxy functionality remains unaffected.

micro_proxy supports Let's Encrypt certificate requests through ACME protocol for automatic domain ownership verification.

### Configuration Overview

1. **Decide if HTTPS is needed**: If not, completely ignore SSL-related configuration
2. **Configure `proxy-config.yml`**: Set `web_root`, `cert_dir`, and `domain` fields
3. **Request SSL certificate**: Use ACME.sh tool to request certificates
4. **Verify configuration**: Ensure certificate files exist and Nginx can load them correctly

### Detailed Configuration Guide

For complete SSL configuration and certificate request guide, please refer to:

- [ACME.sh Installation and Configuration Guide](docs/acme-installation.md)
- [SSL Certificate Request Guide](docs/certificate-application.md)

### ACME Validation Mechanism

micro_proxy automatically adds ACME validation location to generated Nginx configuration:

```nginx
location /.well-known/acme-challenge/ {
    root /var/www/html;
    default_type "text/plain";
}
```

**Important Notes:**
- ACME location only matches `/.well-known/acme-challenge/` path
- Does not affect other normal reverse proxy requests
- HTTP reverse proxy works normally even without certificate configuration

### Docker Compose Configuration

Ensure Docker Compose configuration correctly mounts certificate directories:

```yaml
services:
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - /var/www/html:/var/www/html:ro
      - /etc/nginx/certs:/etc/nginx/certs:ro
    networks:
      - proxy-network
```

### Important Notes

1. **Domain Resolution**: Ensure domain is properly resolved to server IP
2. **Firewall**: Ensure ports 80 and 443 are open
3. **web_root Mounting**: Ensure Nginx container can access web_root directory
4. **cert_dir Mounting**: Ensure Nginx container can access cert_dir directory
5. **Auto-renewal**: acme.sh enables auto-renewal by default, no extra configuration needed
6. **Optional Configuration**: If HTTPS is not needed, completely ignore `web_root`, `cert_dir`, and `domain` fields

## Micro-app Development Guide

### What is a Micro-app?

A **micro-app** is an application organization approach inspired by microservice architecture. Each micro-app is an independent, deployable software unit packaged using Docker containerization technology. Multiple micro-apps can be combined to form a more complex system while maintaining their independence and maintainability.

**Core Characteristics:**
- **Independence**: Each micro-app has its own codebase, dependencies, and configuration
- **Composability**: Multiple micro-apps can work together to build complex systems
- **Sustainability**: Supports independent development, testing, deployment, and scaling
- **Containerization**: Uses Docker for standardized deployment and runtime environments

### Micro-app Directory Structure

Each micro-app must be an independent folder, with the folder name serving as the micro-app's name.

#### Frontend/Static Apps and API Services

For micro-apps that need to be exposed externally through Nginx:

```
micro-apps/
├── main-app/              # Folder name becomes micro-app name
│   ├── Dockerfile         # Must be in project root
│   ├── .env               # Environment variables file (optional)
│   ├── setup.sh           # Optional: pre-build script
│   ├── clean.sh           # Optional: cleanup script
│   └── ...                # Other application files
├── resume-app/
│   ├── Dockerfile
│   ├── .env
│   └── ...
└── api-service/
    ├── Dockerfile
    ├── .env
    └── ...
```

#### Internal Services

For services like Redis, MySQL that are only used for internal communication:

```
services/
├── redis/                 # Service folder
│   ├── Dockerfile         # Must be in project root
│   ├── .env               # Environment variables file (optional)
│   ├── setup.sh           # Optional: pre-build script
│   ├── clean.sh           # Optional: cleanup script
│   └── ...                # Other service files
└── mysql/
    ├── Dockerfile
    ├── .env
    └── ...
```

### Application Types

micro_proxy supports three application types that determine how micro-apps are accessed and configured:

#### 1. Static (Static Applications)
- **Use Case**: Frontend applications, static websites
- **Characteristics**: Enables browser caching, suitable for static resources
- **Access Method**: Exposed externally through Nginx reverse proxy
- **Configuration Example**:
  ```yaml
  - name: "frontend"
    routes: ["/app"]
    app_type: "static"
    container_port: 80
  ```

#### 2. API (API Services)
- **Use Case**: Backend API services, microservices
- **Characteristics**: Disables caching, preserves full request path
- **Access Method**: Exposed externally through Nginx reverse proxy
- **Configuration Example**:
  ```yaml
  - name: "backend"
    routes: ["/api"]
    app_type: "api"
    container_port: 8080
  ```

#### 3. Internal (Internal Services)
- **Use Case**: Redis, MySQL, MongoDB and other database services
- **Characteristics**: Not exposed through Nginx, only for internal micro-app communication
- **Access Method**: Other micro-apps access directly by container name
- **Configuration Example**:
  ```yaml
  - name: "redis"
    app_type: "internal"
    container_port: 6379
    path: "./services/redis"
  ```

### Development Workflow

#### 1. Dockerfile Requirements
- Must be placed in the micro-app project root
- Recommended to use `EXPOSE` instruction to declare ports
- Example:
  ```dockerfile
  FROM nginx:alpine
  EXPOSE 80
  COPY . /usr/share/nginx/html
  ```

#### 2. Environment Variable Configuration
- Define build-time environment variables in `.env` file
- These variables are passed to Docker during build
- Example:
  ```env
  APP_PORT=80
  ENV=production
  ```

#### 3. Automation Scripts
- **setup.sh**: Executed before image building, for environment preparation
- **clean.sh**: Executed during cleanup, for removing build artifacts
- Scripts must be placed in the micro-app project root

### Networking and Communication

All micro-apps run in the same Docker network, supporting the following communication methods:

#### External Services
- Static and API type micro-apps are exposed externally through Nginx unified entry point
- Access URL: `http://<host>:<port>/<configured-route>`

#### Internal Communication
- All micro-apps can communicate with each other using container names
- Examples:
  ```bash
  # frontend accessing backend
  curl http://backend:8080/api
  
  # backend accessing redis
  redis-cli -h redis -p 6379
  ```

### Reverse Proxy Configuration

micro_proxy automatically generates appropriate Nginx configuration based on application type:

#### Static Type Path Handling
- **Root Path** (`/`): Directly forwards requests
- **Sub-path** (`/app`): Automatically removes path prefix
  - Access `/app/index.html` → backend receives `/index.html`

#### API Type Path Handling
- **Preserves Full Path**: Does not modify request URI
  - Access `/api/v1/users` → backend receives `/api/v1/users`

### Custom Configuration

Additional Nginx configuration can be added for Static and API type micro-apps:

```yaml
- name: "main-app"
  routes: ["/"]
  nginx_extra_config: |
    add_header 'X-Custom-Header' 'value';
    location /api {
      proxy_pass http://backend:3000;
    }
```

## Troubleshooting

### Viewing Logs

```bash
# Show detailed logs
micro_proxy start -v

# View container logs
docker logs <container-name>

# View nginx logs
docker logs proxy-nginx
```

### Viewing Network Addresses

```bash
# Generate and view network address list
micro_proxy network

# View generated file
cat network-addresses.txt
```

### Checking Container Status

```bash
# View all container status
micro_proxy status

# Use docker command to view
docker ps -a
```

## Project Structure

```
proxy-config/
├── docs/
│   ├── acme-installation.md     # ACME.sh Installation Guide
│   └── certificate-application.md  # Certificate Request Guide
├── src/
│   ├── main.rs          # Main entry point
│   ├── lib.rs           # Library entry point
│   ├── cli.rs           # Command line interface
│   ├── config.rs        # Configuration management
│   ├── discovery.rs     # Application discovery
│   ├── builder.rs       # Image building
│   ├── container.rs     # Container management
│   ├── nginx.rs         # Nginx configuration generation
│   ├── compose.rs       # Docker Compose generation
│   ├── state.rs         # State management
│   ├── script.rs        # Script execution
│   ├── network.rs       # Network management
│   ├── dockerfile.rs    # Dockerfile parsing
│   └── error.rs         # Error definitions
├── Cargo.toml           # Project configuration
├── proxy-config.yml.example  # Configuration file example
└── README.md            # Project documentation
```

## Technology Stack

- **Rust** - Primary programming language
- **Docker** - Containerization
- **Nginx** - Reverse proxy
- **Docker Compose** - Container orchestration

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details

## Contributing

Issues and Pull Requests are welcome!
