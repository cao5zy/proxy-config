
# SSL 证书申请指南

## 准备工作

在申请 SSL 证书之前，请确保完成以下准备工作：

### 1. 配置 proxy-config.yml

在 `proxy-config.yml` 中配置以下字段：

```yaml
# Web根目录（用于 ACME 验证）
web_root: "/var/www/html"

# 证书目录
cert_dir: "/etc/nginx/certs"

# 域名
domain: "your-domain.com"
```

### 2. 确保域名解析

- 确保你的域名（如 `your-domain.com`）已正确解析到服务器的公网 IP
- 可以使用以下命令验证：
  ```bash
  dig your-domain.com +short
  # 或
  nslookup your-domain.com
  ```

### 3. 开放必要端口

- 确保服务器的 80 端口（HTTP）和 443 端口（HTTPS）已开放
- Let's Encrypt 需要通过 80 端口进行 HTTP-01 验证

### 4. 验证 Web Root 访问

确保 Nginx 可以正确提供 `web_root` 目录的内容。micro_proxy 会自动生成包含以下配置的 Nginx 配置：

```nginx
location /.well-known/acme-challenge/ {
    root /var/www/html;
    default_type "text/plain";
}
```

## 申请证书

### 基本申请流程

```bash
#!/bin/bash
set -e

# 配置变量
DOMAIN="your-domain.com"
WEBROOT="/var/www/html"
CERT_DIR="/etc/nginx/certs"

# 创建证书目录
mkdir -p $CERT_DIR
chmod 755 $CERT_DIR

# 申请证书
acme.sh --issue -d $DOMAIN --webroot $WEBROOT

# 部署证书
acme.sh --install-cert -d $DOMAIN \
    --key-file $CERT_DIR/$DOMAIN.key \
    --fullchain-file $CERT_DIR/$DOMAIN.cer \
    --reloadcmd "docker exec proxy-nginx nginx -s reload"
```

### 申请带多个域名的证书

如果需要为多个域名（包括主域名和子域名）申请证书：

```bash
# 申请包含多个域名的证书
DOMAIN="your-domain.com"
DOMAIN_ALT="www.your-domain.com"
WEBROOT="/var/www/html"
CERT_DIR="/etc/nginx/certs"

acme.sh --issue -d $DOMAIN -d $DOMAIN_ALT --webroot $WEBROOT

# 部署证书
acme.sh --install-cert -d $DOMAIN \
    --key-file $CERT_DIR/$DOMAIN.key \
    --fullchain-file $CERT_DIR/$DOMAIN.cer \
    --reloadcmd "docker exec proxy-nginx nginx -s reload"
```

## 验证证书

### 1. 检查证书文件

确保证书文件已正确生成：

```bash
ls -la /etc/nginx/certs/
# 应该看到 your-domain.com.key 和 your-domain.com.cer 文件
```

### 2. 测试 HTTPS 访问

启动 micro_proxy 并测试 HTTPS 访问：

```bash
# 启动服务
micro_proxy start

# 测试 HTTPS
curl -I https://your-domain.com
```

### 3. 验证证书信息

使用 OpenSSL 验证证书信息：

```bash
openssl x509 -in /etc/nginx/certs/your-domain.com.cer -text -noout
```

## 自动续期

ACME.sh 默认会自动处理证书续期。续期过程如下：

1. **自动检测**：ACME.sh 会在证书到期前 30 天自动尝试续期
2. **验证过程**：使用相同的 Web Root 验证方式
3. **部署更新**：续期成功后，自动执行 `--reloadcmd` 重新加载 Nginx

### 手动触发续期

```bash
# 强制续期（即使证书未到期）
acme.sh --renew -d your-domain.com --force

# 测试续期（不实际执行）
acme.sh --renew -d your-domain.com --dry-run
```

## 故障排除

### 证书申请失败

**常见错误及解决方案：**

1. **"Domain not validated"**
   - 检查域名是否正确解析到服务器
   - 确保 80 端口可访问
   - 验证 Web Root 目录权限

2. **"Permission denied"**
   - 检查 `web_root` 和 `cert_dir` 目录权限
   - 确保运行 acme.sh 的用户有写入权限

3. **"Connection refused"**
   - 检查防火墙设置
   - 确保 Nginx 正在运行并监听 80 端口

### 调试命令

```bash
# 启用详细日志
acme.sh --issue -d your-domain.com --webroot /var/www/html --debug 2

# 查看证书状态
acme.sh --list

# 查看特定域名的证书信息
acme.sh --info -d your-domain.com
```

## 最佳实践

1. **使用非 root 用户运行**：避免使用 root 用户运行 acme.sh
2. **定期备份证书**：定期备份 `/etc/nginx/certs/` 目录
3. **监控证书过期**：设置监控告警，确保证书不会意外过期
4. **测试环境验证**：在生产环境之前，先在测试环境验证整个流程

## 参考资源

- [Let's Encrypt Rate Limits](https://letsencrypt.org/docs/rate-limits/)
- [ACME.sh Wiki](https://github.com/acmesh-official/acme.sh/wiki)
- [Nginx SSL Configuration Guide](https://nginx.org/en/docs/http/configuring_https_servers.html)
