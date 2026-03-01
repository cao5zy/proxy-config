
#!/bin/bash

# deploy_to_local.sh
# 编译并部署 micro_proxy 到本地 bin 目录

set -e  # 遇到错误立即退出

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 项目配置
PROJECT_NAME="micro_proxy"
TARGET_DIR="target/release"
BINARY_NAME="${PROJECT_NAME}"
USER_BIN_DIR="${HOME}/bin"

# 检查是否在项目根目录
if [ ! -f "Cargo.toml" ]; then
    log_error "未找到 Cargo.toml，请确保在项目根目录下运行此脚本"
    exit 1
fi

log_info "开始部署 ${PROJECT_NAME} 到本地..."

# 检查 Rust 环境
if ! command -v cargo &> /dev/null; then
    log_error "未找到 cargo，请先安装 Rust 环境"
    exit 1
fi

log_info "Rust 环境检查通过"

# 编译项目
log_info "开始编译项目 (release 模式)..."
if ! cargo build --release; then
    log_error "编译失败"
    exit 1
fi

log_info "编译成功"

# 检查编译产物是否存在
BINARY_PATH="${TARGET_DIR}/${BINARY_NAME}"
if [ ! -f "${BINARY_PATH}" ]; then
    log_error "编译产物未找到: ${BINARY_PATH}"
    exit 1
fi

log_info "找到编译产物: ${BINARY_PATH}"

# 创建用户 bin 目录（如果不存在）
if [ ! -d "${USER_BIN_DIR}" ]; then
    log_info "创建用户 bin 目录: ${USER_BIN_DIR}"
    mkdir -p "${USER_BIN_DIR}"
    if [ $? -ne 0 ]; then
        log_error "创建目录失败: ${USER_BIN_DIR}"
        exit 1
    fi
fi

# 复制二进制文件
log_info "复制二进制文件到 ${USER_BIN_DIR}..."
if ! cp "${BINARY_PATH}" "${USER_BIN_DIR}/"; then
    log_error "复制文件失败"
    exit 1
fi

# 设置执行权限
log_info "设置执行权限..."
chmod +x "${USER_BIN_DIR}/${BINARY_NAME}"

# 验证部署
DEPLOYED_BINARY="${USER_BIN_DIR}/${BINARY_NAME}"
if [ ! -f "${DEPLOYED_BINARY}" ]; then
    log_error "部署验证失败: 文件不存在"
    exit 1
fi

log_info "部署成功: ${DEPLOYED_BINARY}"

# 检查 bin 目录是否在 PATH 中
if [[ ":$PATH:" != *":${USER_BIN_DIR}:"* ]]; then
    log_warn "${USER_BIN_DIR} 不在 PATH 环境变量中"
    log_warn "请将以下内容添加到你的 shell 配置文件中 (~/.zshrc 或 ~/.bash_profile):"
    echo ""
    echo "  export PATH=\"${USER_BIN_DIR}:\$PATH\""
    echo ""
    log_warn "然后执行: source ~/.zshrc (或 source ~/.bash_profile)"
else
    log_info "${USER_BIN_DIR} 已在 PATH 中"
fi

# 显示版本信息
log_info "验证安装..."
if "${DEPLOYED_BINARY}" --version &> /dev/null; then
    "${DEPLOYED_BINARY}" --version
else
    log_warn "无法获取版本信息，但文件已成功部署"
fi

log_info "部署完成！"
