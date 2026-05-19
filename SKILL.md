# micro-app-development

## Purpose

本文档为 AI 编程助手（如 Qoder、Claude Code、Cursor、Windsurf、GitHub Copilot、Cline 等）提供 **micro_proxy** 平台微应用开发所需的完整知识库和行为指南。AI 在回答用户关于 micro-app 创建、配置、部署相关问题时，应参考本文档的内容。

## Platform Compatibility

本文档使用标准 Markdown 编写，结构清晰、分段明确，可直接被以下平台作为上下文或指令文件使用：

| Platform | Usage |
|----------|-------|
| **Qoder** | 放置为 `SKILL.md`，按 `When to Activate` 章节自动触发 |
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
| **Internal** | Database / Middleware (Redis, MySQL) | No | N/A | `container-name:port` |

**Decision guide:**
- User has HTML/JS/CSS or Vue/React project → **Static**
- User has backend code (Node.js/Python/Go/Java) providing HTTP API → **API**
- User needs Redis, MySQL, MongoDB, etc. → **Internal**
- Same project has both frontend and backend → Two separate micro-apps

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
| `routes` | Conditional | `string[]` | Access paths. Static/API: must have ≥1 route. Internal: must be `[]` |
| `container_name` | Yes | `string` | Container name, **globally unique** across all micro-apps |
| `container_port` | Yes | `int` | Container internal port (1-65535) |
| `app_type` | Yes | `enum` | One of: `static`, `api`, `internal` |
| `description` | No | `string` | App description |
| `nginx_extra_config` | No | `string` | Extra Nginx directives (static/api only) |

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
- Permission setup happens before container start, requiring appropriate host privileges

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

### Internal Type (Database / Middleware)

**micro-app.yml:**
```yaml
routes: []                     # Must be empty array
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

**Internal communication** (other containers access via container name):
```bash
redis-cli -h my_redis -p 6379
curl http://my_redis:6379
mongosh mongodb://my_mongodb:27017
```

---

## SPA Deployment Checklist

When deploying single-page applications (Vue/React), verify these items:

| Check | Location | Requirement |
|-------|----------|-------------|
| `try_files` directive | `nginx.conf` | Must have `try_files $uri $uri/ /index.html;` |
| `COPY nginx.conf` | `Dockerfile` | Must copy custom nginx.conf into the image |
| `BASE_URL` trailing slash | `.env` | Sub-path deployment must end with `/` (e.g., `/app/`) |
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
| `name` | Auto-generated | Format: `{scan_dir_relative_path}_{last_dir_name}` (e.g., `apps_craftaidhub_front`) |
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

### 2. Generate Configurations
Based on the diagnosis, provide:
- `micro-app.yml` (always required)
- `Dockerfile` (always required)
- `nginx.conf` (required for SPA, optional for others)
- `micro-app.volumes.yml` (when persistence or permission configuration is needed)

### 3. Validate
Check the generated configurations for:
- `container_name` uniqueness
- `routes` correctness per app type
- `container_port` matching the application's listen port
- Permission security (warn on uid/gid = 0)
- SPA deployment checklist items

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
   └── Existing code structure?

2. Generate required files
   ├── micro-app.yml (always)
   ├── Dockerfile (always)
   ├── nginx.conf (SPA required, others optional)
   └── micro-app.volumes.yml (persistence/permissions needed)

3. Validate configuration
   ├── container_name globally unique
   ├── routes match app type requirements
   ├── container_port matches app listen port
   └── No security risks (root uid/gid warning)

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
