
//! Nginx配置生成模块
//!
//! 负责根据配置生成nginx.conf
//! 使用动态DNS解析，避免因某个服务不可用导致nginx启动失败

use crate::{config::AppConfig, config::AppType, Error, Result};
use std::path::Path;

/// 生成nginx配置
///
/// # 参数
/// - `apps`: 应用配置列表（会自动过滤掉 Internal 类型的应用）
/// - `web_root`: Web根目录，用于存放ACME验证文件
/// - `cert_dir`: 证书目录
/// - `domain`: 域名（可选）
///
/// # 返回
/// 返回生成的nginx配置内容
///
/// # 注意
/// nginx.conf 中的 listen 端口固定为容器内部端口：
/// - HTTP: 80
/// - HTTPS: 443
/// 宿主机端口映射由 docker-compose.yml 控制
pub fn generate_nginx_config(
    apps: &[AppConfig],
    web_root: &str,
    cert_dir: &str,
    domain: &Option<String>,
) -> Result<String> {
    log::info!("开始生成nginx配置");
    log::debug!("web_root: {}", web_root);
    log::debug!("cert_dir: {}", cert_dir);
    log::debug!("domain: {:?}", domain);
    log::debug!("应用数量: {}", apps.len());

    // 过滤掉 Internal 类型的应用
    let nginx_apps: Vec<&AppConfig> = apps
        .iter()
        .filter(|app| app.app_type != AppType::Internal)
        .collect();

    log::debug!("需要nginx代理的应用数量: {}", nginx_apps.len());
    for app in &nginx_apps {
        log::debug!("  - {} ({})", app.name, format_app_type(&app.app_type));
    }

    let mut config = String::new();

    // 生成配置头部
    config.push_str(&generate_header());

    // 检查是否配置了域名且证书存在
    let ssl_enabled = if let Some(ref domain_name) = domain {
        check_ssl_certificates(cert_dir, domain_name)
    } else {
        false
    };

    if ssl_enabled {
        log::info!("检测到SSL证书，启用HTTPS配置");
        // 生成HTTP server块（重定向到HTTPS + ACME验证）
        config.push_str("\n");
        config.push_str(&generate_http_redirect_server_block(
            web_root,
            domain.as_ref().unwrap(), // unwrap is safe here because ssl_enabled is true
        ));

        // 生成HTTPS server块
        config.push_str("\n");
        config.push_str(&generate_https_server_block(
            &nginx_apps,
            web_root,
            cert_dir,
            domain.as_ref().unwrap(), // unwrap is safe here because ssl_enabled is true
        ));
    } else {
        log::info!("未检测到SSL证书，仅启用HTTP配置");
        // 生成HTTP server块（包含业务逻辑）
        config.push_str("\n");
        config.push_str(&generate_http_server_block(
            &nginx_apps,
            web_root,
        ));
    }

    // 闭合 http 块
    config.push_str("}\n");

    log::info!("nginx配置生成完成");
    log::debug!("生成的配置:\n{}", config);

    Ok(config)
}

/// 检查SSL证书文件是否存在
///
/// # 参数
/// - `cert_dir`: 证书目录
/// - `domain`: 域名
///
/// # 返回
/// 如果证书和密钥文件都存在，返回 true，否则返回 false
fn check_ssl_certificates(cert_dir: &str, domain: &str) -> bool {
    // 尝试常见的证书文件扩展名
    let cert_extensions = vec![".cer", ".crt"];
    let mut cert_exists = false;

    for ext in &cert_extensions {
        let cert_path = Path::new(cert_dir).join(format!("{}{}", domain, ext));
        log::debug!("检查证书文件: {:?}", cert_path);
        if cert_path.exists() {
            cert_exists = true;
            break;
        }
    }

    if !cert_exists {
        log::debug!("未找到证书文件: {}/{}{{.cer,.crt}}", cert_dir, domain);
        return false;
    }

    // 检查密钥文件
    let key_path = Path::new(cert_dir).join(format!("{}.key", domain));
    log::debug!("检查密钥文件: {:?}", key_path);
    if key_path.exists() {
        log::info!("找到SSL证书和密钥文件");
        true
    } else {
        log::debug!("未找到密钥文件: {:?}", key_path);
        false
    }
}

/// 格式化应用类型用于日志输出
fn format_app_type(app_type: &AppType) -> &'static str {
    match app_type {
        AppType::Static => "Static",
        AppType::Api => "Api",
        AppType::Internal => "Internal",
    }
}

/// 生成配置头部（纯函数）
///
/// # 返回
/// 返回配置头部内容
fn generate_header() -> String {
    log::debug!("生成nginx配置头部");

    format!(
        r#"# Nginx配置文件
# 由proxy-config自动生成
# 生成时间: {}

worker_processes auto;

events {{
    worker_connections 1024;
}}

http {{
    include       /etc/nginx/mime.types;
    default_type  application/octet-stream;

    log_format  main  '$remote_addr - $remote_user [$time_local] "$request" '
                      '$status $body_bytes_sent "$http_referer" '
                      '"$http_user_agent" "$http_x_forwarded_for"';

    access_log  /var/log/nginx/access.log  main;
    error_log   /var/log/nginx/error.log warn;

    sendfile        on;
    tcp_nopush      on;
    tcp_nodelay     on;
    keepalive_timeout  65;
    types_hash_max_size 2048;

    # Gzip压缩
    gzip on;
    gzip_vary on;
    gzip_proxied any;
    gzip_comp_level 6;
    gzip_types text/plain text/css text/xml text/javascript 
               application/json application/javascript application/xml+rss 
               application/rss+xml font/truetype font/opentype 
               application/vnd.ms-fontobject image/svg+xml;

    # 使用Docker内部DNS解析器，支持动态DNS解析
    # valid=30s: DNS解析结果缓存30秒
    # ipv6=off: 禁用IPv6解析，避免解析延迟
    resolver 127.0.0.11 valid=30s ipv6=off;

"#,
        chrono::Utc::now().to_rfc3339()
    )
}

/// 生成HTTP重定向Server配置块（纯函数）
/// 用于HTTPS场景下的HTTP端口，将非ACME请求重定向到HTTPS
///
/// # 参数
/// - `web_root`: Web根目录，用于存放ACME验证文件
/// - `domain`: 域名
///
/// # 返回
/// 返回server配置块内容
///
/// # 注意
/// listen 端口固定为 80（容器内部端口）
fn generate_http_redirect_server_block(web_root: &str, domain: &str) -> String {
    log::debug!("生成HTTP重定向server配置块，监听端口: 80, 域名: {}", domain);

    format!(
        r#"    server {{
        listen 80;
        server_name {};

        # ACME 验证（Let's Encrypt 证书申请）
        # acme.sh 会在此目录下创建验证文件
        location /.well-known/acme-challenge/ {{
            root {};
            default_type "text/plain";
        }}

        # 其他所有请求重定向到HTTPS
        location / {{
            return 301 https://$host$request_uri;
        }}
    }}
"#,
        domain, web_root
    )
}

/// 生成HTTP Server配置块（纯函数）
/// 用于非HTTPS场景，包含业务逻辑
///
/// # 参数
/// - `apps`: 应用配置列表（已过滤掉 Internal 类型）
/// - `web_root`: Web根目录，用于存放ACME验证文件
///
/// # 返回
/// 返回server配置块内容
///
/// # 注意
/// listen 端口固定为 80（容器内部端口）
fn generate_http_server_block(
    apps: &[&AppConfig],
    web_root: &str,
) -> String {
    log::debug!("生成HTTP server配置块，监听端口: 80");
    generate_server_block_internal(apps, 80, web_root, false, None, None)
}

/// 生成HTTPS Server配置块（纯函数）
/// 用于HTTPS场景，包含业务逻辑
///
/// # 参数
/// - `apps`: 应用配置列表（已过滤掉 Internal 类型）
/// - `web_root`: Web根目录，用于存放ACME验证文件
/// - `cert_dir`: 证书目录
/// - `domain`: 域名
///
/// # 返回
/// 返回server配置块内容
fn generate_https_server_block(
    apps: &[&AppConfig],
    web_root: &str,
    cert_dir: &str,
    domain: &str,
) -> String {
    log::debug!("生成HTTPS server配置块，监听端口: 443, 域名: {}", domain);
    generate_server_block_internal(apps, 443, web_root, true, Some(cert_dir), Some(domain))
}

/// 生成Server配置块内部实现（纯函数）
///
/// # 参数
/// - `apps`: 应用配置列表（已过滤掉 Internal 类型）
/// - `port`: 监听端口（容器内部端口）
/// - `web_root`: Web根目录，用于存放ACME验证文件
/// - `is_https`: 是否为HTTPS
/// - `cert_dir`: 证书目录（HTTPS时需要）
/// - `domain`: 域名（HTTPS时需要）
///
/// # 返回
/// 返回server配置块内容
fn generate_server_block_internal(
    apps: &[&AppConfig],
    port: u16,
    web_root: &str,
    is_https: bool,
    cert_dir: Option<&str>,
    domain: Option<&str>,
) -> String {
    log::debug!(
        "生成server配置块，监听端口: {}, HTTPS: {}",
        port,
        is_https
    );
    log::debug!("web_root: {}", web_root);

    let mut server_config = String::new();

    // SSL 配置
    let ssl_config = if is_https {
        let cert_dir = cert_dir.unwrap(); // safe unwrap
        let domain = domain.unwrap();     // safe unwrap
        
        // 尝试确定证书文件扩展名
        let cert_ext = if Path::new(cert_dir).join(format!("{}.cer", domain)).exists() {
            ".cer"
        } else {
            ".crt"
        };

        format!(
            r#"        listen 443 ssl;
        server_name {};

        # SSL 证书配置
        ssl_certificate {}/{}{};
        ssl_certificate_key {}/{}.key;

        # SSL 优化配置
        ssl_protocols TLSv1.2 TLSv1.3;
        ssl_ciphers HIGH:!aNULL:!MD5;
        ssl_prefer_server_ciphers on;
        ssl_session_cache shared:SSL:10m;
        ssl_session_timeout 10m;

"#,
            domain, cert_dir, domain, cert_ext, cert_dir, domain
        )
    } else {
        format!(
            r#"        listen {};
        server_name localhost;

"#,
            port
        )
    };

    server_config.push_str("    server {\n");
    server_config.push_str(&ssl_config);

    // 仅在 HTTP 模式下添加 ACME 验证 location
    // HTTPS 模式下，ACME 验证在 HTTP 重定向 server 块中处理
    if !is_https {
        server_config.push_str(&format!(
            r#"        # ACME 验证（Let's Encrypt 证书申请）
        # acme.sh 会在此目录下创建验证文件
        location /.well-known/acme-challenge/ {{
            root {};
            default_type "text/plain";
        }}

"#,
            web_root
        ));
    }

    // 为每个应用定义服务地址变量
    // 这些变量将在location中使用，实现动态DNS解析
    for app in apps {
        let upstream_host_var = format!("{}_upstream_host", app.name);
        let set_line = format!(
            r#"        # 定义{}服务地址变量
        set ${} {};

"#,
            app.name, upstream_host_var, app.container_name
        );
        log::debug!("生成set指令: {}", set_line.trim());
        server_config.push_str(&set_line);
    }

    // 检查是否有应用使用了根路径 "/"
    let has_root_route = apps
        .iter()
        .any(|app| app.routes.iter().any(|route| route == "/"));

    // 如果没有应用使用根路径，则生成默认的404 location
    if !has_root_route {
        server_config.push_str(
            r#"        # 默认返回404
        location / {
            return 404;
        }

"#,
        );
    }

    // 收集所有location配置，并按路径长度降序排序（更具体的路径优先）
    let mut location_configs: Vec<(String, String)> = Vec::new();
    for app in apps {
        for route in &app.routes {
            location_configs.push((route.clone(), generate_location_config(app, route)));
        }
    }

    // 按路径长度降序排序，确保更具体的路径（如 /resume_app）在通用路径（如 /）之前
    location_configs.sort_by(|a, b| {
        let len_a = a.0.len();
        let len_b = b.0.len();
        len_b.cmp(&len_a) // 降序排序
    });

    log::debug!("location配置排序结果:");
    for (route, _) in &location_configs {
        log::debug!("  - {}", route);
    }

    // 生成location配置
    for (_, config) in location_configs {
        server_config.push_str(&config);
    }

    server_config.push_str("    }\n");

    server_config
}

/// 生成location配置（纯函数）
///
/// # 参数
/// - `app`: 应用配置
/// - `route`: 路由路径
///
/// # 返回
/// 返回location配置内容
fn generate_location_config(app: &AppConfig, route: &str) -> String {
    log::debug!("生成location配置: app={}, route={}", app.name, route);

    let mut location = String::new();

    // 判断是否为根路径
    let is_root_route = route == "/";

    // 使用变量实现动态DNS解析
    // 变量名格式: {app_name}_upstream_host
    let upstream_host_var = format!("{}_upstream_host", app.name);

    match app.app_type {
        AppType::Static => {
            // 对于静态资源服务：
            // 1. 如果是根路径，直接转发，不修改URI
            // 2. 如果是非根路径（如 /resume_app），需要特殊处理：
            //    - 访问 /resume_app 时，重写为 /，然后转发到后端根路径
            //    - 访问 /resume_app/ 时，重写为 /，然后转发到后端根路径
            //    - 访问 /resume_app/assets/... 时，重写为 /assets/...，然后转发
            //    这样可以支持前端使用 VITE_BASE_URL=/resume_app 的配置

            let (proxy_pass_url, rewrite_rule) = if is_root_route {
                // 根路径：直接转发
                (
                    format!("http://${{{}}}:{}", upstream_host_var, app.container_port),
                    String::new(),
                )
            } else {
                // 非根路径：使用 rewrite 处理
                // ^/resume_app(/.*)?$ 匹配：
                //   - /resume_app -> $1 为空 -> 重写为 /
                //   - /resume_app/ -> $1 为 / -> 重写为 /
                //   - /resume_app/assets/xxx -> $1 为 /assets/xxx -> 重写为 /assets/xxx
                let rewrite_pattern = format!("^{}(/.*)?$", route);
                let rewrite_target = "/$1";
                let rewrite_rule = format!(
                    "            rewrite {} {} break;\n",
                    rewrite_pattern, rewrite_target
                );
                (
                    format!("http://${{{}}}:{}", upstream_host_var, app.container_port),
                    rewrite_rule,
                )
            };

            location.push_str(&format!(
                r#"        # 静态网站: {}
        location {} {{
{}            proxy_pass {};
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;

            # 静态文件缓存
            expires 7d;
            add_header Cache-Control "public, immutable";
        }}

"#,
                app.name, route, rewrite_rule, proxy_pass_url
            ));
        }
        AppType::Api => {
            // 对于API服务，直接转发完整的请求URI
            // 不添加尾部 /，确保后端收到完整的路径（如 /api/v1/status）
            let proxy_pass_url =
                format!("http://${{{}}}:{}", upstream_host_var, app.container_port);

            location.push_str(&format!(
                r#"        # API服务: {}
        location {} {{
            proxy_pass {};
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;

            # API超时设置
            proxy_connect_timeout 60s;
            proxy_send_timeout 60s;
            proxy_read_timeout 60s;

            # 禁用API缓存
            expires off;
            add_header Cache-Control "no-cache, no-store, must-revalidate";
"#,
                app.name, route, proxy_pass_url
            ));

            // 添加额外的nginx配置
            if let Some(extra_config) = &app.nginx_extra_config {
                for line in extra_config.lines() {
                    location.push_str(&format!("            {}\n", line));
                }
            }

            location.push_str("        }\n\n");
        }
        AppType::Internal => {
            // Internal 类型不应该生成 nginx 配置
            log::warn!(
                "Internal 应用 '{}' 不应该生成 nginx location 配置",
                app.name
            );
        }
    }

    location
}

/// 保存nginx配置到文件
///
/// # 参数
/// - `config`: nginx配置内容
/// - `output_path`: 输出文件路径
///
/// # 返回
/// 返回保存结果
pub fn save_nginx_config(config: &str, output_path: &str) -> Result<()> {
    log::info!("保存nginx配置到文件: {}", output_path);

    std::fs::write(output_path, config).map_err(|e| {
        log::error!("写入nginx配置文件失败: {}, 错误: {}", output_path, e);
        Error::Nginx(format!("写入nginx配置文件失败: {}", e))
    })?;

    log::info!("nginx配置文件保存成功: {}", output_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_header() {
        let header = generate_header();
        assert!(header.contains("worker_processes auto;"));
        // 检查是否包含resolver指令
        assert!(header.contains("resolver 127.0.0.11 valid=30s ipv6=off;"));
    }

    #[test]
    fn test_generate_location_config_static_root() {
        let app = AppConfig {
            name: "test-app".to_string(),
            routes: vec!["/".to_string()],
            container_name: "test-container".to_string(),
            container_port: 80,
            app_type: AppType::Static,
            description: None,
            nginx_extra_config: None,
            path: None,
        };

        let location = generate_location_config(&app, "/");
        assert!(location.contains("location /"));
        // 应该使用变量
        assert!(location.contains("proxy_pass http://${test-app_upstream_host}:80;"));
        // 根路径不应该有尾部的 /
        assert!(!location.contains("proxy_pass http://${test-app_upstream_host}:80/;"));
        // 根路径不应该有 rewrite 规则
        assert!(!location.contains("rewrite"));
        assert!(location.contains("expires 7d;"));
    }

    #[test]
    fn test_generate_location_config_static_subpath() {
        let app = AppConfig {
            name: "resume-app".to_string(),
            routes: vec!["/resume_app".to_string()],
            container_name: "resume-container".to_string(),
            container_port: 80,
            app_type: AppType::Static,
            description: None,
            nginx_extra_config: None,
            path: None,
        };

        let location = generate_location_config(&app, "/resume_app");
        assert!(location.contains("location /resume_app"));
        // 应该使用变量
        assert!(location.contains("proxy_pass http://${resume-app_upstream_host}:80;"));
        // 静态资源服务不应该有尾部的 /
        assert!(!location.contains("proxy_pass http://${resume-app_upstream_host}:80/;"));
        // 应该有 rewrite 规则，使用可选分组
        assert!(location.contains("rewrite ^/resume_app(/.*)?$ /$1 break;"));
        assert!(location.contains("expires 7d;"));
    }

    #[test]
    fn test_generate_location_config_api() {
        let app = AppConfig {
            name: "api-service".to_string(),
            routes: vec!["/api".to_string()],
            container_name: "api-container".to_string(),
            container_port: 3000,
            app_type: AppType::Api,
            description: None,
            nginx_extra_config: Some("add_header 'Access-Control-Allow-Origin' '*';".to_string()),
            path: None,
        };

        let location = generate_location_config(&app, "/api");
        assert!(location.contains("location /api"));
        // 应该使用变量
        assert!(location.contains("proxy_pass http://${api-service_upstream_host}:3000;"));
        // API服务不应该有尾部的 /
        assert!(!location.contains("proxy_pass http://${api-service_upstream_host}:3000/;"));
        assert!(location.contains("expires off;"));
        assert!(location.contains("add_header 'Access-Control-Allow-Origin' '*';"));
    }

    #[test]
    fn test_generate_nginx_config_http_only() {
        let apps = vec![
            AppConfig {
                name: "main-app".to_string(),
                routes: vec!["/".to_string()],
                container_name: "main-container".to_string(),
                container_port: 80,
                app_type: AppType::Static,
                description: None,
                nginx_extra_config: None,
                path: None,
            },
            AppConfig {
                name: "api-service".to_string(),
                routes: vec!["/api".to_string()],
                container_name: "api-container".to_string(),
                container_port: 3000,
                app_type: AppType::Api,
                description: None,
                nginx_extra_config: None,
                path: None,
            },
        ];

        // 没有配置 domain，应该生成 HTTP 配置
        let config = generate_nginx_config(&apps, "/var/www/html", "/etc/nginx/certs", &None).unwrap();

        // 打印完整配置以便调试
        println!("=== 完整生成的配置 (HTTP Only) ===");
        println!("{}", config);
        println!("=== 配置结束 ===");

        assert!(config.contains("worker_processes auto;"));
        // nginx.conf 中的 listen 应该是容器内部端口 80
        assert!(config.contains("listen 80;"));
        assert!(!config.contains("listen 8080;"));
        assert!(!config.contains("listen 443 ssl;"));
        assert!(config.contains("location /"));
        assert!(config.contains("location /api"));

        // 检查是否包含 ACME 验证 location
        assert!(config.contains("location /.well-known/acme-challenge/"));
        assert!(config.contains("root /var/www/html;"));

        // 检查是否包含resolver指令
        assert!(config.contains("resolver 127.0.0.11 valid=30s ipv6=off;"));

        // 检查是否包含set指令定义变量
        assert!(config.contains("set $main-app_upstream_host main-container;"));
        assert!(config.contains("set $api-service_upstream_host api-container;"));

        // 检查location是否使用变量
        assert!(config.contains("proxy_pass http://${main-app_upstream_host}:80;"));
        assert!(config.contains("proxy_pass http://${api-service_upstream_host}:3000;"));

        // 不应该包含upstream块（检查 "upstream " 后面有空格，或者 "upstream {"）
        assert!(!config.contains("upstream "));
        assert!(!config.contains("upstream {"));

        // 不应该包含resolve参数
        assert!(!config.contains("resolve "));

        // 当有应用使用根路径时，不应该有默认的404 location
        assert!(!config.contains("return 404;"));

        // 检查location顺序：/.well-known/acme-challenge/ 应该在最前面
        let acme_pos = config
            .find("location /.well-known/acme-challenge/")
            .unwrap();
        let api_pos = config.find("location /api ").unwrap();
        let root_pos = config.find("location / ").unwrap();
        assert!(acme_pos < api_pos, "ACME location 应该在最前面");
        assert!(api_pos < root_pos, "location /api 应该在 location / 之前");

        // 检查 http 块是否正确闭合
        assert!(config.ends_with("}\n"));
    }

    #[test]
    fn test_generate_nginx_config_https() {
        let apps = vec![
            AppConfig {
                name: "main-app".to_string(),
                routes: vec!["/".to_string()],
                container_name: "main-container".to_string(),
                container_port: 80,
                app_type: AppType::Static,
                description: None,
                nginx_extra_config: None,
                path: None,
            },
            AppConfig {
                name: "api-service".to_string(),
                routes: vec!["/api".to_string()],
                container_name: "api-container".to_string(),
                container_port: 3000,
                app_type: AppType::Api,
                description: None,
                nginx_extra_config: None,
                path: None,
            },
        ];

        // 创建临时证书文件
        let temp_dir = tempfile::tempdir().unwrap();
        let cert_dir = temp_dir.path().to_str().unwrap();
        let domain = "example.com";
        
        // 创建 .cer 文件
        std::fs::write(
            temp_dir.path().join(format!("{}.cer", domain)),
            "fake cert"
        ).unwrap();
        // 创建 .key 文件
        std::fs::write(
            temp_dir.path().join(format!("{}.key", domain)),
            "fake key"
        ).unwrap();

        // 配置了 domain 且证书存在，应该生成 HTTPS 配置
        let config = generate_nginx_config(&apps, "/var/www/html", cert_dir, &Some(domain.to_string())).unwrap();

        // 打印完整配置以便调试
        println!("=== 完整生成的配置 (HTTPS) ===");
        println!("{}", config);
        println!("=== 配置结束 ===");

        assert!(config.contains("worker_processes auto;"));
        
        // 应该包含 HTTP 重定向 server 块，监听容器内部端口 80
        assert!(config.contains("listen 80;"));
        assert!(!config.contains("listen 8080;"));
        assert!(config.contains("return 301 https://$host$request_uri;"));
        
        // 应该包含 HTTPS server 块
        assert!(config.contains("listen 443 ssl;"));
        assert!(config.contains(&format!("ssl_certificate {}/{}.cer", cert_dir, domain)));
        assert!(config.contains(&format!("ssl_certificate_key {}/{}.key", cert_dir, domain)));
        
        // 业务逻辑应该在 HTTPS server 块中
        assert!(config.contains("location /"));
        assert!(config.contains("location /api"));

        // 检查 ACME 验证 location 是否在 HTTP 块中
        // HTTP 块应该有 ACME 验证
        assert!(config.contains("location /.well-known/acme-challenge/"));
        
        // HTTPS 块不应该有 ACME 验证 location (因为我们在 generate_https_server_block 中没有添加)
        // 但我们需要确认 ACME location 确实只在 HTTP 块中
        // 简单的检查：配置中应该只有一个 ACME location
        let acme_count = config.matches("location /.well-known/acme-challenge/").count();
        assert_eq!(acme_count, 1, "应该只有一个 ACME location (在 HTTP 块中)");

        // 检查 server_name 是否正确设置为域名
        assert!(config.contains("server_name example.com;"));

        // 检查 http 块是否正确闭合
        assert!(config.ends_with("}\n"));
    }

    #[test]
    fn test_generate_nginx_config_with_internal() {
        let apps = vec![
            AppConfig {
                name: "main-app".to_string(),
                routes: vec!["/".to_string()],
                container_name: "main-container".to_string(),
                container_port: 80,
                app_type: AppType::Static,
                description: None,
                nginx_extra_config: None,
                path: None,
            },
            AppConfig {
                name: "redis".to_string(),
                routes: vec![],
                container_name: "redis-container".to_string(),
                container_port: 6379,
                app_type: AppType::Internal,
                description: None,
                nginx_extra_config: None,
                path: Some("./services/redis".to_string()),
            },
            AppConfig {
                name: "api-service".to_string(),
                routes: vec!["/api".to_string()],
                container_name: "api-container".to_string(),
                container_port: 3000,
                app_type: AppType::Api,
                description: None,
                nginx_extra_config: None,
                path: None,
            },
        ];

        let config = generate_nginx_config(&apps, "/var/www/html", "/etc/nginx/certs", &None).unwrap();

        // 打印完整配置以便调试
        println!("=== 完整生成的配置 ===");
        println!("{}", config);
        println!("=== 配置结束 ===");

        assert!(config.contains("worker_processes auto;"));
        // nginx.conf 中的 listen 应该是容器内部端口 80
        assert!(config.contains("listen 80;"));
        assert!(!config.contains("listen 8080;"));
        assert!(config.contains("location /"));
        assert!(config.contains("location /api"));

        // 检查是否包含 ACME 验证 location
        assert!(config.contains("location /.well-known/acme-challenge/"));
        assert!(config.contains("root /var/www/html;"));

        // 检查是否包含resolver指令
        assert!(config.contains("resolver 127.0.0.11 valid=30s ipv6=off;"));

        // 检查是否包含set指令定义变量（不应该包含 redis 的变量）
        assert!(config.contains("set $main-app_upstream_host main-container;"));
        assert!(config.contains("set $api-service_upstream_host api-container;"));
        assert!(!config.contains("set $redis_upstream_host"));

        // 检查location是否使用变量
        assert!(config.contains("proxy_pass http://${main-app_upstream_host}:80;"));
        assert!(config.contains("proxy_pass http://${api-service_upstream_host}:3000;"));

        // 不应该包含upstream块（检查 "upstream " 后面有空格，或者 "upstream {"）
        assert!(!config.contains("upstream "));
        assert!(!config.contains("upstream {"));

        // 不应该包含resolve参数
        assert!(!config.contains("resolve "));

        // 当有应用使用根路径时，不应该有默认的404 location
        assert!(!config.contains("return 404;"));

        // 检查location顺序：/.well-known/acme-challenge/ 应该在最前面
        let acme_pos = config
            .find("location /.well-known/acme-challenge/")
            .unwrap();
        let api_pos = config.find("location /api ").unwrap();
        let root_pos = config.find("location / ").unwrap();
        assert!(acme_pos < api_pos, "ACME location 应该在最前面");
        assert!(api_pos < root_pos, "location /api 应该在 location / 之前");

        // 检查 http 块是否正确闭合
        assert!(config.ends_with("}\n"));
    }

    #[test]
    fn test_generate_nginx_config_without_root_route() {
        let apps = vec![AppConfig {
            name: "api-service".to_string(),
            routes: vec!["/api".to_string()],
            container_name: "api-container".to_string(),
            container_port: 3000,
            app_type: AppType::Api,
            description: None,
            nginx_extra_config: None,
            path: None,
        }];

        let config = generate_nginx_config(&apps, "/var/www/html", "/etc/nginx/certs", &None).unwrap();
        // 当没有应用使用根路径时，应该有默认的404 location
        assert!(config.contains("return 404;"));
        // 应该包含 ACME 验证 location
        assert!(config.contains("location /.well-known/acme-challenge/"));

        // 检查 http 块是否正确闭合
        assert!(config.ends_with("}\n"));
    }

    #[test]
    fn test_location_ordering() {
        let apps = vec![
            AppConfig {
                name: "root-app".to_string(),
                routes: vec!["/".to_string()],
                container_name: "root-container".to_string(),
                container_port: 80,
                app_type: AppType::Static,
                description: None,
                nginx_extra_config: None,
                path: None,
            },
            AppConfig {
                name: "resume-app".to_string(),
                routes: vec!["/resume_app".to_string()],
                container_name: "resume-container".to_string(),
                container_port: 80,
                app_type: AppType::Static,
                description: None,
                nginx_extra_config: None,
                path: None,
            },
            AppConfig {
                name: "api-service".to_string(),
                routes: vec!["/api".to_string()],
                container_name: "api-container".to_string(),
                container_port: 3000,
                app_type: AppType::Api,
                description: None,
                nginx_extra_config: None,
                path: None,
            },
        ];

        let config = generate_nginx_config(&apps, "/var/www/html", "/etc/nginx/certs", &None).unwrap();

        // 检查location顺序：/.well-known/acme-challenge/ 应该在最前面
        let acme_pos = config
            .find("location /.well-known/acme-challenge/")
            .unwrap();
        let resume_pos = config.find("location /resume_app ").unwrap();
        let api_pos = config.find("location /api ").unwrap();
        let root_pos = config.find("location / ").unwrap();

        assert!(acme_pos < resume_pos, "ACME location 应该在最前面");
        assert!(
            resume_pos < api_pos,
            "location /resume_app 应该在 location /api 之前"
        );
        assert!(api_pos < root_pos, "location /api 应该在 location / 之前");

        // 检查 http 块是否正确闭合
        assert!(config.ends_with("}\n"));
    }

    #[test]
    fn test_static_rewrite_rule() {
        // 测试静态资源服务的 rewrite 规则
        let app = AppConfig {
            name: "resume-app".to_string(),
            routes: vec!["/resume_app".to_string()],
            container_name: "resume-container".to_string(),
            container_port: 80,
            app_type: AppType::Static,
            description: None,
            nginx_extra_config: None,
            path: None,
        };

        let location = generate_location_config(&app, "/resume_app");

        // 应该有 rewrite 规则，使用可选分组
        assert!(location.contains("rewrite ^/resume_app(/.*)?$ /$1 break;"));
        // proxy_pass 不应该有尾部的 /
        assert!(location.contains("proxy_pass http://${resume-app_upstream_host}:80;"));
        assert!(!location.contains("proxy_pass http://${resume-app_upstream_host}:80/;"));
    }

    #[test]
    fn test_api_preserves_path() {
        // 测试API服务保留完整路径
        let app = AppConfig {
            name: "api-service".to_string(),
            routes: vec!["/api".to_string()],
            container_name: "api-container".to_string(),
            container_port: 3000,
            app_type: AppType::Api,
            description: None,
            nginx_extra_config: None,
            path: None,
        };

        let location = generate_location_config(&app, "/api");

        // API服务应该保留完整路径，proxy_pass 不应该有尾部的 /
        assert!(location.contains("proxy_pass http://${api-service_upstream_host}:3000;"));
        assert!(!location.contains("proxy_pass http://${api-service_upstream_host}:3000/;"));
        // API服务不应该有 rewrite 规则
        assert!(!location.contains("rewrite"));
    }

    #[test]
    fn test_only_internal_apps() {
        // 测试只有 Internal 应用的场景
        let apps = vec![AppConfig {
            name: "redis".to_string(),
            routes: vec![],
            container_name: "redis-container".to_string(),
            container_port: 6379,
            app_type: AppType::Internal,
            description: None,
            nginx_extra_config: None,
            path: Some("./services/redis".to_string()),
        }];

        let config = generate_nginx_config(&apps, "/var/www/html", "/etc/nginx/certs", &None).unwrap();

        // 应该生成基本的 nginx 配置，但没有应用 location
        assert!(config.contains("worker_processes auto;"));
        // nginx.conf 中的 listen 应该是容器内部端口 80
        assert!(config.contains("listen 80;"));
        assert!(!config.contains("listen 8080;"));
        // 应该有 ACME 验证 location
        assert!(config.contains("location /.well-known/acme-challenge/"));
        // 应该有默认的404 location
        assert!(config.contains("return 404;"));
        // 不应该有任何应用的 set 指令
        assert!(!config.contains("set $"));

        // 检查 http 块是否正确闭合
        assert!(config.ends_with("}\n"));
    }

    #[test]
    fn test_acme_location() {
        // 测试 ACME 验证 location 的生成
        let apps = vec![];

        let config = generate_nginx_config(&apps, "/custom/web/root", "/etc/nginx/certs", &None).unwrap();

        // 应该包含 ACME 验证 location
        assert!(config.contains("location /.well-known/acme-challenge/"));
        assert!(config.contains("root /custom/web/root;"));
        assert!(config.contains("default_type \"text/plain\";"));

        // 检查 http 块是否正确闭合
        assert!(config.ends_with("}\n"));
    }
}
