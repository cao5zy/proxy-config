
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
# 访问路径（static/api类型必需）
routes: ["/", "/api"]

# Docker容器名称（必需，全局唯一）
container_name: "my-container"

# 容器内部端口（必需）
container_port: 80

# 应用类型（必需）：static, api, internal
app_type: "static"

# 应用描述（可选）
description: "应用描述"

# Docker volumes 映射（可选）
docker_volumes:
  - "./data:/app/data"           # 读写挂载
  - "./config:/app/config:ro"    # 只读挂载

# 额外的 nginx 配置（可选，仅 static 和 api 有效）
nginx_extra_config: |
  add_header 'X-Custom-Header' 'value';
```

**详细配置说明**请参阅 **[微应用开发专题](docs/micro-app-development.md)**。

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

## 微应用开发

关于微应用开发的详细说明，请参阅 **[微应用开发专题](docs/micro-app-development.md)**。

### 应用类型简介

micro_proxy 支持三种应用类型：

| 类型 | 说明 | 访问方式 |
|------|------|----------|
| **Static** | 静态应用（前端页面） | 通过 Nginx 反向代理对外服务 |
| **API** | API 服务（后端接口） | 通过 Nginx 反向代理对外服务 |
| **Internal** | 内部服务（数据库等） | 仅用于微应用间内部通信 |

### 标准微应用目录结构

```
micro-apps/
└── my-app/                    # 微应用目录
    ├── micro-app.yml          # 微应用配置文件（必需）
    ├── Dockerfile             # Docker 构建文件（必需）
    ├── nginx.conf             # Nginx 配置（SPA 应用必需）
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

```bash
# 检查宿主机路径是否存在
ls -la ./data

# 检查容器内的挂载点
docker exec <container-name> ls -la /app/data

# 查看容器详细信息
docker inspect <container-name> | grep -A 10 Mounts
```

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
