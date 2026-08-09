---
name: micro-app-development
description: >
  Use when creating, configuring, or deploying micro-apps on the micro_proxy
  platform. Covers micro-app.yml, Dockerfile, volume configuration, SPA
  deployment, Nginx customization, and all three app types (Static, API,
  Internal). Triggered by questions about micro-app creation, configuration
  files, app types, volume permissions, SPA 404 errors, Docker build,
  internal services, and Nginx setup.
---

# micro-app-development

## Purpose

本文档为 AI 编程助手（如 Qoder、OpenCode、Claude Code、Cursor、Windsurf、GitHub Copilot、Cline 等）提供 **micro_proxy** 平台微应用开发所需的完整知识库和行为指南。AI 在回答用户关于 micro-app 创建、配置、部署相关问题时，应参考本文档的内容。

## Platform Compatibility

本文档使用标准 Markdown 编写，结构清晰、分段明确，可直接被以下平台作为上下文或指令文件使用：

| Platform | Usage |
|----------|-------|
| **Qoder** | 放置为 `SKILL.md`，按 `When to Activate` 章节自动触发 |
| **OpenCode** | 放置为 `.opencode/skills/<name>/SKILL.md`，自动发现 |
| **Claude Code** | 引用为 `CLAUDE.md` 或通过 `-p` 参数加载 |
| **Cursor** | 放置为 `.cursorrules` 或项目规则 |
| **Windsurf** | 放置为 `.windsurfrules` |
| **GitHub Copilot** | 引用 `.github/copilot-instructions.md` |
| **Cline** | 放置为 `.clinerules` |
| **Any AI** | 直接粘贴到对话中作为上下文 |

---

## When to Activate

AI 检测到用户输入涉及以下主题时，应自动调用本文档中的知识：

| Trigger | User might say |
|---------|---------------|
| Creating a new micro-app | "I want to create a new micro-app" / "How to set up a micro-app" |
| Configuration files | "How to configure micro-app.yml" / "What is the routes field" |
| App type selection | "What's the difference between Static and API" / "Which type should I use" |
| Volume / permissions | "How to set volume permissions" / "Permission denied error in container" |
| SPA deployment | "Page refresh returns 404" / "Deploying Vue/React app" / "SPA routing" |
| Docker build | "How to write Dockerfile" / "Build cache issue" |
| Internal services | "How to deploy Redis" / "Container-to-container communication" |
| Nginx configuration | "How to add custom Nginx config" / "CORS setup" |
| General inquiry | "How to develop micro-apps" / "Best practices for micro-apps" |

---

## Core Knowledge

### 1. Application Types

micro_proxy 支持三种微应用类型，这是所有配置的基础：

| Type | Use Case | External Access | Nginx Cache | Access Method |
|------|----------|:---------------:|:-----------:|--------------|
| **Static** | Frontend / Static website | Yes | Enabled | `http://host/<route>` |
| **API** | Backend API service | Yes | Disabled | `http://host/<route>` |
| **Internal** | Database / Middleware / Internal HTTP services | Conditional | Disabled | `container-name:port` or `http://host/<route>` |

**Decision guide:**
- User has HTML/JS/CSS or Vue/React project → **Static**
- User has backend code (Node.js/Python/Go/Java) providing HTTP API → **API**
- User needs Redis, MySQL, MongoDB, etc. (no HTTP exposure) → **Internal** (no routes)
- User needs MinIO, Adminer, or other internal HTTP service → **Internal** (with routes)
- Same project has both frontend and backend → Two separate micro-apps

> **Internal with routes:** Some internal services (like MinIO, Adminer, pgAdmin) serve HTTP but are not user-facing API backends. Set `app_type: "internal"` with `routes: ["/path"]` to expose them through nginx. The route prefix is **automatically stripped** (like Static type), because third-party services cannot handle path prefixes.

### 2. File Structure

Every micro-app requires these files in its root directory:

**Required:**

| File | Description |
|------|-------------|
| `micro-app.yml` | Core configuration: routes, container name, port, type |
| `Dockerfile` | Docker image build instructions |

**Optional:**

| File | Description | When Needed |
|------|-------------|-------------|
| `micro-app.volumes.yml` | Volume mounts and permissions | Data persistence required |
| `nginx.conf` | Custom Nginx configuration | **Required** for SPA (prevents 404 on refresh) |
| `.env` | Build-time environment variables | Passing VITE_BASE_URL etc. |
| `setup.sh` | Pre-build script | Additional setup before build |
| `clean.sh` | Cleanup script | Removing build artifacts |

### 3. micro-app.yml Schema

**Fields:**

| Field | Required | Type | Description |
|-------|----------|------|-------------|
| `routes` | Conditional | `string[]` | Access paths. Static/API: must have ≥1 route. Internal: optional (empty=no proxy, ≥1 route=expose via nginx). 不能以 `/` 结尾（根路由 `/` 除外） |
| `container_name` | Yes | `string` | Container name, **globally unique** across all micro-apps |
| `container_port` | Yes | `int` | Container internal port (1-65535) |
| `app_type` | Yes | `enum` | One of: `static`, `api`, `internal` |
| `description` | No | `string` | App description |
| `nginx_extra_config` | No | `string` | Extra Nginx directives (static/api/internal with routes) |

**Example:**

```yaml
routes: ["/", "/api"]
container_name: "my-container"
container_port: 80
app_type: "static"
description: "Application description"
nginx_extra_config: |
  add_header 'X-Custom-Header' 'value';
```

**Note:** The `docker_volumes` field has been removed from `micro-app.yml`. Use the separate `micro-app.volumes.yml` file instead.

### 4. micro-app.volumes.yml Schema

```yaml
volumes:
  - source: "./data"         # Host path (relative or absolute)
    target: "/data"          # Container path
    permissions:             # Optional
      uid: 999               # User ID
      gid: 999               # Group ID
      recursive: true        # Recursively set permissions

run_as_user: "999:999"       # Optional: container runtime user (format: "uid:gid" or "username")
```

**volumes array:**

| Field | Required | Description |
|-------|----------|-------------|
| `source` | Yes | Host path. Relative paths are relative to the generated `docker-compose.yml` location |
| `target` | Yes | Container-internal path |
| `permissions` | No | Permission configuration object |

**permissions object:**

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| `uid` | Yes | - | User ID |
| `gid` | Yes | - | Group ID |
| `recursive` | No | `true` | Whether to set permissions recursively |

**run_as_user:**

| Field | Required | Description |
|-------|----------|-------------|
| `run_as_user` | No | Container runtime user, format: `uid:gid` or `username` |

### 5. Permission Strategy

Containers and hosts have independent UID/GID systems. If they don't match, the container process may not access mounted directories.

**Strategy A: Adapt to container user** (recommended for official images)

If using official images (nginx, redis) with fixed internal UIDs:

- Set `permissions.uid/gid` to the container process's UID/GID
- Do NOT configure `run_as_user` (use image default)

```yaml
volumes:
  - source: "./redis-data"
    target: "/data"
    permissions:
      uid: 999    # Redis official image UID
      gid: 999
# No run_as_user - use Redis image default
```

**Strategy B: Adapt to host user** (recommended for custom images)

If you want the container process to run as the host user:

- Set `permissions.uid/gid` to the host user's UID/GID
- Set `run_as_user` to the same UID/GID

```yaml
volumes:
  - source: "./data"
    target: "/data"
    permissions:
      uid: 1000   # Host user UID
      gid: 1000
run_as_user: "1000:1000"  # Container also runs as UID 1000
```

**Important notes:**
- `uid=0` or `gid=0` (root) triggers a security warning
- If no volumes are needed, you can still configure `run_as_user` alone for security hardening
- If using `run_as_user`, it's recommended to also configure matching `permissions.uid/gid`
- Permission setup happens automatically during `micro_proxy start` — the tool generates a shell script with `mkdir -p` + `chown` and executes it before `docker compose up`. If chown fails without sudo, it retries with `sudo bash` automatically. If both fail, a warning is logged with the manual command to run.
- **Path consistency:** The `target` path determines where data is persisted on the container filesystem. Ensure that any data paths configured in `.env` or other config files (e.g., SQLite database path, upload directory, log file location) are located **under** a volume target. Otherwise data will be stored inside the container's ephemeral layer and lost on container restart.

### 6. Naming & Derivation

micro_proxy 中有多个关键名称，它们的来源和用途各不相同。理解这条推导链路对排查问题至关重要。

**Derivation chain:**

```
Directory name (e.g., my_app/)
  │
  ├──→ app.name = "my_app"
  │      ├──→ Docker image   = "{app.name}:latest"          → "my_app:latest"
  │      ├──→ nginx variable = "${app.name}_upstream_host"  → "$my_app_upstream_host"
  │      └──→ apps-config.yml `name` field
  │
  └── micro-app.yml (hand-written)
         └── container_name = "my-container"   ← user-configured, independent of directory name
```

**Name summary:**

| Name | Source | Derivation Rule | Example |
|------|--------|-----------------|---------|
| `app.name` | Directory name (auto) | Direct child: directory name. Nested: path components joined with `_` | `my_app`, `group_my_app` |
| Docker image | From `app.name` | `{app.name}:latest` | `my_app:latest` |
| nginx variable | From `app.name` | `${app.name}_upstream_host` | `$my_app_upstream_host` |
| `container_name` | User in `micro-app.yml` | No derivation, must be globally unique | `my-container` |

**Directory naming constraints:**

Because `app.name` is used to construct nginx variable names, and nginx variables only allow `[A-Za-z0-9_]` (must not start with a digit), directory names must follow the same rules:

- ✅ Valid: `my_app`, `group_my_app`, `App123`
- ❌ Invalid: `my-app` (contains `-`), `123app` (starts with digit), `my.app` (contains `.`)

> `micro_proxy start` validates this and **fails with an error** if any directory name violates the rule.

**Key insight for AI:** When suggesting directory names or project structures for new micro-apps, always use underscores (`_`) instead of hyphens (`-`). The `container_name` in `micro-app.yml` is independent and **can** contain hyphens (it's a Docker container name, not an nginx variable).

---

## Type-Specific Templates

### Static Type (Frontend / Static Website)

**micro-app.yml:**
```yaml
routes: ["/"]
container_name: "my_frontend"
container_port: 80
app_type: "static"
description: "Frontend application"
```

**Dockerfile** (multi-stage build example):
```dockerfile
FROM node:18-alpine as builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM nginx:alpine
COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /app/dist /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

**nginx.conf** (required for SPA):
```nginx
server {
    listen 80;
    server_name localhost;
    root /usr/share/nginx/html;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }

    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
```

**Note:** If using `VITE_BASE_URL` or similar env vars, ensure it ends with `/` (e.g., `/app/`), otherwise paths will render as `//path`.

**⚠️ Critical: Container path vs. VITE_BASE_URL**

micro_proxy 对 Static 类型的请求会 **剥离 route 前缀** 后再转发给容器，因此：

- **容器内部** 始终以根路径 `/` 部署：文件放在 `/usr/share/nginx/html`，nginx `location /` 提供服务
- **浏览器侧** 必须知道应用挂载在哪个子路径，才能生成正确的资源 URL 和路由链接

**规则：`VITE_BASE_URL`（或等效的 base 配置）必须等于 `micro-app.yml` 中配置的 route 路径，且以 `/` 结尾。**

| `micro-app.yml` routes | 容器内部部署路径 | `VITE_BASE_URL` | 浏览器实际访问 |
|-------------------------|-----------------|------------------|--------------|
| `["/"]` | `/usr/share/nginx/html` | `/` | `http://host/` |
| `["/gg123/"]` | `/usr/share/nginx/html` | `/gg123/` | `http://host/gg123/` |
| `["/admin/dashboard/"]` | `/usr/share/nginx/html` | `/admin/dashboard/` | `http://host/admin/dashboard/` |

**错误配置示例：**
- `routes: ["/gg123/"]` + `VITE_BASE_URL=/` → 浏览器生成 `<script src="/assets/index.js">`，请求 `/assets/index.js`，micro_proxy 找不到匹配的 route，**404**
- `routes: ["/gg123/"]` + 文件部署到 `/usr/share/nginx/html/gg123/` → 容器 nginx 收到剥离前缀后的请求 `/`，在该路径下找不到文件，**404**

**Request flow (correct configuration):**
```
Browser requests:  https://host/gg123/assets/index.js
                         ↓
Upper-level nginx: matches route /gg123/, strips prefix
                         ↓
Forwards to container: GET /assets/index.js
                         ↓
Container nginx:    location / → serves /usr/share/nginx/html/assets/index.js ✓
```

### API Type (Backend Service)

**micro-app.yml:**
```yaml
routes: ["/api"]
container_name: "my_backend"
container_port: 8080
app_type: "api"
description: "Backend API service"
nginx_extra_config: |
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

**Dockerfile:**
```dockerfile
FROM node:18-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
EXPOSE 8080
CMD ["node", "server.js"]
```

**Path handling:** API requests are forwarded **with the path preserved**:
```
Request:  http://host/api/v1/users
         ↓ (Nginx proxy)
Backend receives: http://backend:8080/api/v1/users  ← path unchanged
```

### Internal Type (Database / Middleware / Internal HTTP Services)

Internal 类型有两种模式：

#### 模式 A：纯内部服务（无 routes）

适用于 Redis、MySQL 等不需要对外暴露的服务。

**micro-app.yml:**
```yaml
routes: []                     # 空数组 = 纯内部服务，不通过 nginx 暴露
container_name: "my_redis"
container_port: 6379
app_type: "internal"
description: "Redis cache service"
```

**Dockerfile:**
```dockerfile
FROM redis:7-alpine
EXPOSE 6379
```

**Internal communication** (其他容器通过容器名访问):
```bash
redis-cli -h my_redis -p 6379
curl http://my_redis:6379
mongosh mongodb://my_mongodb:27017
```

#### 模式 B：对外暴露 HTTP 服务的内部应用（有 routes）

适用于 MinIO、Adminer、pgAdmin 等提供 HTTP 接口但属于基础设施的内部服务。通过 nginx 代理对外暴露，**自动剥离路由前缀**（与 Static 类型相同，因为第三方服务无法处理路由前缀）。

**micro-app.yml:**
```yaml
routes: ["/minio"]             # 配置路由以对外暴露
container_name: "my_minio"
container_port: 9000
app_type: "internal"
description: "MinIO object storage"
nginx_extra_config: |
  client_max_body_size 0;      # MinIO 需要上传大文件
```

**Dockerfile:**
```dockerfile
FROM minio/minio:latest
EXPOSE 9000
CMD ["server", "/data"]
```

**配置说明：**
- `routes` 非空时，内部应用通过 nginx 代理，并**自动剥离路由前缀**（例：访问 `/minio/bucket/file`，后端收到 `/bucket/file`）
- 支持 `nginx_extra_config`、`proxy_connect_timeout` 等配置
- nginx `depends_on` 会自动包含该容器
- 网络信息展示中会显示可访问 URL

**访问方式对比：**

| 场景 | routes | 对外访问 | 网络信息展示 |
|------|--------|----------|-------------|
| Redis（纯内部） | `[]` | 无 | 显示"无（内部服务）" |
| MinIO（对外暴露） | `["/minio"]` | `http://host/minio` | 显示 URL |

---

## SPA Deployment Checklist

When deploying single-page applications (Vue/React), verify these items:

| Check | Location | Requirement |
|-------|----------|-------------|
| `try_files` directive | `nginx.conf` | Must have `try_files $uri $uri/ /index.html;` |
| `COPY nginx.conf` | `Dockerfile` | Must copy custom nginx.conf into the image |
| `BASE_URL` equals route | `.env` / `vite.config.ts` | **Must equal the route** from `micro-app.yml` (with trailing `/`). E.g., `routes: ["/gg123/"]` → `VITE_BASE_URL=/gg123/`. If `routes: ["/"]` → `VITE_BASE_URL=/`. |
| Force rebuild | Docker build | After modifying `.env`, clear cache: `docker build --no-cache -t <name> .` |

**Common errors:**

| Error | Cause | Fix |
|-------|-------|-----|
| 404 on page refresh | Missing `try_files` | Add `try_files $uri $uri/ /index.html;` to nginx.conf |
| Double slash `//path` in URLs | BASE_URL missing trailing `/` | Change `.env` to `VITE_BASE_URL=/app/` |
| Config change not taking effect | Docker build cache | Rebuild with `docker build --no-cache ...` |
| Permission denied | Missing volume permissions | Configure `permissions` in `micro-app.volumes.yml` |

---

## apps-config.yml (Auto-Generated)

`apps-config.yml` is **auto-generated** by `micro_proxy`. **Do not edit manually.** It records scanned micro-app configurations.

**Location:** Configured via `apps_config_path` in `proxy-config.yml`:
```yaml
apps_config_path: "./apps-config.yml"
```

**Regeneration triggers:**
1. Running `micro_proxy start`
2. Changes detected in scan directories (add/remove/modify micro-apps)

**Key fields in apps-config.yml:**

| Field | Source | Description |
|-------|--------|-------------|
| `name` | Auto-generated | Format: `{scan_dir_relative_path}_{last_dir_name}` (e.g., `apps_craftaidhub_front`). **Directory names must not contain hyphens (`-`)** — this name is used to construct nginx server variable names (e.g., `{name}_upstream_host`), and nginx variables only allow `[A-Za-z0-9_]`. Use underscores (`_`) instead. |
| `path` | Auto-generated | Absolute path to the micro-app directory on the host |
| `docker_volumes` | Converted from `micro-app.volumes.yml` | Volume mappings in `source:target` format |
| `run_as_user` | Copied from `micro-app.volumes.yml` | Container runtime user, null if not configured |
| All other fields | From `micro-app.yml` | Direct copy |

**Relationship diagram:**
```
┌─────────────────┐     Scan discover     ┌──────────────────┐
│  micro-app.yml  │ ───────────────────→  │ apps-config.yml  │
│  (hand-written) │                       │ (auto-generated)  │
└─────────────────┘                       └──────────────────┘
       │                                         │
       │ Read config                              │ Generate nginx
       ▼                                         ▼
┌─────────────────┐                     ┌──────────────────┐
│   Dockerfile    │                     │   nginx.conf     │
│  (hand-written) │                     │ (auto-generated) │
└─────────────────┘                     └──────────────────┘
       │
       │ Read config
       ▼
┌─────────────────┐
│micro-app.       │
│volumes.yml      │
│(hand-written)   │
└─────────────────┘
```

---

## AI Behavior Guidelines

When helping users with micro-app development, follow these principles:

### 1. Diagnosis First
Before generating any files, determine:
- What type of app does the user need? (Static / API / Internal)
- What features are required? (Data persistence, SPA routing, CORS, etc.)
- Does the user have existing code?
- **What route path(s) the app should use** — This must be explicitly asked from the user. Do NOT auto-derive default routes (such as `/` for Static or `/api` for API). **Why:** Multiple micro-apps share the same proxy domain, and each must have unique `routes` to avoid conflicts. Auto-derived defaults would collide when multiple apps are deployed. Only the user knows the intended deployment path; the AI must not guess.

### 2. Generate Configurations
Based on the diagnosis, provide:
- `micro-app.yml` (always required)
- `Dockerfile` (always required)
- `nginx.conf` (required for SPA, optional for others)
- `micro-app.volumes.yml` (when persistence or permission configuration is needed)

### 3. Validate
Check the generated configurations for:
- **Directory naming** — Micro-app directory names must only contain `[A-Za-z0-9_]` and must not start with a digit. The directory name is used to construct nginx server variable names (`{name}_upstream_host`), and nginx variables only allow alphanumeric characters and underscores. Hyphens (`-`) will cause `micro_proxy start` to fail with a validation error.
- `container_name` uniqueness
- `container_port` matching the application's listen port
- Permission security (warn on uid/gid = 0)
- SPA deployment checklist items
- **`routes` 完整性校验（两步）：**
  1. **格式检查：** Static/API 必须有 ≥1 条路由，Internal 必须为 `[]`
  2. **一致性检查：** 对照应用源码中的路由/路径配置，确认 `micro-app.yml` 的 `routes` 与应用自身配置一致。常见检查位置：
     - Static: `VITE_BASE_URL` 在 `.env`，`base` 在 `vite.config.ts` / `vit.config.ts`，`publicPath` 在 `vue.config.js`，`homepage` 在 `package.json`，框架 router `base` 选项。**判定标准：`VITE_BASE_URL`（或 `base` 配置）必须等于 `micro-app.yml` 中的 route 路径。** 例如 `routes: ["/gg123/"]` 时，`VITE_BASE_URL` 必须是 `/gg123/`；`routes: ["/"]` 时必须是 `/`。
     - API: `API_PREFIX` / `BASE_PATH` 在 `.env`，`routePrefix` 在服务启动代码中，框架级挂载路径
  如果 `micro-app.yml` 的 routes **与应用的路径配置不匹配**，或应用**根本没有相关配置**，则**停止并警告**用户。说明潜在的偏差，让用户修正应用配置或 `micro-app.yml` 后再继续。
- **Volume target path consistency** — If `micro-app.volumes.yml` defines volumes, verify that any file storage paths in other configurations (especially `.env`) fall **under** a volume `target` directory. E.g., volume target `/data` and `.env` contains `DATABASE_PATH=/data/db.sqlite` ✓; `DATABASE_PATH=/app/db.sqlite` ✗ (path outside volume). Consider all common path-like env vars (`*_PATH`, `*_DIR`, `*_FILE`, `*_STORAGE`, etc.). Warn if a data path falls outside any volume mount.

### 4. Explain, Don't Just Output
When providing configuration examples, explain:
- Why each field is needed
- What each value means
- Common mistakes to avoid

### 5. Troubleshoot
Use the error tables in this document to help users diagnose issues.

---

## FAQ

| Question | Answer |
|----------|--------|
| Is `micro-app.volumes.yml` required? | No, it's optional. Without it, `docker_volumes` will be an empty array and `run_as_user` will be null. |
| Can I edit `apps-config.yml` manually? | Not recommended. It's regenerated on every `micro_proxy start`. Edit `micro-app.yml` or `micro-app.volumes.yml` instead. |
| What if `name` fields conflict? | Restructure your micro-app directories. Use different parent directories to distinguish same-named apps. |
| Must `run_as_user` and `permissions` be configured together? | No, they are independent. You can use either one or both. |
| How do I know which user the app runs as? | Check `run_as_user` in `apps-config.yml`. If null, the image's default user is used (often root). |

---

## Workflow Summary

```
1. Diagnose user needs
   ├── App type: Static / API / Internal
   ├── Features: Persistence? SPA? CORS? Custom Nginx?
   ├── Existing code structure?
   └── **Routes: explicitly ask user** (do NOT auto-derive — multiple apps need unique routes)

2. Generate required files
   ├── micro-app.yml (always)
   ├── Dockerfile (always)
   ├── nginx.conf (SPA required, others optional)
   └── micro-app.volumes.yml (persistence/permissions needed)

3. Validate configuration
   ├── Directory names: only [A-Za-z0-9_], no hyphens (used as nginx variable names)
   ├── container_name globally unique
   ├── routes 完整性校验（格式检查 → 一致性检查，不匹配则停止并警告）
   ├── container_port matches app listen port
   ├── No security risks (root uid/gid warning)
   └── Volume target ↔ .env path consistency (data paths must fall under a volume target)

4. Guide deployment
   ├── Run `micro_proxy start`
   ├── Verify apps-config.yml is correctly generated
   └── Verify service is running
```

---

## References

- Source document: `https://github.com/cao5zy/proxy-config/blob/main/docs/micro-app-development.md`
- Main config example: `https://github.com/cao5zy/proxy-config/blob/main/proxy-config.yml.example`
- Micro-app config example: `https://github.com/cao5zy/proxy-config/blob/main/micro-app.yml.example`
- Project README: `https://github.com/cao5zy/proxy-config/blob/main/README.md`
