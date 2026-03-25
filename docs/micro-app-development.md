
# micro_proxy 应用开发指南

本指南详细介绍如何在 **micro_proxy** 环境中开发和配置三种类型的微应用：**Static**（静态）、**API**（服务）和 **Internal**（内部服务）。

---

## 📋 快速导航

| 章节 | 说明 |
|------|------|
| [配置文件总览](#配置文件总览) | 三种类型应用的必需/可选文件对照表 |
| [微应用配置文件](#微应用配置文件) | micro-app.yml 配置详解 |
| [卷配置文件](#卷配置文件) | micro-app.volumes.yml 配置详解 |
| [Static 类型应用](#static-类型应用) | 前端/静态网站配置规范 |
| [API 类型应用](#api-类型应用) | 后端 API 服务配置规范 |
| [Internal 类型应用](#internal-类型应用) | 数据库/内部服务配置规范 |
| [SPA 应用部署要点](#spa-应用部署要点) | 单页应用的关键配置检查项 |
| [网络与通信](#网络与通信) | 微应用间通信方式 |
| [自定义 Nginx 配置](#自定义-nginx-配置) | 高级路由规则定制 |
| [apps-config 自动生成说明](#apps-config-自动生成说明) | 动态配置文件的生成逻辑 |

---

## 配置文件总览

在使用 micro_proxy 开发应用前，请先确认你的项目属于以下哪种类型，并准备好对应的配置文件：

### 必需文件

| 文件 | 是否必需 | 说明 |
|------|:--------:|------|
| `micro-app.yml` | ✅ 必需 | 微应用配置文件，位于微应用根目录 |
| `Dockerfile` | ✅ 必需 | Docker 镜像构建文件，位于微应用根目录 |

### 可选文件

| 文件 | 是否必需 | 说明 |
|------|:--------:|------|
| `micro-app.volumes.yml` | ⚠️ 可选 | 卷和权限配置文件 |
| `.env` | ⚠️ 可选 | 环境变量文件 |
| `nginx.conf` | ⚠️ 条件 | SPA 部署时必需，自定义 Nginx 配置 |
| `setup.sh` | ⚠️ 可选 | 构建前执行脚本，可用于环境准备 |
| `clean.sh` | ⚠️ 可选 | 清理脚本，用于移除构建产物 |

### 类型选择速查表

| 特性 | Static | API | Internal |
|------|--------|-----|----------|
| 适用场景 | 前端/静态资源 | 后端服务 | Redis/MySQL等 |
| 对外暴露 | ✅ 是 | ✅ 是 | ❌ 否 |
| Nginx缓存 | ✅ 启用 | ❌ 禁用 | N/A |
| 访问方式 | `http://host/route` | `http://host/api` | `container-name:port` |
| routes 配置 | 必须 | 必须 | 空数组 [] |

---

## 微应用配置文件

每个微应用目录下必须包含 `micro-app.yml` 文件，用于配置该微应用的核心属性。

### 配置示例

```yaml
# micro-app.yml 示例
routes: ["/", "/api"]          # 访问路径（static/api 类型必需）
container_name: "my-container" # 容器名称（必需，全局唯一）
container_port: 80             # 容器内部端口（必需）
app_type: "static"             # 应用类型：static, api, internal（必需）
description: "应用描述"        # 可选
nginx_extra_config: |          # 额外的 nginx 配置（可选）
  add_header 'X-Custom-Header' 'value';
```

### 配置字段说明

| 字段 | 是否必需 | 说明 |
|------|----------|------|
| `routes` | 条件 | 访问路径，static/api 类型必需，internal 类型忽略 |
| `container_name` | ✅ | 容器名称，**全局唯一**，不能重复 |
| `container_port` | ✅ | 容器内部端口 |
| `app_type` | ✅ | 应用类型：static, api, internal |
| `description` | ❌ | 应用描述 |
| `nginx_extra_config` | ❌ | 额外的 nginx 配置（仅 static 和 api 有效） |

**注意：** `docker_volumes` 字段已从 `micro-app.yml` 中移除，现在使用独立的 `micro-app.volumes.yml` 文件进行配置。详见[卷配置文件](#卷配置文件)章节。

---

## 卷配置文件

`micro-app.volumes.yml` 是可选的配置文件，用于定义 Docker 卷挂载和权限设置。

### 配置示例

```yaml
# micro-app.volumes.yml 示例
volumes:
  - source: "./data"              # 宿主机路径
    target: "/data"               # 容器内路径
    permissions:                  # 可选：权限配置
      uid: 999                    # 用户ID
      gid: 999                    # 组ID
      recursive: true             # 是否递归设置权限

  - source: "./config"
    target: "/app/config"
    permissions:
      uid: 1000
      gid: 1000
      recursive: false

  - source: "./logs"
    target: "/var/log/app"       # 不配置权限则使用默认

run_as_user: "999:999"            # 可选：容器运行用户
```

### 配置字段说明

#### volumes

| 字段 | 是否必需 | 说明 |
|------|----------|------|
| `source` | ✅ | 宿主机路径（支持相对路径和绝对路径, 相对路径是相对于动态生成的docker-compose.yml的路径） |
| `target` | ✅ | 容器内路径 |
| `permissions` | ❌ | 权限配置对象 |

#### permissions

| 字段 | 是否必需 | 默认值 | 说明 |
|------|----------|--------|------|
| `uid` | ✅ | - | 用户ID |
| `gid` | ✅ | - | 组ID |
| `recursive` | ❌ | true | 是否递归设置权限 |

#### run_as_user

| 字段 | 是否必需 | 说明 |
|------|----------|------|
| `run_as_user` | ❌ | 容器运行用户，格式：`uid:gid` 或 `username` |

### 使用场景

1. **数据持久化**：将容器内的数据目录挂载到宿主机
   ```yaml
   volumes:
     - source: "./redis-data"
       target: "/data"
       permissions:
         uid: 999
         gid: 999
         recursive: true
   run_as_user: "999:999"
   ```

2. **配置文件共享**：将宿主机的配置文件挂载到容器中
   ```yaml
   volumes:
     - source: "./config"
       target: "/app/config"
       permissions:
         uid: 1000
         gid: 1000
         recursive: false
   ```

3. **日志输出**：将容器内的日志目录挂载到宿主机
   ```yaml
   volumes:
     - source: "./logs"
       target: "/var/log/app"
   ```

### 路径说明

- **相对路径**：相对于 `docker-compose.yml` 所在目录
- **绝对路径**：使用完整路径，如 `/var/data`

### 权限说明

`permissions` 配置用于在容器启动前自动设置宿主机目录的所有者和权限，确保容器内的进程可以正确访问挂载的目录。

#### 为什么需要权限配置？

容器和宿主机有独立的用户 ID 系统，容器内进程的 uid/gid 与宿主机上的 uid/gid 可能不同。如果两者不匹配，容器内进程可能无法访问宿主机挂载的目录。

**示例：**

```
# 问题场景
宿主机：./data 所有者 = uid 1000（你的用户）
容器内：nginx 进程 uid = 101

结果：nginx 无法写入 ./data ✗
```

#### uid/gid 设置策略

有两种策略确保权限匹配：

**策略 1：适应容器内用户（推荐使用官方镜像时）**

如果你的容器使用官方镜像（如 nginx、redis），容器内已有固定的用户 ID，那么：

- `permissions.uid/gid` 设置为容器内进程的 uid/gid
- 不配置 `run_as_user`（使用镜像默认用户）

```yaml
# Redis 官方镜像示例
volumes:
  - source: "./redis-data"
    target: "/data"
    permissions:
      uid: 999    # Redis 官方镜像使用的 uid
      gid: 999

# 不配置 run_as_user，使用 Redis 镜像的默认用户
```

**策略 2：适应宿主机用户（推荐自定义镜像时）**

如果你想容器内进程以宿主机用户身份运行，那么：

- `permissions.uid/gid` 设置为宿主机用户的 uid/gid
- `run_as_user` 设置为相同的 uid/gid

```yaml
# 自定义应用示例
volumes:
  - source: "./data"
    target: "/data"
    permissions:
      uid: 1000   # 宿主机用户 uid
      gid: 1000

run_as_user: "1000:1000"   # 容器内进程也以 uid 1000 运行
```

- **recursive**：为 true 时递归设置目录及其所有子目录/文件的权限

**注意事项：**
- 如果不需要挂载卷，可以只配置 `run_as_user` 来增强安全性
- 如果使用 `run_as_user`，建议同时配置 `permissions` 中的 uid/gid，两者应保持一致
- uid=0 或 gid=0（root 权限）会触发警告，存在安全风险
- 权限设置是在容器启动前完成的，因此需要在宿主机上有相应的权限（通常是 root 权限来修改目录所有者）

---

## Static 类型应用

适用于前端应用、静态网站等需要通过 Nginx 对外提供服务的场景。

### 📁 项目文件结构

```
micro-apps/craftaidhub_front/
├── micro-app.yml           # ✅ 必需 - 微应用配置
├── micro-app.volumes.yml   # ⚠️ 可选 - 卷和权限配置
├── Dockerfile              # ✅ 必需 - Docker 构建文件
├── nginx.conf              # ⚠️ SPA 应用必需 - 自定义 Nginx 配置
├── .env                    # ⚠️ 可选 - 环境变量
├── setup.sh                # ⚠️ 可选 - 构建前脚本
├── clean.sh                # ⚠️ 可选 - 清理脚本
└── dist/                   # 构建产物
    ├── index.html
    ├── assets/
    └── ...
```

### 1. micro-app.yml 配置示例

```yaml
# Static 类型微应用配置
routes: ["/"]                    # 访问路径
container_name: "craftaidhub_front"
container_port: 80
app_type: "static"
description: "主入口网站"
```

### 2. micro-app.volumes.yml 配置示例

```yaml
# 卷配置示例（可选）
volumes:
  - source: "./uploads"
    target: "/var/www/html/uploads"
    permissions:
      uid: 101
      gid: 101
      recursive: true

run_as_user: "nginx"              # 使用 nginx 用户运行
```

### 3. Dockerfile 配置示例

```dockerfile
# 多阶段构建示例 - Node.js + Vite 前端项目
FROM node:18-alpine as builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

# 运行阶段 - Nginx
FROM nginx:alpine
# 复制自定义 nginx 配置（SPA 必需）
COPY nginx.conf /etc/nginx/conf.d/default.conf
# 复制构建产物
COPY --from=builder /app/dist /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

### 4. nginx.conf 配置示例（SPA 应用必需）

```nginx
server {
    listen 80;
    server_name localhost;
    root /usr/share/nginx/html;
    index index.html;

    # SPA 路由回退 - 必须配置，否则刷新页面会 404
    location / {
        try_files $uri $uri/ /index.html;
    }

    # 静态资源缓存优化
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
```

---

## API 类型应用

适用于后端 API 服务、微服务等场景，请求路径会被完整传递到后端。

### 📁 项目文件结构

```
micro-apps/backend/
├── micro-app.yml           # ✅ 必需 - 微应用配置
├── micro-app.volumes.yml   # ⚠️ 可选 - 卷和权限配置
├── Dockerfile              # ✅ 必需 - Docker 构建文件
├── .env                    # ⚠️ 可选 - 环境变量
├── setup.sh                # ⚠️ 可选 - 构建前脚本
├── clean.sh                # ⚠️ 可选 - 清理脚本
└── src/                    # 源代码
    ├── server.js
    └── ...
```

### 1. micro-app.yml 配置示例

```yaml
# API 类型微应用配置
routes: ["/api"]                    # API 路由前缀
container_name: "resume-agent-backend"
container_port: 8080                # 后端应用监听端口
app_type: "api"
description: "简历智能体后端 API 服务"
nginx_extra_config: |
  # 处理 OPTIONS 预检请求
  if ($request_method = 'OPTIONS') {
    add_header 'Access-Control-Allow-Origin' '*';
    add_header 'Access-Control-Allow-Methods' 'GET, POST, OPTIONS';
    add_header 'Access-Control-Allow-Headers' 'DNT,User-Agent,X-Requested-With,If-Modified-Since,Cache-Control,Content-Type,Range,Authorization';
    add_header 'Access-Control-Max-Age' 1728000;
    add_header 'Content-Type' 'text/plain; charset=utf-8';
    add_header 'Content-Length' 0;
    return 204;
  }
```

### 2. micro-app.volumes.yml 配置示例

```yaml
# 日志持久化配置（可选）
volumes:
  - source: "./logs"
    target: "/app/logs"
    permissions:
      uid: 1000
      gid: 1000
      recursive: true

  - source: "./uploads"
    target: "/app/uploads"
    permissions:
      uid: 1000
      gid: 1000
      recursive: true

run_as_user: "1000:1000"           # 使用非 root 用户运行
```

### 3. Dockerfile 配置示例

```dockerfile
# Node.js API 服务示例
FROM node:18-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
EXPOSE 8080
CMD ["node", "server.js"]
```

### 路径处理说明

对于 API 类型，**请求路径会被完整保留**：

```
请求：http://host/api/v1/users
      ↓ (Nginx 转发)
后端接收：http://backend:8080/api/v1/users  ← 路径不变
```

---

## Internal 类型应用

适用于 Redis、MySQL 等仅用于内部通信的数据库服务，**不对外暴露**。

### 📁 项目文件结构

```
services/backend_redis/
├── micro-app.yml           # ✅ 必需 - 微应用配置
├── micro-app.volumes.yml   # ⚠️ 可选 - 卷和权限配置
├── Dockerfile              # ✅ 必需 - Docker 构建文件
├── data/                   # 数据持久化目录
├── logs/                   # 日志持久化目录
├── setup.sh                # ⚠️ 可选 - 构建前脚本
└── clean.sh                # ⚠️ 可选 - 清理脚本
```

### 1. micro-app.yml 配置示例

```yaml
# Internal 类型微应用配置
routes: []                           # Internal 类型 routes 必须为空
container_name: "backend_redis"
container_port: 6379                 # Redis 默认端口
app_type: "internal"
description: "Redis 缓存服务"
```

### 2. micro-app.volumes.yml 配置示例

```yaml
# Redis 数据和日志持久化配置
volumes:
  - source: "./data"
    target: "/data"
    permissions:
      uid: 999
      gid: 999
      recursive: true

  - source: "./logs"
    target: "/var/log/redis"
    permissions:
      uid: 999
      gid: 999
      recursive: true

run_as_user: "999:999"               # Redis 默认用户
```

### 3. Dockerfile 配置示例

```dockerfile
# Redis 服务示例
FROM redis:7-alpine
EXPOSE 6379
```

### 内部通信方式

其他应用可通过容器名称直接访问：

```bash
# 从后端访问 Redis
redis-cli -h backend_redis -p 6379

# 从后端访问 MongoDB
mongosh mongodb://resume_agent_mongodb:27017

# 容器间 HTTP 调用
curl http://backend_redis:6379
```

---

## SPA 应用部署要点

Static 类型的单页应用（Vue/React 等）需要特别注意以下配置：

### 核心检查清单

| 检查项 | 位置 | 要求 |
|--------|------|------|
| `try_files` | nginx.conf | 必须有 `$uri $uri/ /index.html` |
| `COPY nginx.conf` | Dockerfile | 必须复制自定义配置 |
| `BASE_URL` | .env | 子路径必须以 `/` 结尾 |
| 强制重建 | Docker | 修改.env 后需清除缓存 |

### 常见错误及修复

| 问题 | 原因 | 修复方法 |
|------|------|----------|
| 刷新页面 404 | 缺少 `try_files` | 在 nginx.conf 中添加 `try_files $uri $uri/ /index.html;` |
| 路径显示 `//path` | BASE_URL 缺少结尾斜杠 | `.env` 改为 `VITE_BASE_URL=/app/` |
| 配置未生效 | Docker 构建缓存 | `docker build --no-cache -t <name> .` |
| 权限错误 | 缺少权限配置 | 在 `micro-app.volumes.yml` 中配置 `permissions` |

### 权限配置建议

对于 SPA 应用，如果需要用户上传文件或写入数据，建议配置适当的权限：

```yaml
# micro-app.volumes.yml
volumes:
  - source: "./uploads"
    target: "/var/www/html/uploads"
    permissions:
      uid: 101          # nginx 用户的 UID
      gid: 101          # nginx 用户的 GID
      recursive: true   # 确保所有子目录都有正确的权限

run_as_user: "nginx"
```

---

## 网络与通信

### 对外访问流程

```
HTTP 请求 → Nginx (统一入口) → 对应容器
http://your-host.com/resume_app/     → resume_agent_front:80
http://your-host.com/api/v1/users    → backend:8080
```

### 内部通信流程

```
容器 A → 容器 B via Docker 网络
frontend → curl http://backend:8080/api
backend → redis-cli -h backend_redis -p 6379
backend → mongosh mongodb://resume_agent_mongodb:27017
```

---

## 自定义 Nginx 配置

可以通过 `nginx_extra_config` 为 Static/API 类型添加额外的 Nginx 指令：

```yaml
routes: ["/"]
app_type: "static"
nginx_extra_config: |
  # 添加自定义响应头
  add_header 'X-Custom-Header' 'value';
  
  # 代理特定路径到其他服务
  location /api {
    proxy_pass http://backend:3000;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
  }
```

**典型使用场景：**
- CORS 跨域配置
- 特殊路径转发
- 请求/响应头修改
- 限流配置

---

## apps-config 自动生成说明

`apps-config.yml` 是由 `micro_proxy` 自动生成的动态配置文件，**请勿手动修改**。该文件记录了所有扫描到的微应用的完整配置信息。

### 文件位置

在 `proxy-config.yml` 中通过 `apps_config_path` 配置：

```yaml
# proxy-config.yml
apps_config_path: "./apps-config.yml"
```

### 生成时机

`apps-config.yml` 在以下情况下会被重新生成：
1. 执行 `micro_proxy start` 命令时
2. 扫描目录中的微应用发生变化时（新增/删除/修改）

### 字段说明

以下是 `apps-config.yml` 中各字段的含义和生成逻辑：

```yaml
# 此文件由 micro_proxy 自动生成，请勿手动修改
apps:
  - name: "apps_craftaidhub_front"      # 🔹 动态生成
    routes:
      - "/"
    container_name: "craftaidhub_front"
    container_port: 80
    app_type: "static"
    description: "主入口网站"
    path: "/home/ubuntu/apps/craftaidhub_front"  # 🔹 动态生成
    docker_volumes: []
    run_as_user: null                # 🔹 从 micro-app.volumes.yml 加载
  - name: "backend"
    routes:
      - "/api"
    container_name: "resume-agent-backend"
    container_port: 8080
    app_type: "api"
    description: "简历智能体后端 API 服务"
    nginx_extra_config: |
      # 处理 OPTIONS 预检请求
      if ($request_method = 'OPTIONS') {
        add_header 'Access-Control-Allow-Origin' '*';
        ...
      }
    path: "/home/ubuntu/apps/resume-agent/backend"  # 🔹 动态生成
    docker_volumes:
      - "./logs:/app/logs"        # 🔹 从 micro-app.volumes.yml 转换
    run_as_user: "1000:1000"      # 🔹 从 micro-app.volumes.yml 加载
```

#### 🔹 name 字段生成逻辑

`name` 字段是微应用的**唯一标识**，由 `micro_proxy` 根据以下规则自动生成：

**生成公式：**
```
name = {scan_dir 相对路径}_{最后一级目录名}
```

**具体规则：**

| 场景 | scan_dir | 应用目录 | 生成的 name |
|------|----------|----------|-------------|
| 直接子目录 | `/home/ubuntu` | `/home/ubuntu/craftaidhub_front` | `craftaidhub_front` |
| 一级嵌套 | `/home/ubuntu` | `/home/ubuntu/apps/craftaidhub_front` | `apps_craftaidhub_front` |
| 多级嵌套 | `/home/ubuntu` | `/home/ubuntu/services/backend/api` | `services_backend_api` |

**设计目的：**
- **解决命名冲突**：当多个扫描目录中存在同名目录时，通过相对路径区分
- **保持可读性**：相比哈希值，使用路径层级更易于识别和管理
- **长度可控**：相比完整路径，使用下划线连接的方式更简洁

#### 🔹 path 字段生成逻辑

`path` 字段是微应用在**宿主机上的绝对路径**，由 `micro_proxy` 自动记录：

**生成规则：**
```
path = 微应用目录的绝对路径
```

**用途：**
1. **Docker 镜像构建**：作为构建上下文（build context）
2. **Volume 挂载参考**：用于验证 docker_volumes 配置的相对路径基准
3. **状态管理**：基于路径 hash 判断是否需要重新构建

**注意：**
- `path` 字段是**只读的**，不应在 `micro-app.yml` 中手动配置
- 如果需要修改微应用位置，请移动目录后重新扫描

#### 🔹 docker_volumes 字段生成逻辑

`docker_volumes` 字段由 `micro_proxy` 从 `micro-app.volumes.yml` 自动转换生成：

**转换规则：**
```
docker_volumes = volumes.map(|v| format!("{}:{}", v.source, v.target))
```

**示例：**

```yaml
# micro-app.volumes.yml
volumes:
  - source: "./data"
    target: "/app/data"
  - source: "./config"
    target: "/etc/app"
```

转换为：

```yaml
# apps-config.yml
docker_volumes:
  - "./data:/app/data"
  - "./config:/etc/app"
```

#### 🔹 run_as_user 字段生成逻辑

`run_as_user` 字段直接从 `micro-app.volumes.yml` 中复制：

```yaml
# micro-app.volumes.yml
run_as_user: "999:999"
```

转换为：

```yaml
# apps-config.yml
run_as_user: "999:999"
```

### 与其他配置文件的关系

```
┌─────────────────┐     扫描发现      ┌──────────────────┐
│  micro-app.yml  │ ───────────────→ │  apps-config.yml │
│  (手动编写)      │                  │  (自动生成)       │
└─────────────────┘                  └──────────────────┘
       │                                    │
       │  读取配置                            │  生成 nginx 配置
       ▼                                    ▼
┌─────────────────┐                  ┌──────────────────┐
│  Dockerfile     │                  │    nginx.conf    │
│  (手动编写)      │                  │  (自动生成)       │
└─────────────────┘                  └──────────────────┘
       │
       │ 读取配置
       ▼
┌─────────────────┐
│micro-app.       │
│volumes.yml      │
│(手动编写，可选)  │
└─────────────────┘
```

### 常见问题

**Q: 我可以手动修改 apps-config.yml 吗？**

A: **不建议**。该文件会在每次启动时被重新生成，手动修改会被覆盖。如需调整配置，请修改 `micro-app.yml` 或 `micro-app.volumes.yml`。

**Q: name 字段冲突了怎么办？**

A: 通过调整微应用的目录结构来解决。例如，将同名的微应用移动到不同的父目录下：
```
# 冲突前
apps/
  ├── backend/
  └── backend/  # 冲突！

# 冲突后
apps/
  ├── frontend_backend/
  └── payment_backend/
```

**Q: 我没有 micro-app.volumes.yml 文件会怎样？**

A: 没有任何问题。`micro-app.volumes.yml` 是可选文件，如果不存在，`apps-config.yml` 中的 `docker_volumes` 将为空数组，`run_as_user` 将为 null。

**Q: 如何知道应用以什么用户运行？**

A: 查看 `apps-config.yml` 中的 `run_as_user` 字段，如果为 null 则使用镜像的默认用户（通常为 root）。

**Q: run_as_user 和 permissions 必须一起配置吗？**

A: 不必。两者可以独立使用：
- 只配置 `run_as_user`：用于增强安全性，容器内不以 root 运行
- 只配置 `permissions`：使用镜像默认用户，但提前设置宿主机目录权限
- 两者都配置：确保容器内用户与宿主机目录所有者一致

**相关文档：**
- [主文档 - 快速开始](../README.md#快速开始)
- [配置说明](../README.md#配置说明)
- [ACME.sh 安装指南](acme-installation.md)
- [SSL 证书申请指南](certificate-application.md)
- [卷配置重构方案](micro-app-volumes-refactor-plan.md)
