# proxy-config 设计方案

## 1. 项目概述

proxy-config是一个用于管理微应用的工具，它能够：
- 自动发现微应用
- 构建微应用的Docker镜像
- 生成nginx反向代理配置
- 生成docker-compose.yml文件
- 管理容器的生命周期（启动、停止、清理）
- 支持微应用的预构建和后清理脚本

## 2. 系统架构

### 2.1 模块划分

```
proxy-config/
├── config/           # 配置管理模块
├── discovery/        # 应用发现模块
├── builder/          # 镜像构建模块
├── container/        # 容器管理模块
├── nginx/            # Nginx配置生成模块
├── compose/          # Docker Compose生成模块
├── state/            # 状态管理模块
├── script/           # 脚本执行模块
└── cli/              # 命令行接口
```

### 2.2 核心模块说明

#### 2.2.1 配置管理模块 (config)
- 职责：读取和解析proxy-config的配置文件
- 功能：
  - 解析YAML配置文件
  - 验证配置的有效性
  - 提供配置访问接口

#### 2.2.2 应用发现模块 (discovery)
- 职责：扫描微应用目录，发现符合条件的微应用
- 功能：
  - 遍历指定的微应用目录
  - 检查每个目录是否包含Dockerfile
  - 检查每个目录是否包含.env文件
  - 收集微应用的基本信息

#### 2.2.3 镜像构建模块 (builder)
- 职责：构建微应用的Docker镜像
- 功能：
  - 调用docker build命令
  - 传递.env文件中的环境变量
  - 处理构建过程中的错误
  - 记录构建日志

#### 2.2.4 容器管理模块 (container)
- 职责：管理容器的生命周期
- 功能：
  - 创建容器
  - 启动容器
  - 停止容器
  - 删除容器
  - 查询容器状态

#### 2.2.5 Nginx配置生成模块 (nginx)
- 职责：根据配置生成nginx.conf
- 功能：
  - 解析应用配置
  - 生成upstream配置
  - 生成location配置
  - 处理反向代理规则

#### 2.2.6 Docker Compose生成模块 (compose)
- 职责：生成docker-compose.yml文件
- 功能：
  - 生成services配置
  - 配置网络
  - 配置卷挂载
  - 配置环境变量

#### 2.2.7 状态管理模块 (state)
- 职责：记录和管理微应用的状态
- 功能：
  - 计算微应用目录的hash值
  - 保存状态到文件
  - 读取状态文件
  - 比较状态判断是否需要重新构建

#### 2.2.8 脚本执行模块 (script)
- 职责：执行微应用的setup.sh和clean.sh脚本
- 功能：
  - 检查脚本是否存在
  - 执行脚本
  - 捕获脚本输出
  - 处理脚本错误

#### 2.2.9 命令行接口模块 (cli)
- 职责：提供命令行交互接口
- 功能：
  - 解析命令行参数
  - 调用相应的模块功能
  - 显示帮助信息
  - 显示执行结果

## 3. 数据结构设计

### 3.1 主配置结构 (ProxyConfig)

```rust
pub struct ProxyConfig {
    pub apps: Vec<AppConfig>,
    pub micro_apps_dir: String,
    pub nginx_config_path: String,
    pub compose_config_path: String,
    pub state_file_path: String,
}

pub struct AppConfig {
    pub name: String,
    pub context: String,
    pub container_name: String,
    pub container_port: u16,
    pub description: Option<String>,
}
```

### 3.2 微应用结构 (MicroApp)

```rust
pub struct MicroApp {
    pub name: String,
    pub path: PathBuf,
    pub env_file: PathBuf,
    pub dockerfile: PathBuf,
    pub setup_script: Option<PathBuf>,
    pub clean_script: Option<PathBuf>,
    pub image_name: String,
    pub container_name: String,
    pub container_port: u16,
}
```

### 3.3 应用状态结构 (AppState)

```rust
pub struct AppState {
    pub app_name: String,
    pub hash: String,
    pub last_built: DateTime<Utc>,
    pub image_exists: bool,
}
```

## 4. 工作流程

### 4.1 启动流程

```
1. 读取配置文件
   ↓
2. 扫描微应用目录
   ↓
3. 对每个微应用：
   a. 检查状态文件
   b. 计算当前目录hash
   c. 比较hash值，判断是否需要重新构建
   d. 如果需要重新构建：
      - 执行setup.sh（如果存在）
      - 构建Docker镜像
      - 更新状态文件
   ↓
4. 生成nginx.conf
   ↓
5. 生成docker-compose.yml
   ↓
6. 启动所有容器
```

### 4.2 停止流程

```
1. 读取配置文件
   ↓
2. 停止所有容器
```

### 4.3 清理流程

```
1. 读取配置文件
   ↓
2. 停止并删除所有容器
   ↓
3. 对每个微应用：
   a. 删除镜像
   b. 执行clean.sh（如果存在）
   ↓
4. 删除状态文件
```

## 5. 配置文件格式

### 5.1 proxy-config配置文件 (proxy-config.yml)

```yaml
# 微应用目录
micro_apps_dir: "./micro-apps"

# Nginx配置文件输出路径
nginx_config_path: "./nginx.conf"

# Docker Compose配置文件输出路径
compose_config_path: "./docker-compose.yml"

# 状态文件路径
state_file_path: "./proxy-config.state"

# 反向代理配置
apps:
  - name: "main-app"
    context: "/"
    container_name: "craftaidhub-main"
    container_port: 80
    description: "主入口网站"
    
  - name: "resume-app"
    context: "/resume_app"
    container_name: "resume-agent-frontend"
    container_port: 80
    description: "简历应用"
```

### 5.2 微应用目录结构

```
micro-apps/
├── main-app/
│   ├── Dockerfile
│   ├── .env
│   ├── setup.sh      # 可选
│   ├── clean.sh      # 可选
│   └── ...           # 其他应用文件
├── resume-app/
│   ├── Dockerfile
│   ├── .env
│   ├── setup.sh      # 可选
│   ├── clean.sh      # 可选
│   └── ...           # 其他应用文件
```

### 5.3 微应用.env文件示例

```env
# 应用端口
APP_PORT=80

# 其他环境变量
ENV=production
DEBUG=false
```

## 6. 命令行接口

### 6.1 启动命令

```bash
proxy-config start [options]
```

选项：
- `-c, --config <path>`: 指定配置文件路径（默认：./proxy-config.yml）
- `-v, --verbose`: 显示详细日志

### 6.2 停止命令

```bash
proxy-config stop [options]
```

选项：
- `-c, --config <path>`: 指定配置文件路径（默认：./proxy-config.yml）

### 6.3 清理命令

```bash
proxy-config clean [options]
```

选项：
- `-c, --config <path>`: 指定配置文件路径（默认：./proxy-config.yml）
- `--force`: 强制清理，不询问确认

### 6.4 状态查看命令

```bash
proxy-config status [options]
```

选项：
- `-c, --config <path>`: 指定配置文件路径（默认：./proxy-config.yml）

## 7. 技术选型

### 7.1 Rust依赖

- `serde`: 序列化/反序列化
- `serde_yaml`: YAML配置文件解析
- `walkdir`: 目录遍历
- `sha2`: 计算目录hash
- `chrono`: 时间处理
- `log`: 日志记录
- `env_logger`: 日志初始化
- `clap`: 命令行参数解析
- `anyhow`: 错误处理
- `thiserror`: 错误定义

### 7.2 外部依赖

- Docker: 用于镜像构建和容器管理
- Nginx: 反向代理服务器

## 8. 错误处理

所有模块都应该返回Result类型，使用anyhow或thiserror进行错误处理。错误信息应该包含足够的上下文信息，便于调试。

## 9. 日志记录

使用log和env_logger进行日志记录，日志级别包括：
- error: 错误信息
- warn: 警告信息
- info: 一般信息
- debug: 调试信息
- trace: 详细跟踪信息

## 10. 测试策略

- 单元测试：测试各个模块的核心功能
- 集成测试：测试模块之间的交互
- 端到端测试：测试完整的启动、停止、清理流程

## 11. 文档

- 配置指南：说明如何配置proxy-config
- 使用指南：说明如何使用proxy-config
- 微应用开发指南：说明如何开发符合要求的微应用
