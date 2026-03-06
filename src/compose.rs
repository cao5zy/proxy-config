
//! Docker Compose生成模块
//!
//! 负责生成docker-compose.yml文件

use crate::{config::AppConfig, config::AppType, Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Docker Compose配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeConfig {
    services: serde_yaml::Mapping,
    networks: serde_yaml::Mapping,
}

/// 生成docker-compose.yml配置
///
/// # 参数
/// - `apps`: 应用配置列表
/// - `network_name`: Docker网络名称
/// - `nginx_host_port`: nginx监听的主机端口
/// - `env_files`: 环境变量文件路径映射（应用名称 -> .env文件相对路径）
/// - `web_root`: Web根目录，用于存放ACME验证文件
/// - `cert_dir`: 证书目录，用于存放SSL证书
/// - `domain`: 域名（可选），用于判断是否启用HTTPS
///
/// # 返回
/// 返回生成的docker-compose.yml内容
pub fn generate_compose_config(
    apps: &[AppConfig],
    network_name: &str,
    nginx_host_port: u16,
    env_files: &HashMap<String, String>,
    web_root: &str,
    cert_dir: &str,
    domain: &Option<String>,
) -> Result<String> {
    log::info!("开始生成docker-compose.yml配置");
    log::debug!("网络名称: {}", network_name);
    log::debug!("nginx端口: {}", nginx_host_port);
    log::debug!("应用数量: {}", apps.len());
    log::debug!("环境变量文件数量: {}", env_files.len());
    log::debug!("web_root: {}", web_root);
    log::debug!("cert_dir: {}", cert_dir);
    log::debug!("domain: {:?}", domain);

    let mut compose = ComposeConfig {
        services: serde_yaml::Mapping::new(),
        networks: serde_yaml::Mapping::new(),
    };

    // 添加网络配置
    let mut network_config = serde_yaml::Mapping::new();
    // 指定网络名称，避免docker-compose自动添加项目名前缀
    network_config.insert(
        serde_yaml::Value::String("name".to_string()),
        serde_yaml::Value::String(network_name.to_string()),
    );
    // 标记为外部网络，使用已存在的网络而不是创建新网络
    network_config.insert(
        serde_yaml::Value::String("external".to_string()),
        serde_yaml::Value::Bool(true),
    );
    compose.networks.insert(
        serde_yaml::Value::String(network_name.to_string()),
        serde_yaml::Value::Mapping(network_config),
    );

    // 检查是否启用HTTPS
    let ssl_enabled = if let Some(ref domain_name) = domain {
        check_ssl_certificates(cert_dir, domain_name)
    } else {
        false
    };

    if ssl_enabled {
        log::info!("检测到SSL证书，启用HTTPS端口映射");
    } else {
        log::info!("未检测到SSL证书，仅启用HTTP端口映射");
    }

    // 添加nginx服务（仅依赖于非 Internal 类型的应用）
    let nginx_service = generate_nginx_service(
        nginx_host_port,
        network_name,
        apps,
        web_root,
        cert_dir,
        ssl_enabled,
    );
    compose.services.insert(
        serde_yaml::Value::String("nginx".to_string()),
        serde_yaml::Value::Mapping(nginx_service),
    );

    // 添加每个应用的服务
    for app in apps {
        // 获取该应用的 .env 文件路径
        let env_file = env_files.get(&app.name).cloned();
        let app_service = generate_app_service(app, network_name, env_file);
        compose.services.insert(
            serde_yaml::Value::String(app.container_name.clone()),
            serde_yaml::Value::Mapping(app_service),
        );
    }

    // 序列化为YAML
    let yaml = serde_yaml::to_string(&compose).map_err(|e| {
        log::error!("序列化docker-compose配置失败: {}", e);
        Error::Compose(format!("序列化docker-compose配置失败: {}", e))
    })?;

    log::info!("docker-compose.yml配置生成完成");
    log::debug!("生成的配置:\n{}", yaml);

    Ok(yaml)
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

/// 生成nginx服务配置（纯函数）
///
/// # 参数
/// - `nginx_host_port`: nginx监听的主机端口
/// - `network_name`: 网络名称
/// - `apps`: 应用配置列表（用于设置依赖关系，仅依赖非 Internal 类型）
/// - `web_root`: Web根目录，用于存放ACME验证文件
/// - `cert_dir`: 证书目录，用于存放SSL证书
/// - `ssl_enabled`: 是否启用HTTPS
///
/// # 返回
/// 返回nginx服务配置
fn generate_nginx_service(
    nginx_host_port: u16,
    network_name: &str,
    apps: &[AppConfig],
    web_root: &str,
    cert_dir: &str,
    ssl_enabled: bool,
) -> serde_yaml::Mapping {
    log::debug!("生成nginx服务配置");
    log::debug!("web_root: {}", web_root);
    log::debug!("cert_dir: {}", cert_dir);
    log::debug!("ssl_enabled: {}", ssl_enabled);

    let mut service = serde_yaml::Mapping::new();

    // 镜像
    service.insert(
        serde_yaml::Value::String("image".to_string()),
        serde_yaml::Value::String("nginx:alpine".to_string()),
    );

    // 容器名称
    service.insert(
        serde_yaml::Value::String("container_name".to_string()),
        serde_yaml::Value::String("proxy-nginx".to_string()),
    );

    // 端口映射
    let mut ports = vec![format!("{}:80", nginx_host_port)];
    if ssl_enabled {
        ports.push("443:443".to_string());
        log::debug!("添加HTTPS端口映射: 443:443");
    }
    service.insert(
        serde_yaml::Value::String("ports".to_string()),
        serde_yaml::Value::Sequence(ports.into_iter().map(serde_yaml::Value::String).collect()),
    );

    // 卷挂载
    // 1. nginx.conf 配置文件
    // 2. web_root 目录（用于 ACME 验证）
    // 3. cert_dir 目录（用于 SSL 证书）
    let volumes = vec![
        "./nginx.conf:/etc/nginx/nginx.conf:ro".to_string(),
        format!("{}:{}", web_root, web_root),
        format!("{}:{}", cert_dir, cert_dir),
    ];
    log::debug!("nginx卷挂载配置: {:?}", volumes);
    service.insert(
        serde_yaml::Value::String("volumes".to_string()),
        serde_yaml::Value::Sequence(volumes.into_iter().map(serde_yaml::Value::String).collect()),
    );

    // 网络配置
    let mut networks = serde_yaml::Mapping::new();
    networks.insert(
        serde_yaml::Value::String(network_name.to_string()),
        serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
    );
    service.insert(
        serde_yaml::Value::String("networks".to_string()),
        serde_yaml::Value::Mapping(networks),
    );

    // 依赖关系：nginx仅依赖于非 Internal 类型的应用服务
    let non_internal_apps: Vec<String> = apps
        .iter()
        .filter(|app| app.app_type != AppType::Internal)
        .map(|app| app.container_name.clone())
        .collect();

    if !non_internal_apps.is_empty() {
        let count = non_internal_apps.len();
        service.insert(
            serde_yaml::Value::String("depends_on".to_string()),
            serde_yaml::Value::Sequence(
                non_internal_apps
                    .into_iter()
                    .map(serde_yaml::Value::String)
                    .collect(),
            ),
        );
        log::debug!("nginx 依赖于 {} 个非 Internal 应用", count);
    } else {
        log::debug!("nginx 不依赖于任何应用（所有应用都是 Internal 类型）");
    }

    // 重启策略
    service.insert(
        serde_yaml::Value::String("restart".to_string()),
        serde_yaml::Value::String("unless-stopped".to_string()),
    );

    service
}

/// 生成应用服务配置（纯函数）
///
/// # 参数
/// - `app`: 应用配置
/// - `network_name`: 网络名称
/// - `env_file`: 环境变量文件路径（可选）
///
/// # 返回
/// 返回应用服务配置
fn generate_app_service(
    app: &AppConfig,
    network_name: &str,
    env_file: Option<String>,
) -> serde_yaml::Mapping {
    let mut service = serde_yaml::Mapping::new();

    // 镜像名称（使用应用名称）
    let image_name = format!("{}:latest", app.name);
    service.insert(
        serde_yaml::Value::String("image".to_string()),
        serde_yaml::Value::String(image_name),
    );

    // 容器名称
    service.insert(
        serde_yaml::Value::String("container_name".to_string()),
        serde_yaml::Value::String(app.container_name.clone()),
    );

    // 网络配置
    let mut networks = serde_yaml::Mapping::new();
    networks.insert(
        serde_yaml::Value::String(network_name.to_string()),
        serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
    );
    service.insert(
        serde_yaml::Value::String("networks".to_string()),
        serde_yaml::Value::Mapping(networks),
    );

    // 重启策略
    service.insert(
        serde_yaml::Value::String("restart".to_string()),
        serde_yaml::Value::String("unless-stopped".to_string()),
    );

    // 如果有环境变量文件，添加 env_file 配置
    if let Some(env_file_path) = env_file {
        log::debug!("为应用 '{}' 添加环境变量文件: {}", app.name, env_file_path);
        service.insert(
            serde_yaml::Value::String("env_file".to_string()),
            serde_yaml::Value::String(env_file_path),
        );
    } else {
        log::debug!("应用 '{}' 没有环境变量文件", app.name);
    }

    // 如果有 volumes 配置，添加 volumes 字段
    if !app.docker_volumes.is_empty() {
        log::debug!(
            "为应用 '{}' 添加 {} 个 volumes 映射",
            app.name,
            app.docker_volumes.len()
        );
        for volume in &app.docker_volumes {
            log::debug!("  - {}", volume);
        }
        service.insert(
            serde_yaml::Value::String("volumes".to_string()),
            serde_yaml::Value::Sequence(
                app.docker_volumes
                    .iter()
                    .map(|v| serde_yaml::Value::String(v.clone()))
                    .collect(),
            ),
        );
    } else {
        log::debug!("应用 '{}' 没有配置 volumes 映射", app.name);
    }

    // 根据应用类型添加额外配置
    match app.app_type {
        AppType::Static => {
            // 静态网站需要健康检查
            let healthcheck = format!(
                r#"CMD-SHELL, wget --quiet --tries=1 --spider http://localhost:{} || exit 1"#,
                app.container_port
            );
            let mut healthcheck_map = serde_yaml::Mapping::new();
            healthcheck_map.insert(
                serde_yaml::Value::String("test".to_string()),
                serde_yaml::Value::String(healthcheck),
            );
            healthcheck_map.insert(
                serde_yaml::Value::String("interval".to_string()),
                serde_yaml::Value::String("30s".to_string()),
            );
            healthcheck_map.insert(
                serde_yaml::Value::String("timeout".to_string()),
                serde_yaml::Value::String("10s".to_string()),
            );
            healthcheck_map.insert(
                serde_yaml::Value::String("retries".to_string()),
                serde_yaml::Value::Number(3.into()),
            );
            service.insert(
                serde_yaml::Value::String("healthcheck".to_string()),
                serde_yaml::Value::Mapping(healthcheck_map),
            );
            log::debug!("为 Static 应用 '{}' 添加健康检查", app.name);
        }
        AppType::Api => {
            // API服务需要健康检查
            let healthcheck = format!(
                r#"CMD-SHELL, wget --quiet --tries=1 --spider http://localhost:{} || exit 1"#,
                app.container_port
            );
            let mut healthcheck_map = serde_yaml::Mapping::new();
            healthcheck_map.insert(
                serde_yaml::Value::String("test".to_string()),
                serde_yaml::Value::String(healthcheck),
            );
            healthcheck_map.insert(
                serde_yaml::Value::String("interval".to_string()),
                serde_yaml::Value::String("30s".to_string()),
            );
            healthcheck_map.insert(
                serde_yaml::Value::String("timeout".to_string()),
                serde_yaml::Value::String("10s".to_string()),
            );
            healthcheck_map.insert(
                serde_yaml::Value::String("retries".to_string()),
                serde_yaml::Value::Number(3.into()),
            );
            service.insert(
                serde_yaml::Value::String("healthcheck".to_string()),
                serde_yaml::Value::Mapping(healthcheck_map),
            );
            log::debug!("为 Api 应用 '{}' 添加健康检查", app.name);
        }
        AppType::Internal => {
            // Internal 类型不添加健康检查（可能不是 HTTP 服务）
            log::debug!("Internal 应用 '{}' 不添加健康检查", app.name);
        }
    }

    service
}

/// 保存docker-compose配置到文件
///
/// # 参数
/// - `config`: docker-compose配置内容
/// - `output_path`: 输出文件路径
///
/// # 返回
/// 返回保存结果
pub fn save_compose_config(config: &str, output_path: &str) -> Result<()> {
    log::info!("保存docker-compose配置到文件: {}", output_path);

    std::fs::write(output_path, config).map_err(|e| {
        log::error!(
            "写入docker-compose配置文件失败: {}, 错误: {}",
            output_path,
            e
        );
        Error::Compose(format!("写入docker-compose配置文件失败: {}", e))
    })?;

    log::info!("docker-compose配置文件保存成功: {}", output_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_compose_config() {
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
                docker_volumes: vec![],
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
                docker_volumes: vec![],
            },
        ];

        let mut env_files = HashMap::new();
        env_files.insert(
            "main-app".to_string(),
            "./micro-apps/main-app/.env".to_string(),
        );
        env_files.insert(
            "api-service".to_string(),
            "./micro-apps/api-service/.env".to_string(),
        );

        let config = generate_compose_config(
            &apps,
            "test-network",
            8080,
            &env_files,
            "/var/www/html",
            "/etc/nginx/certs",
            &None,
        )
        .unwrap();

        assert!(!config.contains("version:"));
        assert!(config.contains("services:"));
        assert!(config.contains("networks:"));
        assert!(config.contains("nginx:"));
        assert!(config.contains("main-container:"));
        assert!(config.contains("api-container:"));
        assert!(config.contains("test-network:"));
        assert!(config.contains("8080:80"));
        // 没有SSL证书，不应该有443端口
        assert!(!config.contains("443:443"));
        // 检查依赖关系
        assert!(config.contains("depends_on:"));
        assert!(config.contains("- main-container"));
        assert!(config.contains("- api-container"));
        // 检查网络名称配置
        assert!(config.contains("name: test-network"));
        // 检查外部网络配置
        assert!(config.contains("external: true"));
        // 检查环境变量文件
        assert!(config.contains("env_file:"));
        assert!(config.contains("./micro-apps/main-app/.env"));
        assert!(config.contains("./micro-apps/api-service/.env"));
        // 检查 web_root 和 cert_dir 卷挂载
        assert!(config.contains("/var/www/html:/var/www/html"));
        assert!(config.contains("/etc/nginx/certs:/etc/nginx/certs"));
    }

    #[test]
    fn test_generate_compose_config_with_ssl() {
        let apps = vec![AppConfig {
            name: "main-app".to_string(),
            routes: vec!["/".to_string()],
            container_name: "main-container".to_string(),
            container_port: 80,
            app_type: AppType::Static,
            description: None,
            nginx_extra_config: None,
            path: None,
            docker_volumes: vec![],
        }];

        let mut env_files = HashMap::new();

        // 创建临时证书文件
        let temp_dir = tempfile::tempdir().unwrap();
        let cert_dir = temp_dir.path().to_str().unwrap();
        let domain = "example.com";

        // 创建 .cer 文件
        std::fs::write(
            temp_dir.path().join(format!("{}.cer", domain)),
            "fake cert",
        )
        .unwrap();
        // 创建 .key 文件
        std::fs::write(
            temp_dir.path().join(format!("{}.key", domain)),
            "fake key",
        )
        .unwrap();

        let config = generate_compose_config(
            &apps,
            "test-network",
            8080,
            &env_files,
            "/var/www/html",
            cert_dir,
            &Some(domain.to_string()),
        )
        .unwrap();

        // 应该包含443端口映射
        assert!(config.contains("8080:80"));
        assert!(config.contains("443:443"));
    }

    #[test]
    fn test_generate_compose_config_with_internal() {
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
                docker_volumes: vec![],
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
                docker_volumes: vec![],
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
                docker_volumes: vec![],
            },
        ];

        let mut env_files = HashMap::new();
        env_files.insert(
            "main-app".to_string(),
            "./micro-apps/main-app/.env".to_string(),
        );
        env_files.insert("redis".to_string(), "./services/redis/.env".to_string());
        env_files.insert(
            "api-service".to_string(),
            "./micro-apps/api-service/.env".to_string(),
        );

        let config = generate_compose_config(
            &apps,
            "test-network",
            8080,
            &env_files,
            "/var/www/html",
            "/etc/nginx/certs",
            &None,
        )
        .unwrap();

        // 检查所有服务都存在
        assert!(config.contains("nginx:"));
        assert!(config.contains("main-container:"));
        assert!(config.contains("redis-container:"));
        assert!(config.contains("api-container:"));

        // 检查依赖关系：nginx 不应该依赖于 redis
        assert!(config.contains("depends_on:"));
        assert!(config.contains("- main-container"));
        assert!(config.contains("- api-container"));
        assert!(!config.contains("- redis-container"));

        // 检查健康检查：redis 不应该有健康检查
        assert!(config.contains("healthcheck:"));
        // main-container 和 api-container 应该有健康检查
        assert!(config.contains("wget --quiet --tries=1 --spider http://localhost:80"));
        assert!(config.contains("wget --quiet --tries=1 --spider http://localhost:3000"));
        // redis 不应该有 wget 健康检查
        assert!(!config.contains("wget --quiet --tries=1 --spider http://localhost:6379"));

        // 检查环境变量文件
        assert!(config.contains("env_file:"));
        assert!(config.contains("./micro-apps/main-app/.env"));
        assert!(config.contains("./services/redis/.env"));
        assert!(config.contains("./micro-apps/api-service/.env"));

        // 检查 web_root 和 cert_dir 卷挂载
        assert!(config.contains("/var/www/html:/var/www/html"));
        assert!(config.contains("/etc/nginx/certs:/etc/nginx/certs"));
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
            docker_volumes: vec![],
        }];

        let mut env_files = HashMap::new();
        env_files.insert("redis".to_string(), "./services/redis/.env".to_string());

        let config = generate_compose_config(
            &apps,
            "test-network",
            8080,
            &env_files,
            "/var/www/html",
            "/etc/nginx/certs",
            &None,
        )
        .unwrap();

        // 检查服务存在
        assert!(config.contains("nginx:"));
        assert!(config.contains("redis-container:"));

        // nginx 不应该有 depends_on
        assert!(!config.contains("depends_on:"));

        // redis 不应该有健康检查
        assert!(!config.contains("healthcheck:"));

        // 检查环境变量文件
        assert!(config.contains("env_file:"));
        assert!(config.contains("./services/redis/.env"));

        // 检查 web_root 和 cert_dir 卷挂载
        assert!(config.contains("/var/www/html:/var/www/html"));
        assert!(config.contains("/etc/nginx/certs:/etc/nginx/certs"));
    }

    #[test]
    fn test_save_compose_config() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let config = "services: {}";

        let result = save_compose_config(config, temp_file.path().to_str().unwrap());
        assert!(result.is_ok());

        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert_eq!(content, config);
    }

    #[test]
    fn test_generate_compose_config_with_docker_volumes() {
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
                docker_volumes: vec!["./data:/app/data".to_string(), "./config:/app/config:ro".to_string()],
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
                docker_volumes: vec!["./redis-data:/data".to_string()],
            },
        ];

        let mut env_files = HashMap::new();
        env_files.insert(
            "main-app".to_string(),
            "./micro-apps/main-app/.env".to_string(),
        );
        env_files.insert("redis".to_string(), "./services/redis/.env".to_string());

        let config = generate_compose_config(
            &apps,
            "test-network",
            8080,
            &env_files,
            "/var/www/html",
            "/etc/nginx/certs",
            &None,
        )
        .unwrap();

        // 检查 main-container 的 volumes
        assert!(config.contains("main-container:"));
        assert!(config.contains("./data:/app/data"));
        assert!(config.contains("./config:/app/config:ro"));

        // 检查 redis-container 的 volumes
        assert!(config.contains("redis-container:"));
        assert!(config.contains("./redis-data:/data"));

        // 检查 nginx 的 volumes（不应该包含应用的 volumes）
        // nginx 只应该有 nginx.conf, web_root, cert_dir 三个挂载
        assert!(config.contains("nginx:"));
        // 验证 nginx 的 volumes 部分不包含应用的 volumes
        // 通过检查 nginx 服务配置块来验证
        let nginx_start = config.find("nginx:").unwrap();
        let main_container_start = config.find("main-container:").unwrap();
        let nginx_section = &config[nginx_start..main_container_start];
        // nginx 的 volumes 应该只包含 nginx.conf, web_root, cert_dir
        assert!(nginx_section.contains("./nginx.conf:/etc/nginx/nginx.conf:ro"));
        assert!(nginx_section.contains("/var/www/html:/var/www/html"));
        assert!(nginx_section.contains("/etc/nginx/certs:/etc/nginx/certs"));
        // nginx 的 volumes 不应该包含应用的 volumes
        assert!(!nginx_section.contains("./data:/app/data"));
        assert!(!nginx_section.contains("./config:/app/config:ro"));
    }

    #[test]
    fn test_generate_compose_config_without_docker_volumes() {
        let apps = vec![AppConfig {
            name: "main-app".to_string(),
            routes: vec!["/".to_string()],
            container_name: "main-container".to_string(),
            container_port: 80,
            app_type: AppType::Static,
            description: None,
            nginx_extra_config: None,
            path: None,
            docker_volumes: vec![],
        }];

        let mut env_files = HashMap::new();
        env_files.insert(
            "main-app".to_string(),
            "./micro-apps/main-app/.env".to_string(),
        );

        let config = generate_compose_config(
            &apps,
            "test-network",
            8080,
            &env_files,
            "/var/www/html",
            "/etc/nginx/certs",
            &None,
        )
        .unwrap();

        // 检查 main-container 不应该有 volumes 字段
        assert!(config.contains("main-container:"));
        // 在 main-container 的配置块中不应该有 volumes
        let main_container_start = config.find("main-container:").unwrap();
        let next_service_start = config.find("redis-container:").unwrap_or(config.len());
        let main_container_config = &config[main_container_start..next_service_start];
        assert!(!main_container_config.contains("volumes:"));
    }
}
