
# micro_proxy

[![Crates.io](https://img.shields.io/crates/v/micro_proxy)](https://crates.io/crates/micro_proxy)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)

一个用于管理微应用的工具，支持Docker镜像构建、容器管理、Nginx反向代理配置等功能。
[Home](https://www.craftaidhub.com)

## 功能特性

- 🔍 **自动发现微应用** - 支持多个扫描目录，自动发现包含Dockerfile的微应用
- 🐳 **Docker镜像构建** - 自动构建微应用的Docker镜像，支持环境变量传递
- 🔄 **容器生命周期管理** - 启动、停止、清理容器
- 🌐 **Nginx反向代理** - 自动生成nginx配置，作为统一入口
- 📦 **Docker Compose集成** - 生成docker-compose.yml文件
- 📊 **状态管理** - 基于目录hash判断是否需要重新构建
- 🌍 **网络管理** - 统一管理Docker网络，支持微应用间通信
- 📝 **脚本支持** - 支持预构建(setup.sh)和清理(clean.sh)脚本
- 📋 **网络地址列表** - 生成网络地址列表，便于排查连通性问题
- 🔒 **内部服务支持** - 支持Redis、MySQL等不需要nginx代理的内部服务
- 🔐 **SSL证书支持** - 支持Let's Encrypt证书申请，自动配置ACME验证（可选）

## 安装

### 从 crates.io 安装（推荐）

```bash
cargo install micro_proxy
```

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/cao5zy/proxy-config
cd proxy-config

# 构建
cargo build --release

# 安装
cargo install --path .
```

## 快速开始

### 1. 创建配置文件

复制示例配置文件并根据需要修改：

```bash
cp proxy-config.yml.example proxy-config.yml
```

### 2. 启动微应用

```bash
# 启动所有微应用
micro_proxy start

# 强制重新构建所有镜像
micro_proxy start --force-rebuild

# 显示详细日志
micro_proxy start -v
```

### 3. 访问应用

所有应用通过Nginx统一入口访问，默认端口为80（可在 `proxy-config.yml` 配置文件中的 `nginx_host_port` 字段修改）：

```bash
# 访问主应用
curl http://localhost/

# 访问API服务
curl http://localhost/api
```

## 命令说明

### start - 启动微应用

```bash
micro_proxy start [options]
```

选项：
- `-c, --config <path>`: 指定配置文件路径（默认：./micro_proxy.yml）
- `--force-rebuild`: 强制重新构建所有镜像

### stop - 停止微应用

```bash
micro_proxy stop [options]
```

选项：
- `-c, --config <path>`: 指定配置文件路径（默认：./proxy-config.yml）

### clean - 清理微应用

```bash
micro_proxy clean [options]
```

选项：
- `-c, --config <path>`: 指定配置文件路径（默认：./proxy-config.yml）
- `--force`: 强制清理，不询问确认
- `--network`: 同时清理Docker网络

### status - 查看状态

```bash
micro_proxy status [options]
```

选项：
- `-c, --config <path>`: 指定配置文件路径（默认：./proxy-config.yml）

### network - 查看网络地址

```bash
micro_proxy network [options]
```

选项：
- `-c, --config <path>`: 指定配置文件路径（默认：./proxy-config.yml）
- `-o, --output <path>`: 指定输出文件路径（覆盖配置文件中的设置）

## 配置说明

### 主配置文件 (proxy-config.yml)

```yaml
# 扫描目录列表（用于 Static 和 Api 类型）
scan_dirs:
  - "./micro-apps"

# Nginx配置文件输出路径
nginx_config_path: "./nginx.conf"

# Docker Compose配置文件输出路径
compose_config_path: "./docker-compose.yml"

# 状态文件路径
state_file_path: "./proxy-config.state"

# 网络地址列表输出路径
network_list_path: "./network-addresses.txt"

# Docker网络名称
network_name: "proxy-network"

# Nginx监听的主机端口（统一入口）
# 注意：这是宿主机端口，通过Docker端口映射到容器内部的80端口
# 例如：设置为8080时，访问 http://localhost:8080 会映射到容器内部的80端口
nginx_host_port: 80

# Web根目录（可选）
# 用于存放 ACME 验证文件，支持 Let's Encrypt 证书申请
# 默认值: "/var/www/html"
# 如果不需要配置 HTTPS 证书，可以不设置此字段
# web_root: "/var/www/html"

# 证书目录（可选）
# 主机上存放 SSL 证书的目录
# 默认值: "/etc/nginx/certs"
# 如果不需要配置 HTTPS 证书，可以不设置此字段
# cert_dir: "/etc/nginx/certs"

# 域名（可选）
# 用于配置 HTTPS。如果配置了此字段且证书文件存在，nginx 将启用 HTTPS
# 证书文件命名规则: {cert_dir}/{domain}.cer (或 .crt)
# 密钥文件命名规则: {cert_dir}/{domain}.key
# 示例:
# domain: "example.com"

# 反向代理配置
apps:
  # Static 和 Api 类型：name 必须与扫描发现的微应用文件夹名称一致
  - name: "app-name"
    routes: ["/", "/api"]          # 访问路径
    container_name: "container"    # 容器名称
    container_port: 80             # 容器内部端口
    app_type: "static"             # 应用类型: static, api 或 internal
    description: "应用描述"        # 可选
    nginx_extra_config: |          # 可选：额外的nginx配置（仅 static 和 api 有效）
      add_header 'X-Custom-Header' 'value';

  # Internal 类型：不需要 nginx 反向代理，仅用于微应用间内部通信
  - name: "redis"
    routes: []                     # Internal 类型 routes 为空
    container_name: "redis-container"
    container_port: 6379
    app_type: "internal"
    description: "Redis 缓存服务"
    path: "./services/redis"       # 必须配置，指向服务文件夹路径
```

### 端口配置说明

micro_proxy 使用 Docker 端口映射机制，将宿主机端口映射到容器内部端口。理解这个机制对于正确配置非常重要。

#### 端口映射架构

```
┌─────────────────────────────────────────────────────────────┐
│                        宿主机                                │
│                                                              │
│  用户访问: http://localhost:8080                             │
│         │                                                     │
│         ▼                                                     │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  Docker 端口映射 (8080:80)                          │    │
│  │  nginx_host_port: 8080  ──映射──►  容器内部: 80     │    │
│  └─────────────────────────────────────────────────────┘    │
│         │                                                     │
│         ▼                                                     │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              Nginx 容器                              │    │
│  │                                                      │    │
│  │  nginx.conf 中的 listen 指令:                        │    │
│  │    - HTTP:  listen 80;                               │    │
│  │    - HTTPS: listen 443 ssl;                          │    │
│  │                                                      │    │
│  │  注意：nginx.conf 中的端口是容器内部端口，            │    │
│  │       固定为 80 (HTTP) 和 443 (HTTPS)                │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

#### 配置项说明

| 配置项 | 作用 | 示例值 | 说明 |
|--------|------|--------|------|
| `nginx_host_port` | 宿主机端口 | 8080 | 用户访问的端口，通过 Docker 端口映射到容器内部 |
| `nginx.conf` 中的 `listen` | 容器内部端口 | 80 | 固定值，由 micro_proxy 自动生成，无需手动修改 |

#### 端口映射示例

**示例 1：使用默认端口 80**

```yaml
# proxy-config.yml
nginx_host_port: 80
```

生成的 docker-compose.yml：
```yaml
services:
  nginx:
    ports:
      - "80:80"    # 宿主机80端口映射到容器内部80端口
```

访问方式：
```bash
curl http://localhost/
```

**示例 2：使用自定义端口 8080**

```yaml
# proxy-config.yml
nginx_host_port: 8080
```

生成的 docker-compose.yml：
```yaml
services:
  nginx:
    ports:
      - "8080:80"  # 宿主机8080端口映射到容器内部80端口
```

访问方式：
```bash
curl http://localhost:8080/
```

**示例 3：启用 HTTPS**

```yaml
# proxy-config.yml
nginx_host_port: 8080
domain: "example.com"
```

生成的 docker-compose.yml：
```yaml
services:
  nginx:
    ports:
      - "8080:80"   # HTTP: 宿主机8080端口映射到容器内部80端口
      - "443:443"   # HTTPS: 宿主机443端口映射到容器内部443端口
```

访问方式：
```bash
# HTTP (会重定向到 HTTPS)
curl http://localhost:8080/

# HTTPS
curl https://example.com/
```

#### 重要提示

1. **nginx_host_port 只影响宿主机端口**
   - 修改 `nginx_host_port` 只会改变 Docker 端口映射
   - 不会影响 `nginx.conf` 中的 `listen` 指令

2. **nginx.conf 中的端口是固定的**
   - HTTP: 固定为 80
   - HTTPS: 固定为 443
   - 这些端口由 micro_proxy 自动生成，无需手动修改

3. **端口冲突处理**
   - 如果宿主机端口已被占用，请修改 `nginx_host_port`
   - 例如：如果 80 端口被占用，可以设置为 8080

4. **HTTPS 端口**
   - 启用 HTTPS 时，443 端口会自动添加到端口映射
   - 443 端口是 HTTPS 的标准端口，通常不需要修改

#### 扫描目录说明

`scan_dirs` 配置项用于指定扫描微应用的目录列表，以下是重要的使用规则：

**1. 只扫描一级目录**
- 系统只会读取指定目录下的**一级子目录**，不会递归扫描
- 每个一级子目录被视为一个独立的微应用
- 示例：
  ```
  ./micro-apps/
  ├── app1/          # 会被扫描
  ├── app2/          # 会被扫描
  └── nested/
      └── app3/      # 不会被扫描（二级目录）
  ```

**2. 支持多个扫描目录**
- 如果微应用分布在多个不同的目录下，可以在 `scan_dirs` 中指定多个目录
- 示例：
  ```yaml
  scan_dirs:
    - "./frontend-apps"
    - "./backend-apps"
    - "./services"
  ```

**3. 目录名称唯一性要求**
- 在所有扫描目录中，**不允许存在相同名称的目录**
- 如果扫描到相同的目录名称，系统会报错并退出
- 示例（错误配置）：
  ```
  ./frontend-apps/
  └── common/        # 与 backend-apps 下的 common 重名
  
  ./backend-apps/
  └── common/        # 与 frontend-apps 下的 common 重名
  ```
  这种情况下，系统会报错：`发现重复的微应用名称: common`

**4. 目录命名建议**
- 建议使用有意义的、唯一的目录名称
- 目录名称将作为微应用的默认名称
- 避免使用特殊字符和空格

## SSL证书配置（可选）

> **重要提示：SSL证书配置是完全可选的！**  
> 如果不配置HTTPS证书，micro_proxy仍然可以正常运行，HTTP（80端口）的反向代理功能不受影响。

micro_proxy 支持 Let's Encrypt 证书申请，通过 ACME 协议自动验证域名所有权。

### 配置步骤概览

1. **决定是否需要 HTTPS**：如果不需要，可以完全忽略 SSL 相关配置
2. **配置 `proxy-config.yml`**：设置 `web_root`、`cert_dir` 和 `domain` 字段
3. **申请 SSL 证书**：使用 ACME.sh 工具申请证书
4. **验证配置**：确保证书文件存在且 Nginx 能正确加载

### 详细配置指南

完整的 SSL 配置和证书申请指南请参考以下文档：

- [ACME.sh 安装与配置指南](docs/acme-installation.md)
- [SSL 证书申请指南](docs/certificate-application.md)

### ACME 验证机制

micro_proxy 会在生成的 Nginx 配置中自动添加 ACME 验证 location：

```nginx
location /.well-known/acme-challenge/ {
    root /var/www/html;
    default_type "text/plain";
}
```

**重要说明：**
- ACME location 只匹配 `/.well-known/acme-challenge/` 路径
- 不会影响其他正常的反向代理请求
- 即使不配置证书，HTTP 反向代理仍然可以正常工作

### Docker Compose 配置

确保 Docker Compose 配置中正确挂载了证书目录：

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

### 注意事项

1. **域名解析**：确保域名已正确解析到服务器 IP
2. **防火墙**：确保 80 和 443 端口已开放
3. **web_root 挂载**：确保 Nginx 容器可以访问 web_root 目录
4. **cert_dir 挂载**：确保 Nginx 容器可以访问 cert_dir 目录
5. **自动续期**：acme.sh 默认开启自动续期，无需额外配置
6. **可选配置**：如果不配置 HTTPS，可以完全忽略 `web_root`、`cert_dir` 和 `domain` 字段

## 微应用开发专题

### 什么是微应用？

**微应用**是借鉴微服务架构思想的一种应用组织方式。每个微应用都是一个独立的、可单独部署的软件单元，通过 Docker 容器化技术进行封装。多个微应用可以组合成一个更复杂的系统，同时保持各自的独立性和可维护性。

**核心特点：**
- **独立性**：每个微应用都有自己的代码库、依赖和配置
- **可组合性**：多个微应用可以协同工作，构建复杂系统
- **可持续性**：支持独立开发、测试、部署和扩展
- **容器化**：通过 Docker 实现标准化的部署和运行环境

### 微应用目录结构

每个微应用都必须是一个独立的文件夹，文件夹名称即为微应用的名称。

#### 前端/静态应用和API服务

适用于需要通过 Nginx 对外提供服务的微应用：

```
micro-apps/
├── main-app/              # 文件夹名称即为微应用名称
│   ├── Dockerfile         # 必须在项目根目录
│   ├── .env               # 环境变量文件（可选）
│   ├── setup.sh           # 可选：预构建脚本
│   ├── clean.sh           # 可选：清理脚本
│   └── ...                # 其他应用文件
├── resume-app/
│   ├── Dockerfile
│   ├── .env
│   └── ...
└── api-service/
    ├── Dockerfile
    ├── .env
    └── ...
```

#### 内部服务

适用于 Redis、MySQL 等仅用于内部通信的服务：

```
services/
├── redis/                 # 服务文件夹
│   ├── Dockerfile         # 必须在项目根目录
│   ├── .env               # 环境变量文件（可选）
│   ├── setup.sh           # 可选：预构建脚本
│   ├── clean.sh           # 可选：清理脚本
│   └── ...                # 其他服务文件
└── mysql/
    ├── Dockerfile
    ├── .env
    └── ...
```

### 应用类型

micro_proxy 支持三种应用类型，决定了微应用的访问方式和配置：

#### 1. Static（静态应用）
- **适用场景**：前端应用、静态网站
- **特点**：启用浏览器缓存，适合静态资源
- **访问方式**：通过 Nginx 反向代理对外提供服务
- **配置示例**：
  ```yaml
  - name: "frontend"
    routes: ["/app"]
    app_type: "static"
    container_port: 80
  ```

#### 2. API（API服务）
- **适用场景**：后端 API 服务、微服务
- **特点**：禁用缓存，保留完整请求路径
- **访问方式**：通过 Nginx 反向代理对外提供服务
- **配置示例**：
  ```yaml
  - name: "backend"
    routes: ["/api"]
    app_type: "api"
    container_port: 8080
  ```

#### 3. Internal（内部服务）
- **适用场景**：Redis、MySQL、MongoDB 等数据库服务
- **特点**：不通过 Nginx 对外暴露，仅用于微应用间内部通信
- **访问方式**：其他微应用通过容器名称直接访问
- **配置示例**：
  ```yaml
  - name: "redis"
    app_type: "internal"
    container_port: 6379
    path: "./services/redis"
  ```

### 开发工作流

#### 1. Dockerfile 要求
- 必须放在微应用项目根目录
- 建议使用 `EXPOSE` 指令声明端口
- 示例：
  ```dockerfile
  FROM nginx:alpine
  EXPOSE 80
  COPY . /usr/share/nginx/html
  ```

#### 2. 环境变量配置
- 在 `.env` 文件中定义构建时环境变量
- 这些变量会在构建时传递给 Docker
- 示例：
  ```env
  APP_PORT=80
  ENV=production
  ```

#### 3. 自动化脚本
- **setup.sh**：在构建镜像前执行，用于准备环境
- **clean.sh**：在清理时执行，用于清理构建产物
- 脚本必须放在微应用项目根目录

### 网络与通信

所有微应用运行在同一个 Docker 网络中，支持以下通信方式：

#### 对外服务
- Static 和 API 类型的微应用通过 Nginx 统一入口对外提供服务
- 访问地址：`http://<host>:<nginx_host_port>/<configured-route>`

#### 内部通信
- 所有微应用可以通过容器名称相互访问
- 示例：
  ```bash
  # frontend 访问 backend
  curl http://backend:8080/api
  
  # backend 访问 redis
  redis-cli -h redis -p 6379
  ```

### 反向代理配置

micro_proxy 会根据应用类型自动生成合适的 Nginx 配置：

#### Static 类型路径处理
- **根路径** (`/`)：直接转发请求
- **子路径** (`/app`)：自动移除路径前缀
  - 访问 `/app/index.html` → 后端收到 `/index.html`

#### API 类型路径处理
- **保留完整路径**：不修改请求 URI
  - 访问 `/api/v1/users` → 后端收到 `/api/v1/users`

### 自定义配置

可以为 Static 和 API 类型的微应用添加额外的 Nginx 配置：

```yaml
- name: "main-app"
  routes: ["/"]
  nginx_extra_config: |
    add_header 'X-Custom-Header' 'value';
    location /api {
      proxy_pass http://backend:3000;
    }
```

## 故障排查

### 查看日志

```bash
# 显示详细日志
micro_proxy start -v

# 查看容器日志
docker logs <container-name>

# 查看nginx日志
docker logs proxy-nginx
```

### 查看网络地址

```bash
# 生成并查看网络地址列表
micro_proxy network

# 查看生成的文件
cat network-addresses.txt
```

### 检查容器状态

```bash
# 查看所有容器状态
micro_proxy status

# 使用docker命令查看
docker ps -a
```

### 端口冲突问题

如果遇到端口被占用的错误：

```bash
# 检查端口占用情况
sudo lsof -i :80
sudo lsof -i :8080

# 修改 proxy-config.yml 中的 nginx_host_port
nginx_host_port: 8080  # 改为其他未被占用的端口
```

## 项目结构

```
proxy-config/
├── docs/
│   ├── acme-installation.md     # ACME.sh 安装指南
│   └── certificate-application.md  # 证书申请指南
├── src/
│   ├── main.rs          # 主入口
│   ├── lib.rs           # 库入口
│   ├── cli.rs           # 命令行接口
│   ├── config.rs        # 配置管理
│   ├── discovery.rs     # 应用发现
│   ├── builder.rs       # 镜像构建
│   ├── container.rs     # 容器管理
│   ├── nginx.rs         # Nginx配置生成
│   ├── compose.rs       # Docker Compose生成
│   ├── state.rs         # 状态管理
│   ├── script.rs        # 脚本执行
│   ├── network.rs       # 网络管理
│   ├── dockerfile.rs    # Dockerfile解析
│   └── error.rs         # 错误定义
├── Cargo.toml           # 项目配置
├── proxy-config.yml.example  # 配置文件示例
└── README.md            # 项目文档
```

## 技术栈

- **Rust** - 主要编程语言
- **Docker** - 容器化
- **Nginx** - 反向代理
- **Docker Compose** - 容器编排

## 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

## 贡献

欢迎提交Issue和Pull Request！
