
# micro_proxy

[![Crates.io](https://img.shields.io/crates/v/micro_proxy)](https://crates.io/crates/micro_proxy)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://www.rust-lang.org)](https://www.rust-lang.org)

一个用于管理微应用的工具，支持 Docker 镜像构建、容器管理、Nginx 反向代理配置等功能。

关于微应用开发的详细说明，请参阅 **[微应用开发专题](docs/micro-app-development.md)**。

[Home](https://www.craftaidhub.com)

## 📑 文档目录

本文档包含以下内容，帮助您快速了解和使用 micro_proxy：

- [功能特性](#功能特性) - 了解 tool 的核心功能和优势
- [安装](#安装) - 如何安装 micro_proxy
- [快速开始](#快速开始) - 五分钟上手指南
- [命令说明](#命令说明) - 所有可用命令的详细说明
- [配置说明](#配置说明) - 配置文件详解和最佳实践
- [SSL 证书配置（可选）](#ssl-证书配置可选) - HTTPS 证书配置指南
- [微应用开发](#微应用开发) - 微应用的开发规范和要求
- [故障排查](#故障排查) - 常见问题和解决方案
- [项目结构](#项目结构) - 源码目录组织
- [技术栈](#技术栈) - 使用的技术和依赖
- [许可证](#许可证) - 开源协议
- [贡献](#贡献) - 参与项目的方式

---

## 功能特性

- 🔍 **自动发现微应用** - 支持多个扫描目录，自动发现包含 Dockerfile 的微应用
- 🐳 **Docker 镜像构建** - 自动构建微应用的 Docker 镜像，支持环境变量传递
- 🔄 **容器生命周期管理** - 启动、停止、清理容器
- 🌐 **Nginx 反向代理** - 自动生成 nginx 配置，作为统一入口
- 📦 **Docker Compose 集成** - 生成 docker-compose.yml 文件
- 📊 **状态管理** - 基于目录 hash 判断是否需要重新构建
- 🌍 **网络管理** - 统一管理 Docker 网络，支持微应用间通信
- 📝 **脚本支持** - 支持预构建 (setup.sh) 和清理 (clean.sh) 脚本
- 📋 **网络地址列表** - 生成网络地址列表，便于排查连通性问题
- 🔒 **内部服务支持** - 支持 Redis、MySQL 等不需要 nginx 代理的内部服务
- 🔐 **SSL 证书支持** - 支持 Let's Encrypt 证书申请，自动配置 ACME 验证（可选）
- 💾 **Volumes 映射支持** - 支持为微应用配置 Docker volumes 映射，实现数据持久化

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

所有应用通过 Nginx 统一入口访问，默认端口为 80（可在 `proxy-config.yml` 配置文件中的 `nginx_host_port` 字段修改）：

```bash
# 访问主应用
curl http://localhost/

# 访问 API 服务
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
- `--network`: 同时清理 Docker 网络

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

# Nginx 配置文件输出路径
nginx_config_path: "./nginx.conf"

# Docker Compose 配置文件输出路径
compose_config_path: "./docker-compose.yml"

# 状态文件路径
state_file_path: "./proxy-config.state"

# 网络地址列表输出路径
network_list_path: "./network-addresses.txt"

# Docker 网络名称
network_name: "proxy-network"

# Nginx 监听的主机端口（统一入口）
# 注意：这是宿主机端口，通过 Docker 端口映射到容器内部的 80 端口
# 例如：设置为 8080 时，访问 http://localhost:8080 会映射到容器内部的 80 端口
nginx_host_port: 80

# Web 根目录（可选）
# 用于存放 ACME 验证文件，支持 Let's Encrypt 证书申请
# 默认值："/var/www/html"
web_root: "/var/www/html"

# 证书目录（可选）
# 主机上存放 SSL 证书的目录
# 默认值："/etc/nginx/certs"
cert_dir: "/etc/nginx/certs"

# 域名（可选）
# 用于配置 HTTPS。如果配置了此字段且证书文件存在，nginx 将启用 HTTPS
# 证书文件命名规则：{cert_dir}/{domain}.cer (或 .crt)
# 密钥文件命名规则：{cert_dir}/{domain}.key
domain: "example.com"

# 反向代理配置
apps:
  # Static 和 Api 类型：name 必须与扫描发现的微应用文件夹名称一致
  - name: "app-name"
    routes: ["/", "/api"]          # 访问路径
    container_name: "container"    # 容器名称
    container_port: 80             # 容器内部端口
    app_type: "static"             # 应用类型：static, api 或 internal
    description: "应用描述"        # 可选
    docker_volumes:                # Docker volumes 映射（可选）
      - "./data:/app/data"         # 读写挂载
      - "./config:/app/config:ro"  # 只读挂载
    nginx_extra_config: |          # 可选：额外的 nginx 配置（仅 static 和 api 有效）
      add_header 'X-Custom-Header' 'value';

  # Internal 类型：不需要 nginx 反向代理，仅用于微应用间内部通信
  - name: "redis"
    routes: []                     # Internal 类型 routes 为空
    container_name: "redis-container"
    container_port: 6379
    app_type: "internal"
    description: "Redis 缓存服务"
    path: "./services/redis"       # 必须配置，指向服务文件夹路径
    docker_volumes:                # Docker volumes 映射（可选）
      - "./redis-data:/data"       # 持久化 Redis 数据
```

### SSL 证书配置说明

> ℹ️ **完整指南**：如需详细了解 SSL 证书的配置方法、工作原理和常见问题解答，请查阅 **[SSL 配置完整指南](docs/ssl-configuration.md)**。

micro_proxy 支持 Let's Encrypt 证书申请，通过 ACME 协议自动验证域名所有权。以下是简要说明：

#### 必需的三个配置项

| 配置项 | 作用 | 默认值 |
|--------|------|--------|
| `web_root` | 用于存放 ACME 验证文件的目录，Let's Encrypt 通过此目录验证域名所有权 | `/var/www/html` |
| `cert_dir` | 存放 SSL 证书和私钥的目录，会自动挂载到 Nginx 容器 | `/etc/nginx/certs` |
| `domain` | 域名，用于推导证书文件路径和 Nginx 配置 | 无（可选） |

#### 工作流程

1. **申请证书**：使用 acme.sh 向 Let's Encrypt 申请证书
2. **放置证书**：证书被保存到 `cert_dir` 目录
3. **挂载目录**：`docker-compose.yml` 自动将 `cert_dir` 挂载到 Nginx 容器
4. **启用 HTTPS**：检测到证书后，自动生成 HTTPS 配置

#### 快速配置步骤

```bash
# 1. 在 proxy-config.yml 中配置以下三项
web_root: "/var/www/html"
cert_dir: "/etc/nginx/certs"
domain: "your-domain.com"

# 2. 确保目录存在并有写入权限
sudo mkdir -p /var/www/html
sudo mkdir -p /etc/nginx/certs

# 3. 使用 acme.sh 申请证书
acme.sh --issue -d your-domain.com --webroot /var/www/html

# 4. 部署证书
acme.sh --install-cert -d your-domain.com \
  --key-file /etc/nginx/certs/your-domain.com.key \
  --fullchain-file /etc/nginx/certs/your-domain.com.cer

# 5. 启动服务
micro_proxy start
```

#### docker-compose.yml 中的挂载

启用 SSL 后，生成的 `docker-compose.yml` 中会包含：

```yaml
services:
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"  # HTTPS 端口
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - /var/www/html:/var/www/html:ro      # web_root 挂载
      - /etc/nginx/certs:/etc/nginx/certs:ro  # cert_dir 挂载
```

#### ⚠️ 常见疑问

- **web_root 会不会与我的应用冲突？**  
  不会。ACME 验证 location 只匹配 `/.well-known/acme-challenge/` 路径，不影响其他路由。

- **cert_dir 的作用是什么？**  
  确保证书持久化存储在宿主机，不会被容器删除影响，并可被 Nginx 容器访问。

- **domain 除了推导文件路径还有什么用？**  
  还用于 Nginx 的 `server_name` 配置，以及自动启用 HTTPS 的开关。

> 🔗 **查看更多**：[SSL 配置完整指南](docs/ssl-configuration.md) 包含详细的 FAQ、错误排查和最佳实践。

### Docker Volumes 配置说明

`docker_volumes` 字段用于配置 Docker 容器的卷挂载，实现数据持久化和文件共享。

#### 配置格式

```yaml
docker_volumes:
  - "宿主机路径：容器路径"           # 读写挂载（默认）
  - "宿主机路径：容器路径:ro"       # 只读挂载
  - "宿主机路径：容器路径:rw"       # 读写挂载（显式指定）
```

#### 使用场景

1. **数据持久化**：将容器内的数据目录挂载到宿主机，避免容器删除后数据丢失
   ```yaml
   docker_volumes:
     - "./redis-data:/data"        # Redis 数据持久化
     - "./mysql-data:/var/lib/mysql"  # MySQL 数据持久化
   ```

2. **配置文件共享**：将宿主机的配置文件挂载到容器中，便于修改配置
   ```yaml
   docker_volumes:
     - "./config:/app/config:ro"   # 只读挂载配置文件
   ```

3. **日志输出**：将容器内的日志目录挂载到宿主机，便于查看和分析日志
   ```yaml
   docker_volumes:
     - "./logs:/app/logs"          # 日志输出到宿主机
   ```

4. **文件上传**：将用户上传的文件存储到宿主机
   ```yaml
   docker_volumes:
     - "./uploads:/app/uploads"    # 用户上传文件存储
   ```

#### 注意事项

- **路径格式**：支持相对路径和绝对路径
  - 相对路径：`./data:/app/data`（相对于 docker-compose.yml 所在目录）
  - 绝对路径：`/var/data:/app/data`

- **权限控制**：
  - `ro`：只读挂载，容器内无法修改
  - `rw`：读写挂载（默认），容器内可以修改

- **目录创建**：宿主机路径如果不存在，Docker 会自动创建

- **路径分隔符**：建议使用正斜杠 `/`，即使在 Windows 系统上

#### 反向代理配置示例

```yaml
apps:
  # 静态网站应用
  - name: "main-app"
    routes: ["/"]
    container_name: "main-container"
    container_port: 80
    app_type: "static"
    docker_volumes:
      - "./static-data:/usr/share/nginx/html/data"
      - "./static-config:/etc/nginx/conf.d:ro"

  # API 服务
  - name: "api-service"
    routes: ["/api"]
    container_name: "api-container"
    container_port: 3000
    app_type: "api"
    docker_volumes:
      - "./api-logs:/app/logs"
      - "./api-uploads:/app/uploads"

  # Redis 内部服务
  - name: "redis"
    routes: []
    container_name: "redis-container"
    container_port: 6379
    app_type: "internal"
    path: "./services/redis"
    docker_volumes:
      - "./redis-data:/data"
```

### 端口配置说明

micro_proxy 使用 Docker 端口映射机制，将宿主机端口映射到容器内部端口。理解这个机制对于正确配置非常重要。

#### 端口映射架构

```
┌─────────────────────────────────────────────────────────────┐
│                        宿主机                                │
│                                                              │
│  用户访问：http://localhost:8080                             │
│         │                                                     │
│         ▼                                                     │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  Docker 端口映射 (8080:80)                          │    │
│  │  nginx_host_port: 8080  ──映射──►  容器内部：80     │    │
│  └─────────────────────────────────────────────────────┘    │
│         │                                                     │
│         ▼                                                     │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              Nginx 容器                              │    │
│  │                                                      │    │
│  │  nginx.conf 中的 listen 指令：                       │    │
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
| `nginx_host_port` | 宿主机端口 | 80 | 用户访问的端口，通过 Docker 端口映射到容器内部 |
| `nginx.conf` 中的 `listen` | 容器内部端口 | 80 | 固定值，由 micro_proxy 自动生成，无需手动修改 |

#### 端口映射提示
1. **nginx_host_port 只影响宿主机端口**
   - 修改 `nginx_host_port` 只会改变 Docker 容器在宿主机上的端口
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
  这种情况下，系统会报错：`发现重复的微应用名称：common`

**4. 目录命名建议**
- 建议使用有意义的、唯一的目录名称
- 目录名称将作为微应用的默认名称
- 避免使用特殊字符和空格

### 📁 微应用目录结构说明

配置文件中定义的 `scan_dirs` 目录将包含多个微应用子目录。**每个微应用子目录需要遵循特定的文件结构规范**才能被正确识别和构建。

⚠️ **重要提示**：微应用的具体文件结构要求、关键文件命名规范以及构建流程详情，请参阅专门的技术文档：

👉 **[查看详细规范 → 微应用开发专题](docs/micro-app-development.md)**

**核心文件结构概览：**

| 文件/目录 | 是否必需 | 说明 |
|-----------|----------|------|
| `Dockerfile` | ✅ 必需 | Docker 镜像构建文件，位于微应用根目录 |
| `nginx.conf` | ⚠️ 条件 | SPA 部署时必需，自定义 Nginx 配置 |
| `setup.sh` | ❌ 可选 | 构建前执行脚本，可用于环境准备 |
| `clean.sh` | ❌ 可选 | 清理脚本，用于移除构建产物 |
| `.env` | ❌ 可选 | 环境变量文件 |
| `dist/` 或 `build/` | ⚠️ 条件 | 前端项目的构建输出目录 |

**标准微应用目录结构示例：**

```
micro-apps/
└── my-app/                    # 微应用目录
    ├── Dockerfile             # Docker 构建文件（必需）
    ├── nginx.conf             # Nginx 配置（SPA 应用必需）
    ├── setup.sh               # 构建前脚本（可选）
    ├── clean.sh               # 清理脚本（可选）
    ├── .env                   # 环境变量（可选）
    ├── package.json           # Node.js 项目配置
    ├── src/                   # 源代码目录
    └── dist/                  # 构建输出目录
```

> 💡 **提示**：不同类型的微应用（Static、Api、Internal）有不同的文件要求和配置方式，详细规范和最佳实践请参考 [微应用开发专题](docs/micro-app-development.md)。

## 微应用开发

关于微应用开发的详细说明，请参阅 **[微应用开发专题](docs/micro-app-development.md)**。

### 应用类型简介

micro_proxy 支持三种应用类型：

| 类型 | 说明 | 访问方式 |
|------|------|----------|
| **Static** | 静态应用（前端页面） | 通过 Nginx 反向代理对外服务 |
| **API** | API 服务（后端接口） | 通过 Nginx 反向代理对外服务 |
| **Internal** | 内部服务（数据库等） | 仅用于微应用间内部通信 |

### SPA 部署注意事项

单页应用（SPA）部署时需要特别注意以下几点，详见开发专题文档：

- ✅ Nginx 配置必须包含 `try_files` 指令
- ✅ Dockerfile 必须复制自定义的 `nginx.conf`
- ✅ 子路径部署时 BASE_URL 必须以斜杠结尾
- ✅ 修改环境变量后需要强制重新构建

## 故障排查

### 查看日志

```bash
# 显示详细日志
micro_proxy start -v

# 查看容器日志
docker logs <container-name>

# 查看 nginx 日志
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

# 使用 docker 命令查看
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

### Volumes 挂载问题

如果遇到 volumes 挂载失败：

```bash
# 检查宿主机路径是否存在
ls -la ./data

# 检查容器内的挂载点
docker exec <container-name> ls -la /app/data

# 查看容器详细信息
docker inspect <container-name> | grep -A 10 Mounts
```

### SSL 证书相关问题

如果 HTTPS 不工作：

```bash
# 检查证书文件是否存在
ls -la /etc/nginx/certs/

# 验证 nginx 配置
docker exec proxy-nginx nginx -t

# 查看 nginx 错误日志
docker logs proxy-nginx | grep -i ssl

# 手动测试 HTTPS 连接
curl -k https://your-domain.com
```

> ℹ️ **更多帮助**：SSL 配置疑难解答请参见 [SSL 配置完整指南](docs/ssl-configuration.md#故障排查)

## 项目结构

```
proxy-config/
├── docs/
│   ├── acme-installation.md     # ACME.sh 安装指南
│   ├── certificate-application.md  # 证书申请指南
│   ├── ssl-configuration.md     # SSL 配置完整指南
│   └── micro-app-development.md  # 微应用开发专题
├── src/
│   ├── main.rs          # 主入口
│   ├── lib.rs           # 库入口
│   ├── cli.rs           # 命令行接口
│   ├── config.rs        # 配置管理
│   ├── discovery.rs     # 应用发现
│   ├── builder.rs       # 镜像构建
│   ├── container.rs     # 容器管理
│   ├── nginx.rs         # Nginx 配置生成
│   ├── compose.rs       # Docker Compose 生成
│   ├── state.rs         # 状态管理
│   ├── script.rs        # 脚本执行
│   ├── network.rs       # 网络管理
│   ├── dockerfile.rs    # Dockerfile 解析
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

欢迎提交 Issue 和 Pull Request！
