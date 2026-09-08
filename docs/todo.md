# 待办事项

## 镜像构建与部署解耦

目标：将镜像构建从 `micro_proxy start` 中移出，使构建失败或待验证的版本不会影响正在运行的站点；同时保留可快速回滚的已构建镜像。

### 第一期：不可变镜像与显式部署

- [x] 新增 `micro_proxy build [APP_NAME...]`：扫描指定应用（未指定时扫描全部），只构建镜像，不停止、删除或重启任何容器，也不写入部署状态。
- [x] 将镜像命名从固定的 `<app>:latest` 改为不可变标签，例如 `<app>:sha-<构建上下文哈希短码>`；哈希输入应包含 Dockerfile、构建上下文和会影响构建结果的构建参数。
- [x] 新增部署状态文件，记录每个应用的 `active_image`、`previous_image` 和按部署时间保留的 `history_images`。
- [x] 保留 `micro_proxy start`，但使其只按部署状态启动已选镜像；不得扫描源码变化或隐式构建镜像。
- [x] 新增 `micro_proxy deploy <APP_NAME> --image <IMAGE_REF>`，将一个已存在的镜像设为活动版本，并更新生成的 Compose 配置。
- [x] 新增 `micro_proxy rollback <APP_NAME>`，恢复到 `previous_image`，且不重新从源码构建。
- [x] 提供旧安装迁移：若尚无活动部署而 `<app>:latest` 存在，将其登记为 `active_image`；现有 `proxy-config.yml`、`micro-app.yml` 和 `micro-app.volumes.yml` 不作修改。
- [x] 调整 `status` 与 `clean`：显示活动、可回滚及全部历史镜像；清理时只删除历史镜像，不得误删活动镜像或可回滚镜像。
- [x] 为镜像引用、部署状态迁移、Compose 镜像选择等纯函数补充 TDD 单元测试；运行 `cargo test`。
- [x] `build`、`deploy`、`rollback` 仅接受应用名（`APP_NAME`）；保持镜像仓库和部署状态以应用名命名，并在构建输出中展示应用名、容器名和镜像引用的映射。

### 第二期：健康检查与蓝绿切换

- [x] 支持在 `micro-app.yml` 通过可选 `healthcheck_path` 配置 Static/API 的 HTTP 健康检查路径；未配置时使用 `/`，保持既有配置兼容。
- [x] 修复 Static/API 容器的 Compose 健康检查：使用 `CMD-SHELL` 数组而非字符串，固定请求 IPv4 回环地址 `127.0.0.1` 以避免 `localhost` 优先解析至未监听的 IPv6 地址；部署等待窗口覆盖 `interval × retries + timeout`，并输出等待进度。
- [x] 部署新版本时保留旧容器继续服务，以候选容器名启动新镜像。
- [x] 在切流前等待候选容器通过健康检查；超时或失败时删除候选容器，旧版本保持运行。
- [x] 新增 `deploy --force` 作为一次性人工兜底：仅当候选容器仍在运行时，才允许绕过失败的健康检查切流；默认 `false`，已退出的候选容器永不切换。
- [x] 候选容器健康后，更新 Nginx 上游并平滑 reload，使流量切至新版本。
- [x] 切流成功后停止并删除旧容器，并将其镜像保留为 `previous_image`。
- [ ] 为容器未通过健康检查、Nginx reload 失败、切换中断等路径定义可恢复策略与集成测试。
