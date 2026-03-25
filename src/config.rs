
//! 配置管理模块
//!
//! 负责读取和解析proxy-config的配置文件

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

/// 应用类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AppType {
    /// 静态网站
    Static,
    /// API服务
    Api,
    /// 内部服务（如Redis等，不需要nginx反向代理）
    Internal,
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 微应用名称（从目录名称推导）
    pub name: String,

    /// 反向代理路径配置
    /// - 对于 Static 和 Api 类型：必须配置
    /// - 对于 Internal 类型：可以为空（不需要nginx代理）
    #[serde(default)]
    pub routes: Vec<String>,

    /// Docker容器名称
    pub container_name: String,

    /// 容器内部端口
    pub container_port: u16,

    /// 应用类型
    pub app_type: AppType,

    /// 应用描述（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// 额外的nginx配置（可选）
    /// - 仅对 Static 和 Api 类型有效
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nginx_extra_config: Option<String>,

    /// 服务文件夹路径
    /// - 自动从扫描目录推导
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Docker volumes 映射配置（可选）
    /// - 现在从 micro-app.volumes.yml 加载
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub docker_volumes: Vec<String>,

    /// 容器运行用户（可选）
    /// - 格式: "uid:gid" 或 "username"
    /// - 从 micro-app.volumes.yml 加载
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_as_user: Option<String>,
}

/// 动态生成的应用配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppsConfig {
    pub apps: Vec<AppConfig>,
}

impl AppsConfig {
    /// 从文件加载
    pub fn from_file<P: Into<PathBuf>>(path: P) -> Result<Self> {
        let path = path.into();
        log::info!("正在加载动态配置: {:?}", path);

        if !path.exists() {
            log::warn!("动态配置文件不存在: {:?}，将使用空配置", path);
            return Ok(AppsConfig { apps: vec![] });
        }

        let content = std::fs::read_to_string(&path).map_err(|e| {
            log::error!("读取动态配置文件失败: {:?}, 错误: {}", path, e);
            Error::Config(format!("无法读取动态配置文件 {:?}: {}", path, e))
        })?;

        let config: AppsConfig = serde_yaml::from_str(&content).map_err(|e| {
            log::error!("解析动态配置文件失败: {:?}, 错误: {}", path, e);
            Error::Config(format!("解析动态配置文件 {:?} 失败: {}", path, e))
        })?;

        log::info!("动态配置加载成功，发现 {} 个应用", config.apps.len());
        Ok(config)
    }

    /// 保存到文件
    pub fn save_to_file<P: Into<PathBuf>>(&self, path: P) -> Result<()> {
        let path = path.into();
        log::info!("正在保存动态配置到: {:?}", path);

        let content = serde_yaml::to_string(self).map_err(|e| {
            log::error!("序列化动态配置失败: {}", e);
            Error::Config(format!("序列化动态配置失败: {}", e))
        })?;

        // 添加文件头注释
        let header = "# 此文件由 micro_proxy 自动生成，请勿手动修改\n";
        let full_content = format!("{}\n{}", header, content);

        std::fs::write(&path, full_content).map_err(|e| {
            log::error!("保存动态配置文件失败: {:?}, 错误: {}", path, e);
            Error::Config(format!("保存动态配置文件 {:?} 失败: {}", path, e))
        })?;

        log::info!("动态配置保存成功");
        Ok(())
    }
}

/// 主配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// 扫描目录列表（支持多个目录）
    /// 用于自动发现包含 micro-app.yml 的微应用
    pub scan_dirs: Vec<String>,

    /// 动态生成的 apps 配置存储路径
    pub apps_config_path: String,

    /// Nginx配置文件输出路径
    pub nginx_config_path: String,

    /// Docker Compose配置文件输出路径
    pub compose_config_path: String,

    /// 状态文件路径
    pub state_file_path: String,

    /// 网络地址列表输出路径
    pub network_list_path: String,

    /// Docker网络名称
    pub network_name: String,

    /// Nginx监听的主机端口（统一入口）
    pub nginx_host_port: u16,

    /// Web根目录
    #[serde(default = "default_web_root")]
    pub web_root: String,

    /// 证书目录
    #[serde(default = "default_cert_dir")]
    pub cert_dir: String,

    /// 域名（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}

/// 默认 web_root 值
fn default_web_root() -> String {
    log::debug!("使用默认 web_root: /var/www/html");
    "/var/www/html".to_string()
}

/// 默认 cert_dir 值
fn default_cert_dir() -> String {
    log::debug!("使用默认 cert_dir: /etc/nginx/certs");
    "/etc/nginx/certs".to_string()
}

impl ProxyConfig {
    /// 从文件加载配置
    pub fn from_file<P: Into<PathBuf>>(path: P) -> Result<Self> {
        let path = path.into();
        log::info!("正在加载配置文件: {:?}", path);

        let content = std::fs::read_to_string(&path).map_err(|e| {
            log::error!("读取配置文件失败: {:?}, 错误: {}", path, e);
            Error::Config(format!("无法读取配置文件 {:?}: {}", path, e))
        })?;

        let config: ProxyConfig = serde_yaml::from_str(&content).map_err(|e| {
            log::error!("解析配置文件失败: {:?}, 错误: {}", path, e);
            Error::Config(format!("解析配置文件 {:?} 失败: {}", path, e))
        })?;

        log::info!("配置文件加载成功");
        log::debug!("配置内容: {:?}", config);
        log::debug!("web_root: {}", config.web_root);
        log::debug!("cert_dir: {}", config.cert_dir);
        if let Some(ref domain) = config.domain {
            log::debug!("domain: {}", domain);
        }

        Ok(config)
    }

    /// 加载 apps 配置（从 apps_config_path）
    pub fn load_apps(&self) -> Result<Vec<AppConfig>> {
        let apps_config = AppsConfig::from_file(&self.apps_config_path)?;
        Ok(apps_config.apps)
    }

    /// 保存 apps 配置到 apps_config_path
    pub fn save_apps(&self, apps: &[AppConfig]) -> Result<()> {
        let apps_config = AppsConfig {
            apps: apps.to_vec(),
        };
        apps_config.save_to_file(&self.apps_config_path)?;
        Ok(())
    }

    /// 验证配置的有效性
    pub fn validate(&self, apps: &[AppConfig], discovered_apps: &[String]) -> Result<()> {
        log::info!("开始验证配置...");

        // 验证扫描目录
        if self.scan_dirs.is_empty() {
            log::error!("配置错误: scan_dirs 不能为空");
            return Err(Error::Config("scan_dirs 不能为空".to_string()));
        }

        // 验证应用配置
        if apps.is_empty() {
            log::warn!("警告: 没有发现任何微应用");
        }

        // 检查所有应用名称是否唯一
        let mut app_names = HashSet::new();
        for app in apps {
            if app_names.contains(&app.name) {
                log::error!("配置错误: 发现重复的应用名称 '{}'", app.name);
                return Err(Error::Config(format!(
                    "发现重复的应用名称 '{}', 请确保所有应用（Static、Api、Internal）的名称唯一",
                    app.name
                )));
            }
            app_names.insert(app.name.clone());
        }

        // 验证每个应用配置
        for app in apps {
            match app.app_type {
                AppType::Static | AppType::Api => {
                    log::debug!("验证 Static/Api 应用: {}", app.name);

                    // 检查是否在扫描结果中找到
                    if !discovered_apps.contains(&app.name) {
                        log::error!("配置错误: 应用 '{}' 未在扫描目录中找到", app.name);
                        log::error!("扫描到的应用: {:?}", discovered_apps);
                        return Err(Error::Config(format!(
                            "应用 '{}' 未在扫描目录中找到",
                            app.name
                        )));
                    }

                    // 验证路由配置
                    if app.routes.is_empty() {
                        log::error!("配置错误: 应用 '{}' 的 routes 不能为空", app.name);
                        return Err(Error::Config(format!(
                            "应用 '{}' 的 routes 不能为空",
                            app.name
                        )));
                    }
                }
                AppType::Internal => {
                    log::debug!("验证 Internal 应用: {}", app.name);

                    // 验证 path 字段
                    let path = app.path.as_ref().ok_or_else(|| {
                        log::error!("配置错误: Internal 应用 '{}' 缺少 path 字段", app.name);
                        Error::Config(format!("Internal 应用 '{}' 缺少 path 字段", app.name))
                    })?;

                    // 验证 path 是否存在
                    let path_buf = PathBuf::from(path);
                    if !path_buf.exists() {
                        log::error!(
                            "配置错误: Internal 应用 '{}' 的路径不存在: {:?}",
                            app.name,
                            path
                        );
                        return Err(Error::Config(format!(
                            "Internal 应用 '{}' 的路径不存在: {:?}",
                            app.name, path
                        )));
                    }

                    // 验证是否包含 Dockerfile
                    let dockerfile_path = path_buf.join("Dockerfile");
                    if !dockerfile_path.exists() {
                        log::error!(
                            "配置错误: Internal 应用 '{}' 的路径中未找到 Dockerfile: {:?}",
                            app.name,
                            dockerfile_path
                        );
                        return Err(Error::Config(format!(
                            "Internal 应用 '{}' 的路径中未找到 Dockerfile: {:?}",
                            app.name, dockerfile_path
                        )));
                    }

                    // routes 应该为空
                    if !app.routes.is_empty() {
                        log::warn!("警告: Internal 应用 '{}' 配置了 routes，将被忽略", app.name);
                    }

                    // nginx_extra_config 不应该配置
                    if app.nginx_extra_config.is_some() {
                        log::warn!(
                            "警告: Internal 应用 '{}' 配置了 nginx_extra_config，将被忽略",
                            app.name
                        );
                    }
                }
            }

            // 验证 docker_volumes 配置
            if !app.docker_volumes.is_empty() {
                log::debug!(
                    "应用 '{}' 配置了 {} 个 volumes 映射",
                    app.name,
                    app.docker_volumes.len()
                );
                for volume in &app.docker_volumes {
                    log::debug!("  - {}", volume);
                }
            }

            // 验证 run_as_user 配置
            if let Some(ref user) = app.run_as_user {
                log::debug!("应用 '{}' 的 run_as_user: {}", app.name, user);
            }

            log::debug!("应用 '{}' 配置验证通过", app.name);
        }

        log::info!("配置验证通过");
        Ok(())
    }

    /// 获取指定名称的应用配置
    pub fn get_app_config<'a>(&self, apps: &'a [AppConfig], name: &str) -> Option<&'a AppConfig> {
        apps.iter().find(|app| app.name == name)
    }

    /// 获取所有需要 nginx 代理的应用（过滤掉 Internal 类型）
    pub fn get_nginx_apps<'a>(&self, apps: &'a [AppConfig]) -> Vec<&'a AppConfig> {
        apps.iter()
            .filter(|app| app.app_type != AppType::Internal)
            .collect()
    }

    /// 获取所有 Internal 类型的应用
    pub fn get_internal_apps<'a>(&self, apps: &'a [AppConfig]) -> Vec<&'a AppConfig> {
        apps.iter()
            .filter(|app| app.app_type == AppType::Internal)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_app_type_deserialize() {
        let yaml_static = r#"
app_type: static
"#;
        let config: serde_yaml::Value = serde_yaml::from_str(yaml_static).unwrap();
        assert_eq!(config["app_type"].as_str(), Some("static"));

        let yaml_api = r#"
app_type: api
"#;
        let config: serde_yaml::Value = serde_yaml::from_str(yaml_api).unwrap();
        assert_eq!(config["app_type"].as_str(), Some("api"));

        let yaml_internal = r#"
app_type: internal
"#;
        let config: serde_yaml::Value = serde_yaml::from_str(yaml_internal).unwrap();
        assert_eq!(config["app_type"].as_str(), Some("internal"));
    }

    #[test]
    fn test_default_web_root() {
        let config = ProxyConfig {
            scan_dirs: vec!["./apps".to_string()],
            apps_config_path: "./apps-config.yml".to_string(),
            nginx_config_path: "./nginx.conf".to_string(),
            compose_config_path: "./docker-compose.yml".to_string(),
            state_file_path: "./state".to_string(),
            network_list_path: "./network.txt".to_string(),
            network_name: "test-network".to_string(),
            nginx_host_port: 8080,
            web_root: default_web_root(),
            cert_dir: default_cert_dir(),
            domain: None,
        };
        assert_eq!(config.web_root, "/var/www/html");
    }

    #[test]
    fn test_default_cert_dir() {
        let config = ProxyConfig {
            scan_dirs: vec!["./apps".to_string()],
            apps_config_path: "./apps-config.yml".to_string(),
            nginx_config_path: "./nginx.conf".to_string(),
            compose_config_path: "./docker-compose.yml".to_string(),
            state_file_path: "./state".to_string(),
            network_list_path: "./network.txt".to_string(),
            network_name: "test-network".to_string(),
            nginx_host_port: 8080,
            web_root: default_web_root(),
            cert_dir: default_cert_dir(),
            domain: None,
        };
        assert_eq!(config.cert_dir, "/etc/nginx/certs");
    }

    #[test]
    fn test_validate_empty_scan_dirs() {
        let config = ProxyConfig {
            scan_dirs: vec![],
            apps_config_path: "./apps-config.yml".to_string(),
            nginx_config_path: "./nginx.conf".to_string(),
            compose_config_path: "./docker-compose.yml".to_string(),
            state_file_path: "./state".to_string(),
            network_list_path: "./network.txt".to_string(),
            network_name: "test-network".to_string(),
            nginx_host_port: 8080,
            web_root: default_web_root(),
            cert_dir: default_cert_dir(),
            domain: None,
        };

        let result = config.validate(&[], &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_duplicate_app_names() {
        let apps = vec![
            AppConfig {
                name: "test-app".to_string(),
                routes: vec!["/".to_string()],
                container_name: "test-container".to_string(),
                container_port: 80,
                app_type: AppType::Static,
                description: None,
                nginx_extra_config: None,
                path: None,
                docker_volumes: vec![],
                run_as_user: None,
            },
            AppConfig {
                name: "test-app".to_string(), // 重复名称
                routes: vec!["/api".to_string()],
                container_name: "test-container-2".to_string(),
                container_port: 8080,
                app_type: AppType::Api,
                description: None,
                nginx_extra_config: None,
                path: None,
                docker_volumes: vec![],
                run_as_user: None,
            },
        ];

        let config = ProxyConfig {
            scan_dirs: vec!["./apps".to_string()],
            apps_config_path: "./apps-config.yml".to_string(),
            nginx_config_path: "./nginx.conf".to_string(),
            compose_config_path: "./docker-compose.yml".to_string(),
            state_file_path: "./state".to_string(),
            network_list_path: "./network.txt".to_string(),
            network_name: "test-network".to_string(),
            nginx_host_port: 8080,
            web_root: default_web_root(),
            cert_dir: default_cert_dir(),
            domain: None,
        };

        let result = config.validate(&apps, &["test-app".to_string()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("重复的应用名称"));
    }

    #[test]
    fn test_validate_static_app_not_found() {
        let apps = vec![AppConfig {
            name: "test-app".to_string(),
            routes: vec!["/".to_string()],
            container_name: "test-container".to_string(),
            container_port: 80,
            app_type: AppType::Static,
            description: None,
            nginx_extra_config: None,
            path: None,
            docker_volumes: vec![],
            run_as_user: None,
        }];

        let config = ProxyConfig {
            scan_dirs: vec!["./apps".to_string()],
            apps_config_path: "./apps-config.yml".to_string(),
            nginx_config_path: "./nginx.conf".to_string(),
            compose_config_path: "./docker-compose.yml".to_string(),
            state_file_path: "./state".to_string(),
            network_list_path: "./network.txt".to_string(),
            network_name: "test-network".to_string(),
            nginx_host_port: 8080,
            web_root: default_web_root(),
            cert_dir: default_cert_dir(),
            domain: None,
        };

        let result = config.validate(&apps, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_internal_app_missing_path() {
        let apps = vec![AppConfig {
            name: "redis".to_string(),
            routes: vec![],
            container_name: "redis-container".to_string(),
            container_port: 6379,
            app_type: AppType::Internal,
            description: None,
            nginx_extra_config: None,
            path: None,
            docker_volumes: vec![],
            run_as_user: None,
        }];

        let config = ProxyConfig {
            scan_dirs: vec!["./apps".to_string()],
            apps_config_path: "./apps-config.yml".to_string(),
            nginx_config_path: "./nginx.conf".to_string(),
            compose_config_path: "./docker-compose.yml".to_string(),
            state_file_path: "./state".to_string(),
            network_list_path: "./network.txt".to_string(),
            network_name: "test-network".to_string(),
            nginx_host_port: 8080,
            web_root: default_web_root(),
            cert_dir: default_cert_dir(),
            domain: None,
        };

        let result = config.validate(&apps, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_success() {
        // 创建临时目录用于 Internal 应用
        let temp_dir = TempDir::new().unwrap();
        let redis_path = temp_dir.path().join("redis");
        std::fs::create_dir(&redis_path).unwrap();
        std::fs::write(redis_path.join("Dockerfile"), "FROM redis:alpine").unwrap();

        let apps = vec![
            AppConfig {
                name: "test-app".to_string(),
                routes: vec!["/".to_string()],
                container_name: "test-container".to_string(),
                container_port: 80,
                app_type: AppType::Static,
                description: None,
                nginx_extra_config: None,
                path: None,
                docker_volumes: vec![],
                run_as_user: None,
            },
            AppConfig {
                name: "redis".to_string(),
                routes: vec![],
                container_name: "redis-container".to_string(),
                container_port: 6379,
                app_type: AppType::Internal,
                description: None,
                nginx_extra_config: None,
                path: Some(redis_path.to_str().unwrap().to_string()),
                docker_volumes: vec![],
                run_as_user: None,
            },
        ];

        let config = ProxyConfig {
            scan_dirs: vec!["./apps".to_string()],
            apps_config_path: "./apps-config.yml".to_string(),
            nginx_config_path: "./nginx.conf".to_string(),
            compose_config_path: "./docker-compose.yml".to_string(),
            state_file_path: "./state".to_string(),
            network_list_path: "./network.txt".to_string(),
            network_name: "test-network".to_string(),
            nginx_host_port: 8080,
            web_root: default_web_root(),
            cert_dir: default_cert_dir(),
            domain: None,
        };

        let result = config.validate(&apps, &["test-app".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_app_config() {
        let apps = vec![
            AppConfig {
                name: "test-app".to_string(),
                routes: vec!["/".to_string()],
                container_name: "test-container".to_string(),
                container_port: 80,
                app_type: AppType::Static,
                description: None,
                nginx_extra_config: None,
                path: None,
                docker_volumes: vec![],
                run_as_user: None,
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
                run_as_user: None,
            },
        ];

        let config = ProxyConfig {
            scan_dirs: vec!["./apps".to_string()],
            apps_config_path: "./apps-config.yml".to_string(),
            nginx_config_path: "./nginx.conf".to_string(),
            compose_config_path: "./docker-compose.yml".to_string(),
            state_file_path: "./state".to_string(),
            network_list_path: "./network.txt".to_string(),
            network_name: "test-network".to_string(),
            nginx_host_port: 8080,
            web_root: default_web_root(),
            cert_dir: default_cert_dir(),
            domain: None,
        };

        let app = config.get_app_config(&apps, "test-app");
        assert!(app.is_some());
        assert_eq!(app.unwrap().name, "test-app");

        let app = config.get_app_config(&apps, "redis");
        assert!(app.is_some());
        assert_eq!(app.unwrap().name, "redis");

        let app = config.get_app_config(&apps, "non-existent");
        assert!(app.is_none());
    }

    #[test]
    fn test_get_nginx_apps() {
        let apps = vec![
            AppConfig {
                name: "test-app".to_string(),
                routes: vec!["/".to_string()],
                container_name: "test-container".to_string(),
                container_port: 80,
                app_type: AppType::Static,
                description: None,
                nginx_extra_config: None,
                path: None,
                docker_volumes: vec![],
                run_as_user: None,
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
                run_as_user: None,
            },
        ];

        let config = ProxyConfig {
            scan_dirs: vec!["./apps".to_string()],
            apps_config_path: "./apps-config.yml".to_string(),
            nginx_config_path: "./nginx.conf".to_string(),
            compose_config_path: "./docker-compose.yml".to_string(),
            state_file_path: "./state".to_string(),
            network_list_path: "./network.txt".to_string(),
            network_name: "test-network".to_string(),
            nginx_host_port: 8080,
            web_root: default_web_root(),
            cert_dir: default_cert_dir(),
            domain: None,
        };

        let nginx_apps = config.get_nginx_apps(&apps);
        assert_eq!(nginx_apps.len(), 1);
        assert_eq!(nginx_apps[0].name, "test-app");
    }

    #[test]
    fn test_get_internal_apps() {
        let apps = vec![
            AppConfig {
                name: "test-app".to_string(),
                routes: vec!["/".to_string()],
                container_name: "test-container".to_string(),
                container_port: 80,
                app_type: AppType::Static,
                description: None,
                nginx_extra_config: None,
                path: None,
                docker_volumes: vec![],
                run_as_user: None,
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
                run_as_user: None,
            },
        ];

        let config = ProxyConfig {
            scan_dirs: vec!["./apps".to_string()],
            apps_config_path: "./apps-config.yml".to_string(),
            nginx_config_path: "./nginx.conf".to_string(),
            compose_config_path: "./docker-compose.yml".to_string(),
            state_file_path: "./state".to_string(),
            network_list_path: "./network.txt".to_string(),
            network_name: "test-network".to_string(),
            nginx_host_port: 8080,
            web_root: default_web_root(),
            cert_dir: default_cert_dir(),
            domain: None,
        };

        let internal_apps = config.get_internal_apps(&apps);
        assert_eq!(internal_apps.len(), 1);
        assert_eq!(internal_apps[0].name, "redis");
    }

    #[test]
    fn test_docker_volumes_deserialize() {
        let yaml = r#"
name: test-app
routes: ["/"]
container_name: test-container
container_port: 80
app_type: static
docker_volumes:
  - "./data:/app/data"
  - "./config:/app/config:ro"
"#;
        let app: AppConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(app.docker_volumes.len(), 2);
        assert_eq!(app.docker_volumes[0], "./data:/app/data");
        assert_eq!(app.docker_volumes[1], "./config:/app/config:ro");
    }

    #[test]
    fn test_docker_volumes_default() {
        let yaml = r#"
name: test-app
routes: ["/"]
container_name: test-container
container_port: 80
app_type: static
"#;
        let app: AppConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(app.docker_volumes.len(), 0);
    }

    #[test]
    fn test_apps_config_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("apps-config.yml");

        let apps = vec![
            AppConfig {
                name: "app1".to_string(),
                routes: vec!["/".to_string()],
                container_name: "container1".to_string(),
                container_port: 80,
                app_type: AppType::Static,
                description: Some("Test app 1".to_string()),
                nginx_extra_config: None,
                path: Some("./apps/app1".to_string()),
                docker_volumes: vec![],
                run_as_user: None,
            },
            AppConfig {
                name: "app2".to_string(),
                routes: vec!["/api".to_string()],
                container_name: "container2".to_string(),
                container_port: 3000,
                app_type: AppType::Api,
                description: None,
                nginx_extra_config: None,
                path: Some("./apps/app2".to_string()),
                docker_volumes: vec!["./data:/app/data".to_string()],
                run_as_user: Some("999:999".to_string()),
            },
        ];

        let apps_config = AppsConfig { apps };
        apps_config.save_to_file(&config_path).unwrap();

        let loaded_config = AppsConfig::from_file(&config_path).unwrap();
        assert_eq!(loaded_config.apps.len(), 2);
        assert_eq!(loaded_config.apps[0].name, "app1");
        assert_eq!(loaded_config.apps[1].name, "app2");
        assert_eq!(loaded_config.apps[1].run_as_user, Some("999:999".to_string()));
    }
}
