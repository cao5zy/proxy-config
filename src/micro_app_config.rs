
//! 微应用配置模块
//!
//! 负责解析每个微应用目录下的 micro-app.yml 文件

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 未显式配置时使用的 HTTP 健康检查路径。
pub fn default_healthcheck_path() -> String {
    "/".to_string()
}

/// 未显式配置时，微应用以源码包方式构建。
pub fn default_package_type() -> String {
    "source".to_string()
}

/// 微应用配置文件结构（micro-app.yml）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroAppConfig {
    /// 应用包类型：`source` 通过 Dockerfile 构建，`image` 导入已有镜像归档，
    /// `registry` 直接引用已有仓库镜像。
    #[serde(default = "default_package_type")]
    pub package_type: String,

    /// 镜像归档路径。仅当 `package_type: image` 时必需；相对路径基于应用目录。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_archive: Option<String>,

    /// 仓库镜像引用。仅当 `package_type: registry` 时必需，例如 `postgres:15-alpine`。
    #[serde(rename = "image", skip_serializing_if = "Option::is_none")]
    pub registry_image: Option<String>,

    /// 源码包的目标镜像平台，例如 `linux/amd64`。配置后使用 Docker Buildx 构建。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_platform: Option<String>,

    /// 访问路径（static/api类型必需）
    #[serde(default)]
    pub routes: Vec<String>,

    /// 容器名称（必需，全局唯一）
    pub container_name: String,

    /// 容器内部端口（必需）
    pub container_port: u16,

    /// HTTP 健康检查路径（可选，默认 `/`）
    #[serde(default = "default_healthcheck_path")]
    pub healthcheck_path: String,

    /// 应用类型（必需）
    pub app_type: String, // 使用String，后续转换为AppType

    /// 应用描述（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// 额外的 nginx 配置（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nginx_extra_config: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_connect_timeout: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_read_timeout: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_send_timeout: Option<u64>,
}

/// 验证单个路由格式是否合法
///
/// 规则：
/// - 路由不能为空
/// - 必须以 `/` 开头
/// - 不能以 `/` 结尾，除非路由恰好是 `"/"`（根路由）
pub fn validate_route_format(route: &str) -> Result<()> {
    if route.is_empty() {
        return Err(Error::Config("路由不能为空".to_string()));
    }
    if !route.starts_with('/') {
        return Err(Error::Config(format!(
            "路由 '{}' 必须以 '/' 开头",
            route
        )));
    }
    if route != "/" && route.ends_with('/') {
        return Err(Error::Config(format!(
            "路由 '{}' 不能以 '/' 结尾（根路由 '/' 除外）",
            route
        )));
    }
    Ok(())
}

/// 验证健康检查路径格式是否合法。
pub fn validate_healthcheck_path(path: &str) -> Result<()> {
    if path.is_empty() {
        return Err(Error::Config("healthcheck_path 不能为空".to_string()));
    }
    if !path.starts_with('/') {
        return Err(Error::Config(format!(
            "healthcheck_path '{}' 必须以 '/' 开头",
            path
        )));
    }
    Ok(())
}

/// 验证 Buildx 单平台目标格式。
pub fn validate_build_platform(platform: &str) -> Result<()> {
    let parts = platform.split('/').collect::<Vec<_>>();
    if !(2..=3).contains(&parts.len()) || parts[0] != "linux" {
        return Err(Error::Config(format!(
            "build_platform '{}' 必须采用 linux/<arch> 或 linux/<arch>/<variant> 格式",
            platform
        )));
    }
    if parts.iter().any(|part| {
        part.is_empty()
            || !part
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'))
    }) {
        return Err(Error::Config(format!(
            "build_platform '{}' 包含无效的平台组件",
            platform
        )));
    }
    Ok(())
}

impl MicroAppConfig {
    /// 从文件加载微应用配置
    pub fn from_file<P: Into<PathBuf>>(path: P) -> Result<Self> {
        let path = path.into();
        log::debug!("正在加载微应用配置: {:?}", path);

        let content = std::fs::read_to_string(&path).map_err(|e| {
            log::error!("读取微应用配置文件失败: {:?}, 错误: {}", path, e);
            Error::Config(format!("无法读取微应用配置文件 {:?}: {}", path, e))
        })?;

        let config: MicroAppConfig = serde_yaml::from_str(&content).map_err(|e| {
            log::error!("解析微应用配置文件失败: {:?}, 错误: {}", path, e);
            Error::Config(format!("解析微应用配置文件 {:?} 失败: {}", path, e))
        })?;

        log::debug!("微应用配置加载成功: {:?}", config);
        Ok(config)
    }

    /// 验证微应用配置
    pub fn validate(&self, app_name: &str) -> Result<()> {
        log::debug!("验证微应用 '{}' 的配置", app_name);

        // 验证 container_name 不为空
        if self.container_name.is_empty() {
            log::error!("微应用 '{}' 的 container_name 不能为空", app_name);
            return Err(Error::Config(format!(
                "微应用 '{}' 的 container_name 不能为空",
                app_name
            )));
        }

        // 验证 container_port 不为 0
        if self.container_port == 0 {
            log::error!("微应用 '{}' 的 container_port 不能为 0", app_name);
            return Err(Error::Config(format!(
                "微应用 '{}' 的 container_port 不能为 0",
                app_name
            )));
        }

        validate_healthcheck_path(&self.healthcheck_path)?;

        let valid_package_types = ["source", "image", "registry"];
        if !valid_package_types.contains(&self.package_type.as_str()) {
            return Err(Error::Config(format!(
                "微应用 '{}' 的 package_type '{}' 无效，必须是 source、image 或 registry",
                app_name, self.package_type
            )));
        }
        if self.package_type == "image"
            && self
                .image_archive
                .as_deref()
                .is_none_or(str::is_empty)
        {
            return Err(Error::Config(format!(
                "微应用 '{}' 使用 image 包时必须配置 image_archive",
                app_name
            )));
        }
        if self.package_type == "image" && self.build_platform.is_some() {
            return Err(Error::Config(format!(
                "微应用 '{}' 使用 image 包时不能配置 build_platform",
                app_name
            )));
        }
        if self.package_type == "registry"
            && self
                .registry_image
                .as_deref()
                .is_none_or(str::is_empty)
        {
            return Err(Error::Config(format!(
                "微应用 '{}' 使用 registry 包时必须配置 image",
                app_name
            )));
        }
        if self.package_type != "registry" && self.registry_image.is_some() {
            return Err(Error::Config(format!(
                "微应用 '{}' 仅 registry 包可以配置 image",
                app_name
            )));
        }
        if self.package_type == "registry" && self.build_platform.is_some() {
            return Err(Error::Config(format!(
                "微应用 '{}' 使用 registry 包时不能配置 build_platform",
                app_name
            )));
        }
        if let Some(platform) = &self.build_platform {
            validate_build_platform(platform)?;
        }

        // 验证 app_type
        let valid_types = ["static", "api", "internal"];
        if !valid_types.contains(&self.app_type.as_str()) {
            log::error!("微应用 '{}' 的 app_type '{}' 无效", app_name, self.app_type);
            return Err(Error::Config(format!(
                "微应用 '{}' 的 app_type '{}' 无效，必须是 static、api 或 internal",
                app_name, self.app_type
            )));
        }

        // static/api 类型必须配置 routes
        if (self.app_type == "static" || self.app_type == "api") && self.routes.is_empty() {
            log::error!("微应用 '{}' 的 routes 不能为空", app_name);
            return Err(Error::Config(format!(
                "微应用 '{}' 是 {} 类型，routes 不能为空",
                app_name, self.app_type
            )));
        }

        // 验证每个路由格式
        for route in &self.routes {
            validate_route_format(route)?;
        }

        // internal 类型：routes 可选
        // - 空 routes：纯内部服务（如 Redis），不暴露到 nginx
        // - 有 routes：对外暴露 HTTP 服务的内部应用（如 MinIO），通过 nginx 代理
        if self.app_type == "internal" && !self.routes.is_empty() {
            log::debug!(
                "微应用 '{}' 是 internal 类型，配置了 {} 个路由",
                app_name,
                self.routes.len()
            );
        }

        log::debug!("微应用 '{}' 配置验证通过", app_name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_micro_app_config_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("micro-app.yml");

        let yaml_content = r#"
routes: ["/", "/api"]
container_name: "test-container"
container_port: 8080
app_type: "api"
description: "Test API service"
nginx_extra_config: |
  add_header 'X-Custom' 'value';
"#;

        std::fs::write(&config_path, yaml_content).unwrap();

        let config = MicroAppConfig::from_file(&config_path).unwrap();
        assert_eq!(config.routes, vec!["/", "/api"]);
        assert_eq!(config.container_name, "test-container");
        assert_eq!(config.container_port, 8080);
        assert_eq!(config.app_type, "api");
        assert_eq!(config.healthcheck_path, "/");
        assert_eq!(config.description, Some("Test API service".to_string()));
        assert!(config.nginx_extra_config.is_some());
    }

    #[test]
    fn test_micro_app_config_from_file_reads_configured_healthcheck_path() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("micro-app.yml");
        let yaml_content = r#"
routes: ["/api"]
container_name: "test-api"
container_port: 3000
healthcheck_path: "/healthz"
app_type: "api"
"#;

        std::fs::write(&config_path, yaml_content).unwrap();

        let config = MicroAppConfig::from_file(&config_path).unwrap();
        assert_eq!(config.healthcheck_path, "/healthz");
    }

    #[test]
    fn test_micro_app_config_defaults_to_source_package() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("micro-app.yml");
        std::fs::write(
            &config_path,
            "routes: [\"/\"]\ncontainer_name: test\ncontainer_port: 80\napp_type: static\n",
        )
        .unwrap();

        let config = MicroAppConfig::from_file(&config_path).unwrap();
        assert_eq!(config.package_type, "source");
        assert_eq!(config.image_archive, None);
        assert_eq!(config.build_platform, None);
    }

    #[test]
    fn test_micro_app_config_reads_valid_source_build_platform() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("micro-app.yml");
        std::fs::write(
            &config_path,
            "routes: [\"/\"]\ncontainer_name: test\ncontainer_port: 80\napp_type: static\nbuild_platform: linux/amd64\n",
        )
        .unwrap();

        let config = MicroAppConfig::from_file(&config_path).unwrap();
        assert_eq!(config.build_platform.as_deref(), Some("linux/amd64"));
        assert!(config.validate("test-app").is_ok());
    }

    #[test]
    fn test_micro_app_config_reads_registry_package_image() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("micro-app.yml");
        std::fs::write(
            &config_path,
            "routes: []\ncontainer_name: postgres\ncontainer_port: 5432\napp_type: internal\npackage_type: registry\nimage: postgres:15-alpine\n",
        )
        .unwrap();

        let config = MicroAppConfig::from_file(&config_path).unwrap();
        assert_eq!(config.registry_image.as_deref(), Some("postgres:15-alpine"));
        assert!(config.validate("postgres").is_ok());
    }

    #[test]
    fn test_micro_app_config_rejects_registry_package_without_image() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("micro-app.yml");
        std::fs::write(
            &config_path,
            "routes: []\ncontainer_name: postgres\ncontainer_port: 5432\napp_type: internal\npackage_type: registry\n",
        )
        .unwrap();

        let config = MicroAppConfig::from_file(&config_path).unwrap();
        assert!(config.validate("postgres").is_err());
    }

    #[test]
    fn test_validate_build_platform_rejects_non_linux_or_incomplete_values() {
        assert!(validate_build_platform("amd64").is_err());
        assert!(validate_build_platform("darwin/arm64").is_err());
        assert!(validate_build_platform("linux/").is_err());
    }

    #[test]
    fn test_micro_app_config_rejects_image_package_without_archive() {
        let config = MicroAppConfig {
            package_type: "image".to_string(),
            image_archive: None,
            registry_image: None,
            build_platform: None,
            routes: vec!["/".to_string()],
            container_name: "test-container".to_string(),
            container_port: 80,
            healthcheck_path: "/".to_string(),
            app_type: "static".to_string(),
            description: None,
            nginx_extra_config: None,
            proxy_connect_timeout: None,
            proxy_read_timeout: None,
            proxy_send_timeout: None,
        };

        assert!(config.validate("test-app").is_err());
    }

    #[test]
    fn test_micro_app_config_validate_success() {
        let config = MicroAppConfig {
            package_type: "source".to_string(),
            image_archive: None,
            registry_image: None,
            build_platform: None,
            routes: vec!["/".to_string()],
            container_name: "test-container".to_string(),
            container_port: 80,
            healthcheck_path: "/".to_string(),
            app_type: "static".to_string(),
            description: None,
            nginx_extra_config: None,

            proxy_connect_timeout: None,

            proxy_read_timeout: None,

            proxy_send_timeout: None,
        };

        let result = config.validate("test-app");
        assert!(result.is_ok());
    }

    #[test]
    fn test_micro_app_config_validate_rejects_healthcheck_path_without_leading_slash() {
        let config = MicroAppConfig {
            package_type: "source".to_string(),
            image_archive: None,
            registry_image: None,
            build_platform: None,
            routes: vec!["/".to_string()],
            container_name: "test-container".to_string(),
            container_port: 80,
            app_type: "static".to_string(),
            healthcheck_path: "healthz".to_string(),
            description: None,
            nginx_extra_config: None,
            proxy_connect_timeout: None,
            proxy_read_timeout: None,
            proxy_send_timeout: None,
        };

        let result = config.validate("test-app");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("healthcheck_path"));
    }

    #[test]
    fn test_micro_app_config_validate_empty_container_name() {
        let config = MicroAppConfig {
            package_type: "source".to_string(),
            image_archive: None,
            registry_image: None,
            build_platform: None,
            routes: vec!["/".to_string()],
            container_name: "".to_string(),
            container_port: 80,
            healthcheck_path: "/".to_string(),
            app_type: "static".to_string(),
            description: None,
            nginx_extra_config: None,

            proxy_connect_timeout: None,

            proxy_read_timeout: None,

            proxy_send_timeout: None,
        };

        let result = config.validate("test-app");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("container_name 不能为空"));
    }

    #[test]
    fn test_micro_app_config_validate_zero_port() {
        let config = MicroAppConfig {
            package_type: "source".to_string(),
            image_archive: None,
            registry_image: None,
            build_platform: None,
            routes: vec!["/".to_string()],
            container_name: "test-container".to_string(),
            container_port: 0,
            healthcheck_path: "/".to_string(),
            app_type: "static".to_string(),
            description: None,
            nginx_extra_config: None,

            proxy_connect_timeout: None,

            proxy_read_timeout: None,

            proxy_send_timeout: None,
        };

        let result = config.validate("test-app");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("container_port 不能为 0"));
    }

    #[test]
    fn test_micro_app_config_validate_invalid_app_type() {
        let config = MicroAppConfig {
            package_type: "source".to_string(),
            image_archive: None,
            registry_image: None,
            build_platform: None,
            routes: vec!["/".to_string()],
            container_name: "test-container".to_string(),
            container_port: 80,
            healthcheck_path: "/".to_string(),
            app_type: "invalid".to_string(),
            description: None,
            nginx_extra_config: None,

            proxy_connect_timeout: None,

            proxy_read_timeout: None,

            proxy_send_timeout: None,
        };

        let result = config.validate("test-app");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("app_type"));
    }

    #[test]
    fn test_micro_app_config_validate_empty_routes_for_static() {
        let config = MicroAppConfig {
            package_type: "source".to_string(),
            image_archive: None,
            registry_image: None,
            build_platform: None,
            routes: vec![],
            container_name: "test-container".to_string(),
            container_port: 80,
            healthcheck_path: "/".to_string(),
            app_type: "static".to_string(),
            description: None,
            nginx_extra_config: None,

            proxy_connect_timeout: None,

            proxy_read_timeout: None,

            proxy_send_timeout: None,
        };

        let result = config.validate("test-app");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("routes 不能为空"));
    }

    #[test]
    fn test_micro_app_config_validate_internal_with_routes() {
        let config = MicroAppConfig {
            package_type: "source".to_string(),
            image_archive: None,
            registry_image: None,
            build_platform: None,
            routes: vec!["/".to_string()],
            container_name: "test-container".to_string(),
            container_port: 6379,
            healthcheck_path: "/".to_string(),
            app_type: "internal".to_string(),
            description: None,
            nginx_extra_config: None,

            proxy_connect_timeout: None,

            proxy_read_timeout: None,

            proxy_send_timeout: None,
        };

        // internal 类型有 routes 应该只是警告，不报错
        let result = config.validate("test-app");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_route_format_valid_root() {
        assert!(validate_route_format("/").is_ok());
    }

    #[test]
    fn test_validate_route_format_valid_path() {
        assert!(validate_route_format("/api").is_ok());
        assert!(validate_route_format("/gg123_test").is_ok());
        assert!(validate_route_format("/a/b/c").is_ok());
    }

    #[test]
    fn test_validate_route_format_trailing_slash() {
        let result = validate_route_format("/gg123_test/");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("/gg123_test/"));
    }

    #[test]
    fn test_validate_route_format_trailing_slash_root() {
        // "/" is the only exception — it is allowed to "end" with "/"
        assert!(validate_route_format("/").is_ok());
    }

    #[test]
    fn test_validate_route_format_empty() {
        let result = validate_route_format("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_route_format_no_leading_slash() {
        let result = validate_route_format("api");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_route_in_micro_app_config() {
        let config = MicroAppConfig {
            package_type: "source".to_string(),
            image_archive: None,
            registry_image: None,
            build_platform: None,
            routes: vec!["/gg123_test/".to_string()],
            container_name: "test-container".to_string(),
            container_port: 80,
            healthcheck_path: "/".to_string(),
            app_type: "static".to_string(),
            description: None,
            nginx_extra_config: None,

            proxy_connect_timeout: None,

            proxy_read_timeout: None,

            proxy_send_timeout: None,
        };

        let result = config.validate("test-app");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("/gg123_test/"));
    }
}
