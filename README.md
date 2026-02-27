

# micro_proxy

[![Crates.io](https://img.shields.io/crates/v/micro_proxy)](https://crates.io/crates/micro_proxy)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)

一个用于管理微应用的工具，支持Docker镜像构建、容器管理、Nginx反向代理配置等功能。

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

所有应用通过Nginx统一入口访问，默认端口为80（可在 `micro_proxy.yml` 配置文件中的 `nginx_host_port` 字段修改）：

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
nginx_host_port: 80

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

## 微应用开发专题

### 微应用目录结构

#### Static 和 Api 类型

Static 和 Api 类型的微应用目录结构：

```
micro-apps/
├── main-app/              # 文件夹名称即为微应用名称
│   ├── Dockerfile         # 必须在项目根目录
│   ├── .env               # 环境变量文件
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

**关键要求：**
- 每个微应用必须是一个独立的文件夹
- 文件夹名称即为微应用的名称
- Dockerfile必须放在项目根目录
- .env文件用于定义环境变量（可选）
- setup.sh和clean.sh脚本放在项目根目录（可选）

#### Internal 类型

Internal 类型的微应用目录结构（如 Redis、MySQL 等）：

```
services/
├── redis/                 # 服务文件夹
│   ├── Dockerfile         # 必须在项目根目录
│   ├── .env               # 环境变量文件
│   ├── setup.sh           # 可选：预构建脚本
│   ├── clean.sh           # 可选：清理脚本
│   └── ...                # 其他服务文件
└── mysql/
    ├── Dockerfile
    ├── .env
    └── ...
```

**关键要求：**
- 服务文件夹可以放在任意位置（通过配置文件的 path 字段指定）
- Dockerfile必须放在项目根目录
- .env文件用于定义环境变量（可选）
- setup.sh和clean.sh脚本放在项目根目录（可选）
- 不需要通过 nginx 访问，仅用于微应用间内部通信

### Dockerfile要求

- Dockerfile必须放在项目根目录
- 建议使用EXPOSE指令声明暴露的端口
- 如果没有EXPOSE指令，micro_proxy会发出警告

示例：

```dockerfile
FROM nginx:alpine

EXPOSE 80

COPY . /usr/share/nginx/html
```

### 环境变量配置

在`.env`文件中定义环境变量，这些变量会在构建时传递给Docker：

```env
APP_PORT=80
ENV=production
DEBUG=false
```

**注意：**
- .env文件是可选的
- 如果没有.env文件，micro_proxy仍然会构建镜像，但不会传递环境变量
- 环境变量只在构建时传递，运行时环境变量需要在Dockerfile中定义

### 脚本支持

micro_proxy支持在微应用中使用脚本来自动化构建和清理过程。

#### setup.sh - 预构建脚本

在构建镜像前执行，用于准备构建环境：

```bash
#!/bin/bash
echo "Running setup..."
# 执行一些准备工作，如安装依赖、构建前端等
npm install
npm run build
```

#### clean.sh - 清理脚本

在清理时执行，用于清理构建产物：

```bash
#!/bin/bash
echo "Running clean..."
# 执行清理工作，如删除构建产物、清理缓存等
rm -rf dist/
rm -rf node_modules/
```

**脚本执行时机：**
- setup.sh在每次构建镜像前执行
- clean.sh在执行`micro_proxy clean`命令时执行

### 应用类型

micro_proxy支持三种应用类型，影响Nginx配置和容器配置的生成：

#### static - 静态网站

适用于前端应用、静态页面等，启用缓存：

```yaml
app_type: "static"
```

**特性：**
- 启用浏览器缓存
- 适合静态资源
- 响应头包含缓存控制
- 需要配置 routes（访问路径）
- 需要通过 nginx 反向代理访问

#### api - API服务

适用于后端API服务，禁用缓存：

```yaml
app_type: "api"
```

**特性：**
- 禁用浏览器缓存
- 适合动态内容
- 支持自定义nginx配置
- 需要配置 routes（访问路径）
- 需要通过 nginx 反向代理访问

#### internal - 内部服务

适用于Redis、MySQL等不需要nginx代理的内部服务：

```yaml
app_type: "internal"
path: "./services/redis"  # 必须配置，指向服务文件夹路径
```

**特性：**
- 不需要 nginx 反向代理
- 不需要配置 routes（routes 为空）
- 不添加 HTTP 健康检查（可能不是 HTTP 服务）
- 仅用于微应用间内部通信
- 必须配置 path 字段，指向包含 Dockerfile 的文件夹路径
- 可以被其他微应用通过容器名称访问

**使用场景：**
- Redis 缓存服务
- MySQL 数据库服务
- MongoDB 数据库服务
- 其他不需要对外暴露的服务

### 反向代理配置约定

micro_proxy 为 Static 和 Api 类型的微应用生成 Nginx 反向代理配置时，遵循以下约定：

#### Static 类型的反向代理约定

Static 类型适用于前端应用、静态页面等，其反向代理行为如下：

**根路径配置（route: "/"）**
- 直接转发请求，不修改 URI
- 示例：访问 `/index.html` → 后端收到 `/index.html`
- 配置示例：
  ```nginx
  location / {
      proxy_pass http://${app-name_upstream_host}:80;
  }
  ```

**非根路径配置（route: "/resume_app"）**
- 使用 rewrite 规则移除路径前缀
- 示例：
  - 访问 `/resume_app` → 后端收到 `/`
  - 访问 `/resume_app/` → 后端收到 `/`
  - 访问 `/resume_app/assets/style.css` → 后端收到 `/assets/style.css`
- 配置示例：
  ```nginx
  location /resume_app {
      rewrite ^/resume_app(/.*)?$ /$1 break;
      proxy_pass http://${app-name_upstream_host}:80;
  }
  ```

**适用场景：**
- 前端应用部署在子路径下（如 VITE_BASE_URL=/resume_app）
- 静态资源服务
- SPA（单页应用）

#### Api 类型的反向代理约定

Api 类型适用于后端 API 服务，其反向代理行为如下：

**保留完整路径**
- 不修改请求 URI，后端收到完整的请求路径
- 示例：访问 `/api/v1/status` → 后端收到 `/api/v1/status`
- 配置示例：
  ```nginx
  location /api {
      proxy_pass http://${app-name_upstream_host}:8080;
  }
  ```

**重要说明：**
- `proxy_pass` 不以 `/` 结尾，确保后端收到完整的请求路径
- 如果后端服务期望接收 `/api` 前缀，请确保后端路由配置正确
- 如果后端服务不期望 `/api` 前缀，请使用 Static 类型的配置方式

**适用场景：**
- RESTful API 服务
- GraphQL 服务
- 后端微服务

#### 路径重写对比

| 类型 | 访问路径 | 后端收到路径 | 说明 |
|------|---------|------------|------|
| Static (/) | `/index.html` | `/index.html` | 根路径直接转发 |
| Static (/app) | `/app/index.html` | `/index.html` | 移除前缀 |
| Api (/api) | `/api/v1/status` | `/api/v1/status` | 保留完整路径 |

#### 配置建议

**前端应用（Static）**
```yaml
- name: "frontend"
  routes: ["/app"]
  app_type: "static"
  container_port: 80
```
- 前端应用通常部署在子路径下
- 使用 rewrite 规则移除路径前缀
- 前端路由配置应匹配子路径（如 VITE_BASE_URL=/app）

**后端 API（Api）**
```yaml
- name: "backend"
  routes: ["/api"]
  app_type: "api"
  container_port: 8080
```
- 后端 API 通常需要保留完整路径
- 后端路由应包含 `/api` 前缀（如 `/api/v1/users`）
- 如果后端不期望 `/api` 前缀，请考虑使用 Static 类型或调整后端路由

### 网络通信

所有微应用共享同一个Docker网络，可以通过服务名称相互访问：

```bash
# main-app访问api-service
curl http://api-service:3000/api

# api-service访问main-app
curl http://main-app:80/

# api-service访问redis（内部服务）
redis-cli -h redis -p 6379
```

**网络规则：**
- 所有容器都在同一个Docker网络中
- 容器可以通过容器名称相互访问
- 端口映射由micro_proxy自动管理
- Static 和 Api 类型的应用通过 nginx 统一入口对外访问
- Internal 类型的应用仅用于内部通信，不对外暴露

### 反向代理配置

每个 Static 和 Api 类型的微应用都可以配置多个访问路径：

```yaml
apps:
  - name: "main-app"
    routes: ["/", "/app"]          # 多个访问路径
    container_port: 80             # 容器内部端口
    app_type: "static"
```

**路由规则：**
- 所有请求通过Nginx统一入口（默认80端口）
- 根据routes配置将请求转发到对应容器
- 支持路径重写和自定义配置
- Internal 类型的应用不需要配置 routes

### 自定义Nginx配置

可以为每个 Static 和 Api 类型的应用添加额外的nginx配置：

```yaml
apps:
  - name: "main-app"
    routes: ["/"]
    nginx_extra_config: |
      add_header 'X-Custom-Header' 'value';
      location /api {
        proxy_pass http://backend:3000;
      }
```

**注意：**
- nginx_extra_config 仅对 Static 和 Api 类型有效
- Internal 类型不需要也不应该配置 nginx_extra_config

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

## 项目结构

```
proxy-config/
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
