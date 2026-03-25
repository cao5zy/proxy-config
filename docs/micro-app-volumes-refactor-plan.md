### 一、背景与问题

| 问题 | 说明 |
|------|------|
| **权限管理缺失** | Docker 容器与宿主机目录权限不一致导致服务启动失败 |
| **配置分散** | volumes、权限等配置混在 micro-app.yml 中，不够清晰 |
| **扩展性差** | 新增 volume_permissions、run_as_user 等配置会使 micro-app.yml 过于臃肿 |

---

### 二、设计目标

1. **职责分离**：核心配置与卷配置分离
2. **向后兼容**：micro-app.volumes.yml 为可选文件，不影响现有部署
3. **统一管理**：所有卷相关配置集中在一个文件中
4. **易于扩展**：方便后续添加更多卷相关配置项

---

### 三、配置结构变化

#### 当前结构（Before）

```yaml
# micro-app.yml
routes: []
container_name: "resume_agent_mongodb"
container_port: 27017
app_type: "internal"
description: "Mongodb 存储"
docker_volumes:
  - "./resume_agent_mongodb/data:/data/db"
  - "./resume_agent_mongodb/logs:/var/log/mongodb"
```

#### 目标结构（After）

```yaml
# micro-app.yml (核心配置)
routes: []
container_name: "resume_agent_mongodb"
container_port: 27017
app_type: "internal"
description: "Mongodb 存储"
# docker_volumes 已移除
```

```yaml
# micro-app.volumes.yml (可选，卷配置)
volumes:
  - source: "./resume_agent_mongodb/data"
    target: "/data/db"
    permissions:
      uid: 999
      gid: 999
      recursive: true

run_as_user: "999:999"
```

---

### 四、功能需求

| 功能 | 说明 | 优先级 |
|------|------|--------|
| **读取 micro-app.volumes.yml** | 如果文件存在则读取，不存在则忽略 | P0 |
| **volume_permissions 支持** | 自动设置挂载目录的 UID/GID 权限 | P0 |
| **run_as_user 支持** | 指定容器运行用户 | P0 |
| **向后兼容** | 不破坏现有 micro-app.yml 配置 | P0 |
| **错误处理** | 配置文件格式错误时给出明确提示 | P1 |

---

### 五、实现方案

#### 1. 数据结构定义

```rust
// 卷配置
struct VolumeConfig {
    source: String,      // 宿主机路径
    target: String,      // 容器内路径
    permissions: Option<VolumePermissions>,
}

// 权限配置
struct VolumePermissions {
    uid: u32,
    gid: u32,
    recursive: bool,
}

// 完整卷配置
struct VolumesConfig {
    volumes: Vec<VolumeConfig>,
    run_as_user: Option<String>,
}
```

#### 2. 配置加载逻辑

```
┌─────────────────────────────────────────────────────────────┐
│                    配置加载流程                              │
├─────────────────────────────────────────────────────────────┤
│  1. 读取 micro-app.yml → AppConfig                          │
│  2. 检查 micro-app.volumes.yml 是否存在                      │
│  3. 如果存在 → 读取并解析为 VolumesConfig                   │
│  4. 合并配置 → 生成最终的 Docker Compose 配置               │
└─────────────────────────────────────────────────────────────┘
```

#### 3. Docker Compose 生成

```yaml
services:
  resume_agent_mongodb:
    image: resume_agent_mongodb:latest
    volumes:
      - ./resume_agent_mongodb/data:/data/db
    user: "999:999"  # 来自 run_as_user
    # 权限初始化通过 init_container 或外部脚本实现
```

---

### 六、影响范围

| 模块 | 影响程度 | 说明 |
|------|---------|------|
| **配置解析** | 高 | 需要新增 VolumesConfig 结构体 |
| **Docker Compose 生成** | 高 | 需要处理 volumes 和 user 字段 |
| **CLI 命令** | 低 | 无需修改，配置是隐式加载的 |
| **现有部署** | 无 | 保持向后兼容 |

---

### 七、验收标准

- [ ] micro-app.volumes.yml 不存在时，程序正常运行
- [ ] micro-app.volumes.yml 存在时，正确读取并应用配置
- [ ] volume_permissions 能正确设置目录权限
- [ ] run_as_user 能正确设置容器运行用户
- [ ] 生成的 docker-compose.yml 符合预期
- [ ] 现有 micro-app.yml 配置不受影响
