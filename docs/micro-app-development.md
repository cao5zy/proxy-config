# micro_proxy 应用开发指南

本指南详细介绍如何在 **micro_proxy** 环境中开发和配置三种类型的微应用：**Static**（静态）、**API**（服务）和 **Internal**（内部服务）。

---

## 📋 快速导航

| 章节 | 说明 |
|------|------|
| [配置文件总览](#配置文件总览) | 三种类型应用的必需/可选文件对照表 |
| [Static 类型应用](#static-类型应用) | 前端/静态网站配置规范 |
| [API 类型应用](#api-类型应用) | 后端 API 服务配置规范 |
| [Internal 类型应用](#internal-类型应用) | 数据库/内部服务配置规范 |
| [SPA 应用部署要点](#spa-应用部署要点) | 单页应用的关键配置检查项 |
| [网络与通信](#网络与通信) | 微应用间通信方式 |
| [自定义 Nginx 配置](#自定义-nginx-配置) | 高级路由规则定制 |

---

## 配置文件总览

在使用 micro_proxy 开发应用前，请先确认你的项目属于以下哪种类型，并准备好对应的配置文件：

### Static / API 类型（需通过 Nginx 对外暴露）

| 文件 | 是否必需 | 说明 |
|------|:--------:|------|
| `Dockerfile` | ✅ 必需 | 必须放在微应用根目录，用于构建镜像 |
| `.env` | ⚠️ 可选 | 环境变量文件，构建时注入到容器 |
| `nginx.conf` | ⚠️ 可选(推荐) | 自定义 Nginx 配置，**SPA 应用必需** |
| `setup.sh` | ⚠️ 可选 | 构建前的准备脚本 |
| `clean.sh` | ⚠️ 可选 | 清理构建产物的脚本 |

### Internal 类型（仅内部通信）

| 文件 | 是否必需 | 说明 |
|------|:--------:|------|
| `Dockerfile` | ✅ 必需 | 必须放在微应用根目录，用于构建镜像 |
| `.env` | ⚠️ 可选 | 环境变量文件 |
| `setup.sh` | ⚠️ 可选 | 构建前的准备脚本 |
| `clean.sh` | ⚠️ 可选 | 清理构建产物的脚本 |

### 类型选择速查表

| 特性 | Static | API | Internal |
|------|--------|-----|----------|
| 适用场景 | 前端/静态资源 | 后端服务 | Redis/MySQL等 |
| 对外暴露 | ✅ 是 | ✅ 是 | ❌ 否 |
| Nginx缓存 | ✅ 启用 | ❌ 禁用 | N/A |
| 访问方式 | `http://host/route` | `http://host/api` | `container-name:port` |
| routes 配置 | 必须 | 必须 | 空数组 [] |

---

## Static 类型应用

适用于前端应用、静态网站等需要通过 Nginx 对外提供服务的场景。

### 📁 项目文件结构

```
micro-apps/craftaidhub_front/
├── Dockerfile          # ✅ 必需
├── nginx.conf          # ⚠️ SPA应用必需
├── .env                # ⚠️ 可选
├── setup.sh            # ⚠️ 可选
├── clean.sh            # ⚠️ 可选
└── dist/               # 构建产物
    ├── index.html
    ├── assets/
    └── ...
```

### 1. Dockerfile 配置示例

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
# 复制自定义 nginx 配置（SPA必需）
COPY nginx.conf /etc/nginx/conf.d/default.conf
# 复制构建产物
COPY --from=builder /app/dist /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

### 2. nginx.conf 配置示例（SPA 应用必需）

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

### 3. proxy-config.yml 配置示例

```yaml
apps:
  # 主入口网站 - 根路径
  - name: "craftaidhub_front"
    routes: ["/"]                     # 支持多个路径
    container_name: "craftaidhub_front"
    container_port: 80
    app_type: "static"
    description: "主入口网站"

  # 子路径前端应用
  - name: "resume_agent_front"
    routes: ["/resume_app"]           # 访问 http://host/resume_app/
    container_name: "resume-agent-frontend"
    container_port: 80
    app_type: "static"
    description: "简历智能体前端应用"
```

### 关键配置说明

| YAML 字段 | 必填 | 说明 |
|-----------|------|------|
| `name` | ✅ | 应用唯一标识名 |
| `routes` | ✅ | URL 路由路径，数组形式 |
| `container_name` | ✅ | Docker 容器名称 |
| `container_port` | ✅ | 容器内部端口（通常是 80） |
| `app_type` | ✅ | 固定为 `"static"` |
| `description` | ⚠️ | 应用描述信息 |

---

## API 类型应用

适用于后端 API 服务、微服务等场景，请求路径会被完整传递到后端。

### 📁 项目文件结构

```
micro-apps/backend/
├── Dockerfile          # ✅ 必需
├── .env                # ⚠️ 可选
├── setup.sh            # ⚠️ 可选
├── clean.sh            # ⚠️ 可选
└── src/                # 源代码
    ├── server.js
    └── ...
```

### 1. Dockerfile 配置示例

```dockerfile
# Node.js API 服务示例
FROM node:18-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
EXPOSE 8080
CMD ["node", "server.js"]

# 或者 Python 示例
# FROM python:3.11-slim
# WORKDIR /app
# COPY requirements.txt .
# RUN pip install -r requirements.txt
# COPY . .
# EXPOSE 8080
# CMD ["python", "app.py"]
```

### 2. proxy-config.yml 配置示例

```yaml
apps:
  # 后端 API 服务
  - name: "backend"
    routes: ["/api"]                    # 所有 /api/* 请求转发到此服务
    container_name: "resume-agent-backend"
    container_port: 8080                # 后端应用监听端口
    app_type: "api"
    description: "简历智能体后端 API 服务"
    
    # 额外的 nginx 配置，处理 CORS 预检请求
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

  # 另一个 API 服务（知音项目）
  - name: "zhiying_backend"
    routes: ["/zhiyingapi"]
    container_name: "zhiying-backend"
    container_port: 8080
    app_type: "api"
    description: "OpenClaw 后端 API 服务"
```

### 关键配置说明

| YAML 字段 | 必填 | 说明 |
|-----------|------|------|
| `name` | ✅ | 应用唯一标识名 |
| `routes` | ✅ | API 路由前缀，如 `/api` |
| `container_name` | ✅ | Docker 容器名称 |
| `container_port` | ✅ | 后端服务实际端口 |
| `app_type` | ✅ | 固定为 `"api"` |
| `nginx_extra_config` | ⚠️ | 额外 Nginx 配置（CORS、重写等） |

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
├── Dockerfile          # ✅ 必需
├── data/               # 数据持久化目录
├── logs/               # 日志持久化目录
├── setup.sh            # ⚠️ 可选
└── clean.sh            # ⚠️ 可选
```

### 1. Dockerfile 配置示例

```dockerfile
# Redis 服务示例
FROM redis:7-alpine
EXPOSE 6379

# MongoDB 服务示例
# FROM mongo:6
# EXPOSE 27017

# MySQL 服务示例
# FROM mysql:8
# EXPOSE 3306
```

### 2. proxy-config.yml 配置示例

```yaml
apps:
  # Redis 缓存服务
  - name: "backend_redis"
    routes: []                           # Internal 类型 routes 必须为空
    container_name: "backend_redis"
    container_port: 6379                 # Redis 默认端口
    app_type: "internal"
    description: "Redis 缓存服务"
    path: "/home/ubuntu/apps/resume-agent/backend_redis"  # ⚠️ 必须配置服务文件夹路径

  # MongoDB 存储服务
  - name: "resume_agent_mongodb"
    routes: []                           # Internal 类型 routes 必须为空
    container_name: "resume_agent_mongodb"
    container_port: 27017                # MongoDB 默认端口
    app_type: "internal"
    description: "Mongodb 存储"
    path: "/home/ubuntu/apps/resume-agent/resume_agent_mongodb"
    
    # 数据持久化配置
    docker_volumes:
      - "./resume_agent_mongodb/data:/data/db"       # 数据目录挂载
      - "./resume_agent_mongodb/logs:/var/log/mongodb" # 日志目录挂载
```

### 关键配置说明

| YAML 字段 | 必填 | 说明 |
|-----------|------|------|
| `name` | ✅ | 服务唯一标识名 |
| `routes` | ✅ | **必须为空数组 `[]`** |
| `container_name` | ✅ | Docker 容器名称 |
| `container_port` | ✅ | 服务端口（Redis:6379, MongoDB:27017等） |
| `app_type` | ✅ | 固定为 `"internal"` |
| `path` | ✅ | **服务文件夹路径** |
| `docker_volumes` | ⚠️ | 数据/日志持久化挂载 |

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

Static 类型的单页应用（Vue/React等）需要特别注意以下配置：

### 核心检查清单

| 检查项 | 位置 | 要求 |
|--------|------|------|
| `try_files` | nginx.conf | 必须有 `$uri $uri/ /index.html` |
| `COPY nginx.conf` | Dockerfile | 必须复制自定义配置 |
| `BASE_URL` | .env | 子路径必须以 `/` 结尾 |
| 强制重建 | Docker | 修改.env后需清除缓存 |

### 常见错误及修复

| 问题 | 原因 | 修复方法 |
|------|------|----------|
| 刷新页面 404 | 缺少 `try_files` | 在 nginx.conf 中添加 `try_files $uri $uri/ /index.html;` |
| 路径显示 `//path` | BASE_URL 缺少结尾斜杠 | `.env` 改为 `VITE_BASE_URL=/app/` |
| 配置未生效 | Docker 构建缓存 | `docker build --no-cache -t <name> .` |

---

## 网络与通信

### 对外访问流程

```
HTTP请求 → Nginx (统一入口) → 对应容器
http://your-host.com/resume_app/     → resume_agent_front:80
http://your-host.com/api/v1/users    → backend:8080
```

### 内部通信流程

```
容器A → 容器B via Docker 网络
frontend → curl http://backend:8080/api
backend → redis-cli -h backend_redis -p 6379
backend → mongosh mongodb://resume_agent_mongodb:27017
```

---

## 自定义 Nginx 配置

可以通过 `nginx_extra_config` 为 Static/API 类型添加额外的 Nginx 指令：

```yaml
- name: "main-app"
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

**相关文档：**
- [主文档 - 快速开始](../README.md#快速开始)
- [配置说明](../README.md#配置说明)
- [ACME.sh 安装指南](acme-installation.md)
- [SSL 证书申请指南](certificate-application.md)
