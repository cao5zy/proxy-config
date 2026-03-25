
# micro_proxy

[![Crates.io](https://img.shields.io/crates/v/micro_proxy)](https://crates.io/crates/micro_proxy)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://www.rust-lang.org)](https://www.rust-lang.org)

A tool for managing micro-apps, supporting Docker image building, container management, Nginx reverse proxy configuration and more.

For detailed information on micro-app development, please refer to **[Micro-app Development Guide](docs/micro-app-development.md)**.

[Home](https://www.craftaidhub.com)

## 📑 Documentation Index

This documentation contains the following content to help you quickly get started with using micro_proxy:

- [Features](#features) - Understand the core functions and advantages of the tool
- [Installation](#installation) - How to install micro_proxy
- [Quick Start](#quick-start) - A five-minute getting started guide
- [Command Reference](#command-reference) - Detailed description of all available commands
- [Configuration Guide](#configuration-guide) - Configuration file details and best practices
- [SSL Certificate Configuration (Optional)](#ssl-certificate-configuration-optional) - HTTPS certificate setup guide
- [Micro-app Development](#micro-app-development) - Micro-app development specifications and requirements
- [Troubleshooting](#troubleshooting) - Common issues and solutions
- [Project Structure](#project-structure) - Source code directory organization
- [Technology Stack](#technology-stack) - Technologies and dependencies used
- [License](#license) - Open source license
- [Contributing](#contributing) - Ways to contribute to the project

---

## Features

- 🔍 **Automatic Micro-app Discovery** - Supports multiple scan directories to automatically discover micro-apps containing Dockerfiles
- 🐳 **Docker Image Building** - Automatically builds Docker images for micro-apps with environment variable passing support
- 🔄 **Container Lifecycle Management** - Start, stop, and clean up containers
- 🌐 **Nginx Reverse Proxy** - Automatically generates nginx configuration as a unified entry point
- 📦 **Docker Compose Integration** - Generates docker-compose.yml files
- 📊 **State Management** - Determines whether rebuild is needed based on directory hash
- 🌍 **Network Management** - Unified Docker network management with inter-micro-app communication support
- 📝 **Script Support** - Supports pre-build (setup.sh) and cleanup (clean.sh) scripts
- 📋 **Network Address List** - Generates network address list for connectivity troubleshooting
- 🔒 **Internal Service Support** - Supports internal services like Redis, MySQL that don't require nginx proxy
- 🔐 **SSL Certificate Support** - Supports Let's Encrypt certificate requests with automatic ACME validation (optional)
- 💾 **Volumes Mapping Support** - Supports configuring Docker volumes mapping for micro-apps to achieve data persistence

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

All applications are accessed through the Nginx unified entry point, defaulting to port 80 (configurable via the `nginx_host_port` field in `proxy-config.yml`):

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

## Configuration Guide

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
# Note: This is the host port, mapped to container's internal port 80 via Docker port mapping
# For example: when set to 8080, accessing http://localhost:8080 maps to container's internal port 80
nginx_host_port: 80

# Web root directory (optional)
# Used for storing ACME challenge files, supports Let's Encrypt certificate requests
# Default value: "/var/www/html"
web_root: "/var/www/html"

# Certificate directory (optional)
# Host directory for storing SSL certificates
# Default value: "/etc/nginx/certs"
cert_dir: "/etc/nginx/certs"

# Domain name (optional)
# Used for HTTPS configuration. If configured and certificate files exist, Nginx will enable HTTPS
# Certificate file naming: {cert_dir}/{domain}.cer (or .crt)
# Key file naming: {cert_dir}/{domain}.key
domain: "example.com"

# Reverse proxy configuration
apps:
  # Static and Api types: name must match the discovered micro-app folder name
  - name: "app-name"
    routes: ["/", "/api"]          # Access paths
    container_name: "container"    # Container name
    container_port: 80             # Container internal port
    app_type: "static"             # Application type: static, api, or internal
    description: "Application description"  # Optional
    docker_volumes:                # Docker volumes mapping (optional)
      - "./data:/app/data"         # Read-write mount
      - "./config:/app/config:ro"  # Read-only mount
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
    docker_volumes:                # Docker volumes mapping (optional)
      - "./redis-data:/data"       # Persist Redis data
```

### SSL Certificate Configuration Guide

> ℹ️ **Complete Guide**: For detailed information on SSL certificate configuration methods, working principles and FAQ, please refer to **[SSL Configuration Complete Guide](docs/ssl-configuration.md)**.

micro_proxy supports Let's Encrypt certificate requests through ACME protocol for automatic domain ownership verification. Here's a brief overview:

#### Three Required Configuration Items

| Configuration Item | Purpose | Default Value |
|--------|------|--------|
| `web_root` | Directory for storing ACME challenge files, Let's Encrypt verifies domain ownership through this directory | `/var/www/html` |
| `cert_dir` | Directory for storing SSL certificates and private keys, will be auto-mounted to Nginx container | `/etc/nginx/certs` |
| `domain` | Domain name, used to derive certificate file path and Nginx configuration | None (optional) |

#### Workflow

1. **Apply for Certificate**: Use acme.sh to request certificate from Let's Encrypt
2. **Place Certificate**: Certificate is saved to `cert_dir` directory
3. **Mount Directory**: `docker-compose.yml` automatically mounts `cert_dir` to Nginx container
4. **Enable HTTPS**: After detecting certificates, HTTPS configuration is auto-generated

#### Quick Configuration Steps

```bash
# 1. Configure the following three items in proxy-config.yml
web_root: "/var/www/html"
cert_dir: "/etc/nginx/certs"
domain: "your-domain.com"

# 2. Ensure directories exist and have write permissions
sudo mkdir -p /var/www/html
sudo mkdir -p /etc/nginx/certs

# 3. Use acme.sh to request certificate
acme.sh --issue -d your-domain.com --webroot /var/www/html

# 4. Deploy certificate
acme.sh --install-cert -d your-domain.com \
  --key-file /etc/nginx/certs/your-domain.com.key \
  --fullchain-file /etc/nginx/certs/your-domain.com.cer

# 5. Start services
micro_proxy start
```

#### Mounting in docker-compose.yml

After enabling SSL, the generated `docker-compose.yml` will include:

```yaml
services:
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"  # HTTPS port
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - /var/www/html:/var/www/html:ro      # web_root mount
      - /etc/nginx/certs:/etc/nginx/certs:ro  # cert_dir mount
```

#### ⚠️ Common Questions

- **Will web_root conflict with my applications?**  
  No. The ACME location only matches the `/.well-known/acme-challenge/` path, which doesn't affect other routes.

- **What is the purpose of cert_dir?**  
  Ensures certificates are persistently stored on the host, won't be affected by container deletion, and can be accessed by Nginx container.

- **Besides deriving file paths, what else does domain do?**  
  It's also used for Nginx's `server_name` configuration and serves as the switch to automatically enable HTTPS.

> 🔗 **Learn More**: [SSL Configuration Complete Guide](docs/ssl-configuration.md) includes detailed FAQ, troubleshooting and best practices.

### Docker Volumes Configuration Guide

The `docker_volumes` field is used to configure Docker container volume mounts for data persistence and file sharing.

#### Configuration Format

```yaml
docker_volumes:
  - "host_path:container_path"           # Read-write mount (default)
  - "host_path:container_path:ro"       # Read-only mount
  - "host_path:container_path:rw"       # Read-write mount (explicit)
```

#### Use Cases

1. **Data Persistence**: Mount container data directories to host to avoid data loss after container deletion
   ```yaml
   docker_volumes:
     - "./redis-data:/data"        # Redis data persistence
     - "./mysql-data:/var/lib/mysql"  # MySQL data persistence
   ```

2. **Configuration File Sharing**: Mount host configuration files into container for easy configuration modification
   ```yaml
   docker_volumes:
     - "./config:/app/config:ro"   # Read-only mount for configuration files
   ```

3. **Log Output**: Mount container log directories to host for easy log viewing and analysis
   ```yaml
   docker_volumes:
     - "./logs:/app/logs"          # Output logs to host
   ```

4. **File Uploads**: Store user-uploaded files on host
   ```yaml
   docker_volumes:
     - "./uploads:/app/uploads"    # Store user uploaded files
   ```

#### Important Notes

- **Path Format**: Supports both relative and absolute paths
  - Relative path: `./data:/app/data` (relative to docker-compose.yml directory)
  - Absolute path: `/var/data:/app/data`

- **Permission Control**:
  - `ro`: Read-only mount, cannot be modified inside container
  - `rw`: Read-write mount (default), can be modified inside container

- **Directory Creation**: Docker will automatically create host path if it doesn't exist

- **Path Separator**: Recommend using forward slash `/`, even on Windows systems

#### Reverse Proxy Configuration Example

```yaml
apps:
  # Static website application
  - name: "main-app"
    routes: ["/"]
    container_name: "main-container"
    container_port: 80
    app_type: "static"
    docker_volumes:
      - "./static-data:/usr/share/nginx/html/data"
      - "./static-config:/etc/nginx/conf.d:ro"

  # API service
  - name: "api-service"
    routes: ["/api"]
    container_name: "api-container"
    container_port: 3000
    app_type: "api"
    docker_volumes:
      - "./api-logs:/app/logs"
      - "./api-uploads:/app/uploads"

  # Redis internal service
  - name: "redis"
    routes: []
    container_name: "redis-container"
    container_port: 6379
    app_type: "internal"
    path: "./services/redis"
    docker_volumes:
      - "./redis-data:/data"
```

### Port Configuration Guide

micro_proxy uses Docker port mapping mechanism to map host ports to container internal ports. Understanding this mechanism is important for correct configuration.

#### Port Mapping Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Host Machine                          │
│                                                              │
│  User Access: http://localhost:8080                           │
│         │                                                     │
│         ▼                                                     │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  Docker Port Mapping (8080:80)                     │    │
│  │  nginx_host_port: 8080  ──mapped──►  Container: 80  │    │
│  └─────────────────────────────────────────────────────┘    │
│         │                                                     │
│         ▼                                                     │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              Nginx Container                          │    │
│  │                                                      │    │
│  │  nginx.conf listen directive:                        │    │
│  │    - HTTP:  listen 80;                               │    │
│  │    - HTTPS: listen 443 ssl;                          │    │
│  │                                                      │    │
│  │  Note: nginx.conf ports are container internal,     │    │
│  │       fixed at 80 (HTTP) and 443 (HTTPS)            │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

#### Configuration Item Description

| Configuration Item | Purpose | Example Value | Description |
|-------------------|---------|---------------|-------------|
| `nginx_host_port` | Host port | 80 | Port users access, mapped to container internal via Docker port mapping |
| `nginx.conf` `listen` | Container internal port | 80 | Fixed value, automatically generated by micro_proxy, no manual modification needed |

#### Port Mapping Tips
1. **nginx_host_port Only Affects Host Port**
   - Modifying `nginx_host_port` only changes Docker container port on the host
   - Does not affect `nginx.conf` `listen` directive

2. **nginx.conf Ports Are Fixed**
   - HTTP: Fixed at 80
   - HTTPS: Fixed at 443
   - These ports are automatically generated by micro_proxy, no manual modification needed

3. **Port Conflict Handling**
   - If host port is already in use, modify `nginx_host_port`
   - For example: if port 80 is occupied, set to 8080

4. **HTTPS Port**
   - When HTTPS is enabled, port 443 is automatically added to port mapping
   - Port 443 is the standard HTTPS port, usually no modification needed

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

### 📁 Micro-app Directory Structure Guide

The `scan_dirs` directories defined in the configuration file will contain multiple micro-app subdirectories. **Each micro-app subdirectory must follow specific file structure specifications** to be correctly recognized and built.

⚠️ **Important Notice**: For specific file structure requirements, key file naming conventions and build process details, please refer to the dedicated technical documentation:

👉 **[View Full Specifications → Micro-app Development Guide](docs/micro-app-development.md)**

**Core File Structure Overview:**

| File/Directory | Required | Description |
|-----------|----------|------|
| `Dockerfile` | ✅ Required | Docker image build file, located in micro-app root directory |
| `nginx.conf` | ⚠️ Conditional | SPA deployment required, custom Nginx configuration |
| `setup.sh` | ❌ Optional | Pre-build execution script, for environment preparation |
| `clean.sh` | ❌ Optional | Cleanup script, for removing build artifacts |
| `.env` | ❌ Optional | Environment variables file |
| `dist/` or `build/` | ⚠️ Conditional | Frontend project build output directory |

**Standard Micro-app Directory Structure Example:**

```
micro-apps/
└── my-app/                    # Micro-app directory
    ├── Dockerfile             # Docker build file (Required)
    ├── nginx.conf             # Nginx config (SPA apps required)
    ├── setup.sh               # Pre-build script (Optional)
    ├── clean.sh               # Cleanup script (Optional)
    ├── .env                   # Environment variables (Optional)
    ├── package.json           # Node.js project config
    ├── src/                   # Source code directory
    └── dist/                  # Build output directory
```

> 💡 **Tip**: Different types of micro-apps (Static, Api, Internal) have different file requirements and configuration methods. For detailed specifications and best practices, refer to [Micro-app Development Guide](docs/micro-app-development.md).

## Micro-app Development

For detailed information on micro-app development, please refer to **[Micro-app Development Guide](docs/micro-app-development.md)**.

### Application Types Introduction

micro_proxy supports three application types:

| Type | Description | Access Method |
|------|------|----------|
| **Static** | Static applications (frontend pages) | Exposed externally through Nginx reverse proxy |
| **API** | API services (backend interfaces) | Exposed externally through Nginx reverse proxy |
| **Internal** | Internal services (databases etc.) | Only for internal micro-app communication |

### SPA Deployment Considerations

Single Page Application (SPA) deployment requires special attention to the following points, see development guide document for details:

- ✅ Nginx configuration must include `try_files` directive
- ✅ Dockerfile must copy custom `nginx.conf`
- ✅ BASE_URL must end with forward slash for sub-path deployments
- ✅ Force rebuild is required after modifying environment variables

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

### Port Conflict Issues

If you encounter port occupied errors:

```bash
# Check port usage
sudo lsof -i :80
sudo lsof -i :8080

# Modify nginx_host_port in proxy-config.yml
nginx_host_port: 8080  # Change to another unoccupied port
```

### Volumes Mount Issues

If you encounter volumes mount failures:

```bash
# Check if host path exists
ls -la ./data

# Check mount point inside container
docker exec <container-name> ls -la /app/data

# View container details
docker inspect <container-name> | grep -A 10 Mounts
```

### SSL Certificate Related Issues

If HTTPS is not working:

```bash
# Check if certificate files exist
ls -la /etc/nginx/certs/

# Verify nginx configuration
docker exec proxy-nginx nginx -t

# View nginx error logs
docker logs proxy-nginx | grep -i ssl

# Manually test HTTPS connection
curl -k https://your-domain.com
```

> ℹ️ **More Help**: SSL configuration troubleshooting can be found in [SSL Configuration Complete Guide](docs/ssl-configuration.md#troubleshooting)

## Project Structure

```
proxy-config/
├── docs/
│   ├── acme-installation.md     # ACME.sh Installation Guide
│   ├── certificate-application.md  # Certificate Request Guide
│   ├── ssl-configuration.md     # SSL Configuration Complete Guide
│   └── micro-app-development.md  # Micro-app Development Guide
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
