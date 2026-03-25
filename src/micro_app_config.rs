
//! 微应用配置模块
//!
//! 负责解析每个微应用目录下的 micro-app.yml 文件

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 微应用配置文件结构（micro-app.yml）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroAppConfig {
    /// 访问路径（static/api类型必需）
    #[serde(default)]
    pub routes: Vec<String>,

    /// 容器名称（必需，全局唯一）
    pub container_name: String,

    /// 容器内部端口（必需）
    pub container_port: u16,

    /// 应用类型（必需）
    pub app_type: String, // 使用String，后续转换为AppType

    /// 应用描述（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// 额外的 nginx 配置（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nginx_extra_config: Option<String>,
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

        // internal 类型不应该配置 routes
        if self.app_type == "internal" && !self.routes.is_empty() {
            log::warn!(
                "微应用 '{}' 是 internal 类型，routes 配置将被忽略",
                app_name
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
        assert_eq!(config.description, Some("Test API service".to_string()));
        assert!(config.nginx_extra_config.is_some());
    }

    #[test]
    fn test_micro_app_config_validate_success() {
        let config = MicroAppConfig {
            routes: vec!["/".to_string()],
            container_name: "test-container".to_string(),
            container_port: 80,
            app_type: "static".to_string(),
            description: None,
            nginx_extra_config: None,
        };

        let result = config.validate("test-app");
        assert!(result.is_ok());
    }

    #[test]
    fn test_micro_app_config_validate_empty_container_name() {
        let config = MicroAppConfig {
            routes: vec!["/".to_string()],
            container_name: "".to_string(),
            container_port: 80,
            app_type: "static".to_string(),
            description: None,
            nginx_extra_config: None,
        };

        let result = config.validate("test-app");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("container_name 不能为空"));
    }

    #[test]
    fn test_micro_app_config_validate_zero_port() {
        let config = MicroAppConfig {
            routes: vec!["/".to_string()],
            container_name: "test-container".to_string(),
            container_port: 0,
            app_type: "static".to_string(),
            description: None,
            nginx_extra_config: None,
        };

        let result = config.validate("test-app");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("container_port 不能为 0"));
    }

    #[test]
    fn test_micro_app_config_validate_invalid_app_type() {
        let config = MicroAppConfig {
            routes: vec!["/".to_string()],
            container_name: "test-container".to_string(),
            container_port: 80,
            app_type: "invalid".to_string(),
            description: None,
            nginx_extra_config: None,
        };

        let result = config.validate("test-app");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("app_type"));
    }

    #[test]
    fn test_micro_app_config_validate_empty_routes_for_static() {
        let config = MicroAppConfig {
            routes: vec![],
            container_name: "test-container".to_string(),
            container_port: 80,
            app_type: "static".to_string(),
            description: None,
            nginx_extra_config: None,
        };

        let result = config.validate("test-app");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("routes 不能为空"));
    }

    #[test]
    fn test_micro_app_config_validate_internal_with_routes() {
        let config = MicroAppConfig {
            routes: vec!["/".to_string()],
            container_name: "test-container".to_string(),
            container_port: 6379,
            app_type: "internal".to_string(),
            description: None,
            nginx_extra_config: None,
        };

        // internal 类型有 routes 应该只是警告，不报错
        let result = config.validate("test-app");
        assert!(result.is_ok());
    }
}
