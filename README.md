
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

- 🔍 **自动发现微应用** - 支持源码包与镜像包：均需 `micro-app.yml`，源码包使用 Dockerfile，镜像包使用镜像归档
- 🐳 **Docker 镜像构建与导入** - 源码包自动构建；镜像包导入构建机预先打好的 Docker 镜像
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

### 3. 构建并部署微应用

```bash
# 首次部署：先为所有应用构建/导入镜像
micro_proxy build

# 首次部署：每个应用都必须明确选择一个活动镜像。
# APP_NAME 使用应用名；镜像引用从 build 的输出复制。
micro_proxy deploy my-app --image my-app:sha-0123456789ab

# 在每个应用都完成 deploy 后，按活动部署状态启动或恢复全部容器
micro_proxy start -v

# 回滚到上一部署版本，不重新构建源码
micro_proxy rollback my-app
```

首次部署时，配置中发现的**每一个应用**都必须完成一次 `build` 和 `deploy`；仅导入镜像但未执行 `deploy` 的应用没有活动部署状态，`micro_proxy start` 会拒绝启动。后续服务器重启或容器停止时，只要镜像和部署状态文件仍在，直接运行 `micro_proxy start` 即可恢复全部应用。发布单个应用的新版本时，也只需要对该应用重新执行 `build` 和 `deploy`。

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
micro_proxy start
```

按部署状态启动已选择的活动镜像；不会扫描源码或隐式构建镜像。

### build - 构建镜像

```bash
micro_proxy build [APP_NAME...] [--no-cache] [--export]
```

不改变容器、Nginx、流量或部署状态。`package_type: source` 时使用 Dockerfile 构建并生成不可变的 `app:sha-<哈希>` 标签；若配置 `build_platform`，则使用 Docker Buildx 构建该目标平台并加载单平台结果。追加 `--export` 会将该镜像导出为 Dockerfile 同级、文件名固定的 `image.tar`。`package_type: image` 时执行 `docker load` 导入 `image_archive`，并保留构建机写入归档的标签。`package_type: registry` 时不构建，镜像本地不存在才执行 `docker pull image`。`APP_NAME` 仅填写应用名（`app.name`，通常由应用目录推导），不接受 `container_name` 或镜像引用。

```bash
micro_proxy build api
```

例如，应用名为 `api`、容器名为 `test_gg123_api` 时，构建、部署与回滚均使用 `api`；`test_gg123_api` 仅用于 `docker logs`、`docker inspect` 等 Docker 操作。命令输出会同时显示应用名、容器名和镜像引用，便于区分三者。

### deploy / rollback - 切换或回滚版本

```bash
micro_proxy deploy <APP_NAME> --image <IMAGE_REF>
micro_proxy deploy <APP_NAME> --image <IMAGE_REF> --force
micro_proxy rollback <APP_NAME>
```

部署会先启动候选容器并等待健康检查，通过后平滑重载 Nginx，最后停止旧容器；失败时旧版本保持运行。若服务尚未提供可用的健康检查接口，完成容器日志和实际访问验证后，可在该次部署显式加 `--force`。它会跳过本次健康检查等待，只会在候选容器仍处于运行状态时切换；容器已经退出时仍拒绝切换。默认不启用，且不会被保存到部署状态。

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

# HTTP 健康检查路径（可选，默认 "/"）
# 检查地址为 http://127.0.0.1:<container_port><healthcheck_path>
# API 可配置为实际的健康接口，例如 "/healthz"
healthcheck_path: "/healthz"

# 应用类型（必需）：static, api, internal
app_type: "static"

# 应用包类型（可选，默认 source）
# source：通过 Dockerfile 构建；image：导入镜像归档；registry：直接使用仓库镜像
package_type: "source"

# 仅 source 模式可选。Mac（尤其 Apple Silicon）构建并部署到 AMD64 Linux 时设置。
# 配置后使用 docker buildx build --platform linux/amd64 --load。
# build_platform: "linux/amd64"

# 仅 image 模式必需。相对路径基于应用目录。
# image_archive: "image.tar"

# 仅 registry 模式必需。本地不存在时 micro_proxy build 会 docker pull；建议固定版本或 digest。
# image: "postgres:15-alpine"

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

`healthcheck_path` 仅用于 `static` 和 `api` 应用的部署健康检查。路径必须以 `/` 开头；未配置时使用 `/`，因此既有 `micro-app.yml` 无需修改。健康检查在容器内部请求 IPv4 回环地址 `127.0.0.1`，镜像需要提供 `wget`，且该路径应返回 2xx 或 3xx 响应。

#### 镜像包发布

镜像包仅保留运行所需配置、可选 `.env`、可选 `micro-app.volumes.yml` 与镜像归档；不需要上传源码。构建机必须先通过源码模式生成不可变标签，再用 `--export` 导出。归档路径始终是应用目录中的 `image.tar`，不随镜像标签变化。

对于 Postgres、Redis 等没有自有源码、直接采用官方镜像的内部服务，使用 `package_type: registry` 和 `image: postgres:15.8-alpine`；它不需要 Dockerfile、源码或开发机导出的 archive。`micro_proxy build postgres` 只会在该镜像尚未存在时执行 `docker pull`，随后使用同一镜像引用执行 `deploy`。生产环境应固定版本或 digest，避免可变标签带来非预期升级。

**首次在部署机上线时，每一个应用都必须先 `build`（导入镜像），再 `deploy`（选择活动镜像）。** `build` 不会写入部署状态，`start` 也不会猜测应使用哪个刚导入的镜像；如果任何已发现应用未完成首次 `deploy`，`micro_proxy start` 会报“没有活动部署状态”。全部应用部署完成后，后续仅需运行 `micro_proxy start` 来恢复它们。

```bash
# 构建机（例如 Apple Silicon Mac 部署至 AMD64 Linux 时，在源码包的 micro-app.yml 设置
# build_platform: "linux/amd64"；这会产生独立的不可变镜像标签）
micro_proxy build my_app --export
# 输出：my_app:sha-0123456789ab，以及 my_app/image.tar

# 部署机的 my_app/micro-app.yml 配置 package_type: image 与 image_archive: image.tar
# 上传 image.tar、micro-app.yml 与可选 .env（无需上传源码或 Dockerfile）
# 对每一个应用重复以下两步：
micro_proxy build my_app    # 仅 docker load，不重新构建，也不改变部署状态
micro_proxy deploy my_app --image my_app:sha-0123456789ab

# 所有应用均已完成首次 deploy 后：
micro_proxy start

# 后续服务器重启或容器停止后：
micro_proxy start
```

发布 `my_app` 的新版本时，只需覆盖上传该应用的 `image.tar`，然后对它重新运行 `micro_proxy build my_app` 与 `micro_proxy deploy my_app --image <新镜像标签>`；其他应用无需重新部署。构建机请将 `image.tar` 加入 `.dockerignore`，避免该归档被发送到下一次 Docker 构建上下文。镜像归档必须保留不可变标签；不要以 `latest` 覆盖不同版本。覆盖上传 `image.tar` 不会删除部署机已导入的旧镜像，中央部署状态仍会保护活动镜像和可回滚镜像，因此 `micro_proxy rollback my_app` 不需要源码或旧归档。

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
- `package_type: source`（默认）需要 `micro-app.yml` 与 `Dockerfile`；`package_type: image` 需要 `micro-app.yml` 与已配置的镜像归档
- 目录名称将作为微应用的默认名称（`app.name`）
- 所有微应用的 `container_name` 必须全局唯一

### 名称推导逻辑

微应用有多个关键名称，它们的来源不同：

| 名称 | 来源 | 推导规则 | 示例 |
|------|------|----------|------|
| **app.name** | 目录名（自动推导） | 直接子目录：目录名即 `app.name`；嵌套目录：路径组件用 `_` 拼接 | `my_app`、`group_my_app` |
| **Docker 镜像引用** | 从 `app.name` 推导 | `{app.name}:sha-<哈希>` | `my_app:sha-0123456789ab` |
| **nginx 变量名** | 从 `app.name` 推导 | `{app.name}_upstream_host` | `$my_app_upstream_host` |
| **container_name** | 用户在 `micro-app.yml` 中配置 | 不推导，直接配置，需全局唯一 | `my-container` |

**推导链路：**

```
目录名 (my_app/)
  ├──→ app.name = "my_app"
  │      ├──→ Docker image = "my_app:sha-<哈希>"
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
    ├── Dockerfile             # Docker 构建文件（源码包必需）
    ├── nginx.conf             # Nginx 配置（SPA 应用必需）
    ├── micro-app.volumes.yml  # 数据持久化与权限配置（可选）
    ├── setup.sh               # 构建前脚本（可选）
    ├── clean.sh               # 清理脚本（可选）
    ├── .env                   # 环境变量（可选）
    └── src/                   # 源代码目录
```

镜像包目录可改为只包含 `micro-app.yml`、`image.tar`、可选 `.env` 和可选 `micro-app.volumes.yml`；不需要 Dockerfile、`src/`、`setup.sh` 或 `clean.sh`。

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
![公众号二维码](./assets/wechat-id.png)

我的个人网站: [craftaidhub.com](https://www.craftaidhub.com/)
