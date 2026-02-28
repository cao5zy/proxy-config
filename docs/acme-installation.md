
# ACME.sh 安装与配置指南

## 什么是 ACME.sh？

ACME.sh 是一个纯 Unix shell 脚本实现的 ACME 协议客户端，用于从 Let's Encrypt 等 CA 机构申请和管理 SSL/TLS 证书。

## 一键安装脚本（推荐）

以下是一个经过验证的完整安装脚本，特别针对 **micro_proxy** 工具进行了优化，并使用 **Gitee 镜像**（适配中国大陆网络环境）。

> **重要提示**：在使用前，请务必修改脚本中的 `DOMAIN` 和 `ACME_EMAIL` 配置！

```bash
#!/bin/bash

# 移除 set -e 以避免因 acme.sh 的警告信息导致脚本意外退出
# 我们将手动检查关键步骤的返回状态

# ====================== 自定义配置（请修改为你的信息）======================
DOMAIN="craftaidhub.com"          # 你的域名（如：blog.example.com）
DOMAIN_ALT="www.craftaidhub.com" # 可选：备用域名/子域名，无则留空
WEBROOT="/var/www/html"         # 主机上的网站根目录（需和Docker Nginx挂载的目录一致）
DOCKER_NGINX_CONTAINER="proxy-nginx"  # 你的Docker Nginx容器名（如：proxy-nginx）
CERT_DIR="/etc/nginx/certs"     # 主机上存放证书的目录（会自动创建）
ACME_EMAIL="zongying_cao@163.com"  # acme.sh 注册邮箱
# ==========================================================================

# 1. 安装依赖（适配CentOS/Ubuntu/Debian）
echo "===== 安装依赖 ====="
if ! sudo apt update && sudo apt install -y curl socat git; then
    echo "❌ 依赖安装失败"
    exit 1
fi

# 2. 安装acme.sh（使用Gitee镜像，适配中国大陆网络）
echo -e "\n===== 安装acme.sh ====="
ACME_HOME="$HOME/.acme.sh"
ACME_CMD="$ACME_HOME/acme.sh"

if [ ! -f "$ACME_CMD" ]; then
    echo "正在从 Gitee 镜像克隆 acme.sh..."
    
    # 创建临时目录用于克隆
    TMP_DIR=$(mktemp -d)
    cd $TMP_DIR
    
    # 使用 Gitee 镜像克隆
    if ! git clone https://gitee.com/neilpang/acme.sh.git; then
        echo "❌ Git 克隆失败"
        exit 1
    fi
    
    # 进入目录并安装
    cd acme.sh
    if ! ./acme.sh --install -m $ACME_EMAIL; then
        echo "❌ acme.sh 安装失败"
        exit 1
    fi
    
    # 清理临时目录
    cd ~
    rm -rf $TMP_DIR
    
    echo "✅ acme.sh 安装完成"
else
    echo "✅ acme.sh已安装，跳过"
fi

# 3. 创建证书目录（确保权限）
echo -e "\n===== 准备证书目录 ====="
sudo mkdir -p $CERT_DIR
# 关键修改：将证书目录的所有者改为当前用户，避免 acme.sh 在部署时因权限问题或 sudo 检测而失败
sudo chown -R $USER:$USER $CERT_DIR
sudo chmod 755 $CERT_DIR

# 4. 设置默认 CA 为 Let's Encrypt（避免 ZeroSSL 需要额外注册的问题）
echo -e "\n===== 设置证书颁发机构为 Let's Encrypt ====="
# 添加 || true 以忽略可能的警告或非关键错误（例如已经是 Let's Encrypt）
$ACME_CMD --set-default-ca --server letsencrypt || echo "⚠️ 设置默认 CA 时出现警告，继续执行..."

# 5. 准备 Webroot 验证目录（关键步骤：确保目录存在且当前用户可写）
echo -e "\n===== 准备 Webroot 验证目录 ====="
echo "Webroot 路径: $WEBROOT"

# 使用 sudo 创建目录，并修改所有者为当前用户，以确保 acme.sh 可以写入
if [ ! -d "$WEBROOT" ]; then
    echo "Webroot 目录不存在，正在创建..."
    sudo mkdir -p $WEBROOT
fi

# 创建 ACME 验证所需的子目录
ACME_CHALLENGE_DIR="$WEBROOT/.well-known/acme-challenge"
sudo mkdir -p "$ACME_CHALLENGE_DIR"

# 关键修改：将目录的所有者更改为当前用户 ($USER)
# 这样 acme.sh (以当前用户运行) 就有权限写入验证文件
echo "正在修改 $WEBROOT 权限以允许当前用户写入..."
sudo chown -R $USER:$USER $WEBROOT
sudo chmod -R 755 $WEBROOT

echo "✅ 验证目录已准备: $ACME_CHALLENGE_DIR"
echo "✅ 当前用户 $USER 已获得写入权限"

# 6. 申请证书（区分单域名/多域名）
echo -e "\n===== 申请Let's Encrypt证书 ====="
echo "使用 Webroot 模式，路径: $WEBROOT"
ISSUE_SUCCESS=false
if [ -z "$DOMAIN_ALT" ]; then
    # 单域名
    if $ACME_CMD --issue -d $DOMAIN --webroot $WEBROOT --force; then
        ISSUE_SUCCESS=true
    fi
else
    # 多域名
    if $ACME_CMD --issue -d $DOMAIN -d $DOMAIN_ALT --webroot $WEBROOT --force; then
        ISSUE_SUCCESS=true
    fi
fi

if [ "$ISSUE_SUCCESS" = false ]; then
    echo "❌ 证书申请失败，请检查日志"
    exit 1
fi

# 7. 部署证书到主机目录（自动适配Docker Nginx）
echo -e "\n===== 部署证书到主机 ====="
# 注意：这里不再使用 sudo，因为我们在第3步已经将 $CERT_DIR 的所有者改为了当前用户
if ! $ACME_CMD --install-cert -d $DOMAIN \
--key-file $CERT_DIR/$DOMAIN.key \
--fullchain-file $CERT_DIR/$DOMAIN.cer \
--reloadcmd "docker exec $DOCKER_NGINX_CONTAINER nginx -s reload"; then
    echo "❌ 证书部署失败"
    exit 1
fi

# 8. 验证证书部署结果
echo -e "\n===== 验证证书 ====="
if [ -f "$CERT_DIR/$DOMAIN.key" ] && [ -f "$CERT_DIR/$DOMAIN.cer" ]; then
    echo "✅ 证书生成成功！路径：$CERT_DIR"
    # 检查Docker Nginx是否重载成功
    if docker exec $DOCKER_NGINX_CONTAINER nginx -t &> /dev/null; then
        echo "✅ Docker Nginx配置验证通过，证书已生效"
    else
        echo "⚠️ Docker Nginx配置有误，请检查Nginx配置文件"
    fi
else
    echo "❌ 证书文件未找到，生成失败"
    exit 1
fi

# 9. 提示自动续期（acme.sh默认已开启）
echo -e "\n===== 后续说明 ====="
echo "✅ 自动续期已开启（每天检查，到期前30天自动续期并重载Nginx）"
echo "📌 手动续期命令：$ACME_CMD --renew -d $DOMAIN --force"
echo "📌 证书路径（主机）：$CERT_DIR/$DOMAIN.key | $CERT_DIR/$DOMAIN.cer"
echo "📌 确保Docker Nginx已挂载证书目录：-v $CERT_DIR:/etc/nginx/certs"
```

### 使用前必读

1. **修改自定义配置**：在运行脚本前，请务必修改以下变量：
   - `DOMAIN`: 你的主域名（如：example.com）
   - `DOMAIN_ALT`: 备用域名（可选，如：www.example.com）
   - `ACME_EMAIL`: 你的邮箱地址（用于证书过期通知）
   - `DOCKER_NGINX_CONTAINER`: 你的 Nginx 容器名称（通常为 `proxy-nginx`）

2. **网络环境说明**：此脚本使用 **Gitee 镜像**（`https://gitee.com/neilpang/acme.sh.git`），特别适合中国大陆用户。如果你在海外，可以考虑使用官方 GitHub 仓库。

3. **权限处理**：脚本会自动处理目录权限，确保 acme.sh 能够正常写入验证文件和证书文件。

4. **防火墙和端口要求**：
   - **80 端口必须开放**：Let's Encrypt 需要通过 HTTP-01 验证访问 `http://your-domain.com/.well-known/acme-challenge/`
   - **443 端口建议开放**：用于 HTTPS 服务
   - **确保防火墙已配置**：检查服务器防火墙（如 ufw、iptables）或云服务商安全组规则

5. **Nginx 服务状态**：
   - **确保 Nginx 已启动**：在运行脚本前，确保 `micro_proxy start` 已成功启动，Nginx 容器正在运行
   - **验证 Webroot 访问**：确保 `http://your-domain.com/.well-known/acme-challenge/` 路径可以被外部访问

## 手动安装步骤

如果你不想使用一键脚本，可以按照以下步骤手动安装：

### 前提条件
- Linux/Unix 系统
- curl 或 wget
- socat（用于 Standalone 模式）
- git
- **80 端口开放**：确保服务器 80 端口对外可访问（用于 Let's Encrypt 验证）
- **443 端口开放**：建议开放 443 端口（用于 HTTPS 服务）
- **Nginx 正在运行**：确保 micro_proxy 已启动，Nginx 容器正常运行

### 安装步骤

```bash
# 1. 安装依赖
sudo apt update && sudo apt install -y curl socat git

# 2. 克隆 acme.sh（使用 Gitee 镜像）
git clone https://gitee.com/neilpang/acme.sh.git
cd acme.sh

# 3. 安装 acme.sh
./acme.sh --install -m your@email.com

# 4. 重新加载 shell 配置
source ~/.bashrc
# 或者
source ~/.zshrc
```

### 验证安装

```bash
# 检查 acme.sh 是否安装成功
acme.sh --version

# 查看帮助信息
acme.sh --help
```

## 基本配置

ACME.sh 默认会将证书和相关文件存储在 `~/.acme.sh/` 目录下。你可以通过以下环境变量自定义配置：

```bash
# 设置默认邮箱（用于证书过期通知）
export ACCOUNT_EMAIL="your@email.com"

# 设置默认 CA（默认是 Let's Encrypt）
export CA="letsencrypt"
```

## 与 micro_proxy 集成

为了与 micro_proxy 配合使用，你需要确保：

1. **Web root 目录可访问**：ACME.sh 需要能够写入 `web_root` 目录（默认 `/var/www/html`）
2. **证书目录权限**：确保证书目录（默认 `/etc/nginx/certs`）有写入权限
3. **端口开放**：确保 80 和 443 端口在防火墙中开放
4. **Nginx 服务运行**：确保 Nginx 容器正在运行并能正确提供 Webroot 内容

### 环境变量配置示例

```bash
# 在你的 shell 配置文件中添加
export WEBROOT="/var/www/html"
export CERT_DIR="/etc/nginx/certs"
export DOMAIN="your-domain.com"
```

## 自动续期

ACME.sh 默认会自动设置 cron 任务来处理证书续期。你可以通过以下命令查看：

```bash
# 查看 cron 任务
crontab -l | grep acme.sh

# 手动测试续期（不会真正续期，只是测试）
acme.sh --renew --domain your-domain.com --dry-run
```

## 故障排除

### 常见问题

**1. 权限问题**
- 确保运行 acme.sh 的用户对 `web_root` 和 `cert_dir` 有读写权限
- 如果使用 Docker，确保容器内路径正确挂载

**2. 网络问题**
- **确保 80 端口开放**：Let's Encrypt 必须能够通过 80 端口访问验证文件
- **检查防火墙设置**：验证服务器防火墙和云服务商安全组规则
- **确保 Nginx 正在运行**：在申请证书前，确保 `micro_proxy start` 已成功执行

**3. 域名解析问题**
- 确保域名已正确解析到服务器 IP
- 使用 `dig your-domain.com` 或 `nslookup your-domain.com` 验证

**4. Webroot 访问问题**
- 验证 `http://your-domain.com/.well-known/acme-challenge/` 是否可访问
- 检查 Nginx 配置是否正确包含 ACME 验证 location

### 调试命令

```bash
# 启用详细日志
acme.sh --issue -d your-domain.com --webroot /var/www/html --debug

# 查看证书信息
acme.sh --info -d your-domain.com

# 测试 Webroot 访问（从外部）
curl http://your-domain.com/.well-known/acme-challenge/test

# 检查端口开放状态
telnet your-domain.com 80
telnet your-domain.com 443
```

### 防火墙配置示例

**UFW (Ubuntu):**
```bash
# 开放 80 和 443 端口
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw reload
```

**iptables:**
```bash
# 开放 80 和 443 端口
sudo iptables -A INPUT -p tcp --dport 80 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 443 -j ACCEPT
sudo iptables-save
```

**云服务商安全组**：
- AWS EC2: 在安全组中添加入站规则，允许 80 和 443 端口
- 阿里云: 在安全组中添加安全组规则，允许 80 和 443 端口
- 腾讯云: 在安全组中添加入站规则，允许 80 和 443 端口

## 参考链接

- [ACME.sh GitHub 仓库](https://github.com/acmesh-official/acme.sh)
- [ACME.sh Gitee 镜像](https://gitee.com/neilpang/acme.sh)
- [Let's Encrypt 官方文档](https://letsencrypt.org/docs/)
- [ACME 协议规范](https://tools.ietf.org/html/rfc8555)
