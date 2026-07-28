
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

- 🔍 **自动发现微应用** - 支持多个扫描目录，自动发现包含 `micro-app.yml` 和 `Dockerfile` 的微应用
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

### 2. 准备微应用

在每个微应用目录下创建 `micro-app.yml` 配置文件：

```bash
cp micro-app.yml.example ./micro-apps/my-app/micro-app.yml
```

### 3. 启动微应用

```bash
# 启动所有微应用
micro_proxy start

# 强制重新构建所有镜像
micro_proxy start --force-rebuild

# 显示详细日志
micro_proxy start -v
```

### 4. 访问应用

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
- `-c, --config <path>`: 指定配置文件路径（默认：./proxy-config.yml）
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
# 扫描目录列表（用于发现 micro-app.yml）
scan_dirs:
  - "./micro-apps"

# 动态生成的 apps 配置存储路径
# 此文件由 micro_proxy 自动生成，请勿手动修改
apps_config_path: "./apps-config.yml"

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
nginx_host_port: 80

# Web 根目录（可选）
web_root: "/var/www/html"

# 证书目录（可选）
cert_dir: "/etc/nginx/certs"

# 域名（可选）
domain: "example.com"
```

### 微应用配置文件 (micro-app.yml)

每个微应用目录下必须包含 `micro-app.yml` 文件，用于配置该微应用的属性：

```yaml
# 访问路径（static/api类型必需；不能以 `/` 结尾，根路由 `/` 除外）
routes: ["/", "/api"]

# Docker容器名称（必需，全局唯一）
container_name: "my-container"

# 容器内部端口（必需）
container_port: 80

# 应用类型（必需）：static, api, internal
app_type: "static"

# 应用描述（可选）
description: "应用描述"

# 额外的 nginx 配置（可选，仅 static 和 api 有效）
nginx_extra_config: |
  add_header 'X-Custom-Header' 'value';

# 代理超时设置（可选，仅 api 类型有效，单位：秒，默认 60）
proxy_connect_timeout: 60
proxy_read_timeout: 60
proxy_send_timeout: 60
```

**详细配置说明**请参阅 **[微应用开发专题](docs/micro-app-development.md)**。

### 数据持久化配置文件 (micro-app.volumes.yml)

若微应用需要持久化数据或设置容器权限，请在微应用目录下创建 `micro-app.volumes.yml` 文件：

```yaml
volumes:
  - source: "./data"         # 宿主机路径（相对或绝对路径）
    target: "/data"          # 容器内部路径，数据将持久化到此位置
    permissions:             # 可选：挂载目录的权限设置
      uid: 999               # 用户 ID
      gid: 999               # 组 ID
      recursive: true        # 是否递归设置权限

run_as_user: "999:999"       # 可选：容器运行时的用户（格式 "uid:gid" 或用户名）
```

**字段说明：**

| 字段 | 必需 | 说明 |
|------|------|------|
| `source` | 是 | 宿主机路径。相对路径以生成的 `docker-compose.yml` 所在位置为基准 |
| `target` | 是 | 容器内部路径，容器中数据实际存储的位置 |
| `permissions` | 否 | 权限配置对象，包含 `uid`、`gid` 和可选的 `recursive` |
| `run_as_user` | 否 | 容器运行时用户，例如 `"999:999"` 或 `"username"` |

**重要约定：**

- `source` 指向的是**宿主机**上的路径；`target` 指向的是**容器内部**的路径。
- `target` 是容器中数据持久化的位置。应用配置中所有需要持久化的数据路径（如数据库文件、上传目录、日志文件等）都必须位于某个 `target` 之下，否则数据会写入容器可写层，容器重启后丢失。
- `docker_volumes` 字段已从 `micro-app.yml` 中移除，请改用独立的 `micro-app.volumes.yml` 文件。

> 示例：若 `.env` 中配置了 `DATABASE_PATH=/app/db.sqlite`，且未将 `/app` 映射为 volume target，则该数据库文件不会被持久化到宿主机。

### SSL 证书配置说明

> ℹ️ **完整指南**：如需详细了解 SSL 证书的配置方法、工作原理和常见问题解答，请查阅 **[SSL 配置完整指南](docs/ssl-configuration.md)**。

micro_proxy 支持 Let's Encrypt 证书申请，通过 ACME 协议自动验证域名所有权。以下是简要说明：

#### 必需的三个配置项

| 配置项 | 作用 | 默认值 |
|--------|------|--------|
| `web_root` | 用于存放 ACME 验证文件的目录，Let's Encrypt 通过此目录验证域名所有权 | `/var/www/html` |
| `cert_dir` | 存放 SSL 证书和私钥的目录，会自动挂载到 Nginx 容器 | `/etc/nginx/certs` |
| `domain` | 域名，用于推导证书文件路径和 Nginx 配置 | 无（可选） |

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

> 🔗 **查看更多**：[SSL 配置完整指南](docs/ssl-configuration.md) 包含详细的 FAQ、错误排查和最佳实践。

### 端口配置说明

micro_proxy 使用 Docker 端口映射机制，将宿主机端口映射到容器内部端口。

| 配置项 | 作用 | 示例值 | 说明 |
|--------|------|--------|------|
| `nginx_host_port` | 宿主机端口 | 80 | 用户访问的端口，通过 Docker 端口映射到容器内部 |
| `nginx.conf` 中的 `listen` | 容器内部端口 | 80 | 固定值，由 micro_proxy 自动生成，无需手动修改 |

**端口映射提示**：
- HTTP: 固定为 80
- HTTPS: 固定为 443
- 如果宿主机端口已被占用，请修改 `nginx_host_port`

### 扫描目录说明

`scan_dirs` 配置项用于指定扫描微应用的目录列表：

- 只扫描一级目录，不会递归扫描
- 只有同时包含 `micro-app.yml` 和 `Dockerfile` 的目录才会被识别为微应用
- 目录名称将作为微应用的默认名称（`app.name`）
- 所有微应用的 `container_name` 必须全局唯一

### 名称推导逻辑

微应用有多个关键名称，它们的来源不同：

| 名称 | 来源 | 推导规则 | 示例 |
|------|------|----------|------|
| **app.name** | 目录名（自动推导） | 直接子目录：目录名即 `app.name`；嵌套目录：路径组件用 `_` 拼接 | `my_app`、`group_my_app` |
| **Docker image name** | 从 `app.name` 推导 | `{app.name}:latest` | `my_app:latest` |
| **nginx 变量名** | 从 `app.name` 推导 | `{app.name}_upstream_host` | `$my_app_upstream_host` |
| **container_name** | 用户在 `micro-app.yml` 中配置 | 不推导，直接配置，需全局唯一 | `my-container` |

**推导链路：**

```
目录名 (my_app/)
  ├──→ app.name = "my_app"
  │      ├──→ Docker image = "my_app:latest"
  │      └──→ nginx 变量 = "$my_app_upstream_host"
  │
  └── micro-app.yml
         └── container_name = "my-container"  (用户自行配置，独立于目录名)
```

**目录命名约束：**

由于 `app.name` 用于构造 nginx 变量名，而 nginx 变量名只允许 `[A-Za-z0-9_]` 且不能以数字开头，因此目录名必须遵守相同规则：

- ✅ `my_app`、`group_my_app`、`App123`
- ❌ `my-app`（含 `-`）、`123app`（数字开头）、`my.app`（含 `.`）

> 使用不合规的目录名会导致 `micro_proxy start` 验证失败并报错。

## 微应用开发

关于微应用开发的详细说明，请参阅 **[微应用开发专题](docs/micro-app-development.md)**。

### 应用类型简介

micro_proxy 支持三种应用类型：

| 类型 | 说明 | 访问方式 |
|------|------|----------|
| **Static** | 静态应用（前端页面） | 通过 Nginx 反向代理对外服务 |
| **API** | API 服务（后端接口） | 通过 Nginx 反向代理对外服务 |
| **Internal** | 内部服务（数据库、MinIO 等） | 无 routes：仅容器间通信；有 routes：通过 Nginx 对外代理（自动剥离路由前缀） |

### 标准微应用目录结构

```
micro-apps/
└── my-app/                    # 微应用目录
    ├── micro-app.yml          # 微应用配置文件（必需）
    ├── Dockerfile             # Docker 构建文件（必需）
    ├── nginx.conf             # Nginx 配置（SPA 应用必需）
    ├── micro-app.volumes.yml  # 数据持久化与权限配置（可选）
    ├── setup.sh               # 构建前脚本（可选）
    ├── clean.sh               # 清理脚本（可选）
    ├── .env                   # 环境变量（可选）
    └── src/                   # 源代码目录
```

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

```bash
# 检查端口占用情况
sudo lsof -i :80
sudo lsof -i :8080

# 修改 proxy-config.yml 中的 nginx_host_port
nginx_host_port: 8080  # 改为其他未被占用的端口
```

### Volumes 挂载问题

请确认 `micro-app.volumes.yml` 中 `source` 和 `target` 的含义：

- `source`：宿主机路径
- `target`：容器内部路径，数据实际持久化到容器中的位置

```bash
# 检查宿主机路径是否存在（对应 source）
ls -la ./data

# 检查容器内的挂载点（对应 target）
docker exec <container-name> ls -la /data

# 查看容器详细信息
docker inspect <container-name> | grep -A 10 Mounts
```

若数据丢失，请检查应用配置的数据路径是否位于某个 volume 的 `target` 之下。

### SSL 证书相关问题

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

### 微应用配置问题

```bash
# 检查 micro-app.yml 是否存在
ls -la ./micro-apps/my-app/micro-app.yml

# 检查 Dockerfile 是否存在
ls -la ./micro-apps/my-app/Dockerfile

# 验证 micro-app.yml 格式
cat ./micro-apps/my-app/micro-app.yml

# 检查 container_name 是否重复
grep -r "container_name:" ./micro-apps/*/micro-app.yml
```

## 项目结构

```
proxy-config/
├── docs/
│   ├── ssl-configuration.md     # SSL 配置完整指南
│   ├── micro-app-development.md # 微应用开发专题
│   └── ...
├── src/
│   ├── main.rs          # 主入口
│   ├── lib.rs           # 库入口
│   ├── cli.rs           # 命令行接口
│   ├── config.rs        # 配置管理
│   ├── discovery.rs     # 应用发现
│   ├── micro_app_config.rs  # 微应用配置解析
│   └── ...
├── Cargo.toml           # 项目配置
├── proxy-config.yml.example  # 配置文件示例
├── micro-app.yml.example     # 微应用配置示例
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

如果你在使用过程中遇到问题，欢迎提交 Issue。

如果你想关注项目的最新动态，或阅读相关的技术文章，欢迎关注我的微信公众号：
![公众号二维码](./assets/wechat-id.png)]
