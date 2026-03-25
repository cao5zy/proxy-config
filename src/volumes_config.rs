
//! 卷配置模块
//!
//! 负责解析每个微应用目录下的 micro-app.volumes.yml 文件

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 卷权限配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumePermissions {
    /// 用户ID
    pub uid: u32,

    /// 组ID
    pub gid: u32,

    /// 是否递归设置权限
    #[serde(default = "default_recursive")]
    pub recursive: bool,
}

/// 默认递归设置为 true
fn default_recursive() -> bool {
    true
}

/// 单个卷配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeConfig {
    /// 宿主机路径（源路径）
    pub source: String,

    /// 容器内路径（目标路径）
    pub target: String,

    /// 权限配置（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<VolumePermissions>,
}

/// 卷配置文件结构（micro-app.volumes.yml）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumesConfig {
    /// 卷列表
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub volumes: Vec<VolumeConfig>,

    /// 容器运行用户（格式: "uid:gid" 或 "username"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_as_user: Option<String>,
}

impl VolumesConfig {
    /// 从文件加载卷配置
    pub fn from_file<P: Into<PathBuf>>(path: P) -> Result<Self> {
        let path = path.into();
        log::debug!("正在加载卷配置: {:?}", path);

        // 如果文件不存在，返回空配置
        if !path.exists() {
            log::debug!("卷配置文件不存在: {:?}, 使用空配置", path);
            return Ok(VolumesConfig {
                volumes: vec![],
                run_as_user: None,
            });
        }

        let content = std::fs::read_to_string(&path).map_err(|e| {
            log::error!("读取卷配置文件失败: {:?}, 错误: {}", path, e);
            Error::Config(format!("无法读取卷配置文件 {:?}: {}", path, e))
        })?;

        let config: VolumesConfig = serde_yaml::from_str(&content).map_err(|e| {
            log::error!("解析卷配置文件失败: {:?}, 错误: {}", path, e);
            Error::Config(format!("解析卷配置文件 {:?} 失败: {}", path, e))
        })?;

        log::debug!("卷配置加载成功: {:?}", config);
        Ok(config)
    }

    /// 验证卷配置
    pub fn validate(&self, app_name: &str) -> Result<()> {
        log::debug!("验证应用 '{}' 的卷配置", app_name);

        for (index, volume) in self.volumes.iter().enumerate() {
            // 验证 source 不为空
            if volume.source.is_empty() {
                log::error!(
                    "应用 '{}' 的第 {} 个卷配置中 source 不能为空",
                    app_name,
                    index + 1
                );
                return Err(Error::Config(format!(
                    "应用 '{}' 的第 {} 个卷配置中 source 不能为空",
                    app_name,
                    index + 1
                )));
            }

            // 验证 target 不为空
            if volume.target.is_empty() {
                log::error!(
                    "应用 '{}' 的第 {} 个卷配置中 target 不能为空",
                    app_name,
                    index + 1
                );
                return Err(Error::Config(format!(
                    "应用 '{}' 的第 {} 个卷配置中 target 不能为空",
                    app_name,
                    index + 1
                )));
            }

            // 验证权限配置
            if let Some(ref perms) = volume.permissions {
                if perms.uid == 0 || perms.gid == 0 {
                    log::warn!(
                        "应用 '{}' 的第 {} 个卷配置使用了 root 权限 (uid=0 或 gid=0)，这可能存在安全风险",
                        app_name,
                        index + 1
                    );
                }
            }
        }

        // 验证 run_as_user 格式
        if let Some(ref user) = self.run_as_user {
            if user.is_empty() {
                log::error!("应用 '{}' 的 run_as_user 不能为空字符串", app_name);
                return Err(Error::Config(format!(
                    "应用 '{}' 的 run_as_user 不能为空字符串",
                    app_name
                )));
            }
            log::debug!("应用 '{}' 的 run_as_user: {}", app_name, user);
        }

        log::debug!("应用 '{}' 卷配置验证通过", app_name);
        Ok(())
    }

    /// 生成权限初始化脚本
    pub fn generate_permission_init_script(&self) -> Option<String> {
        if self.volumes.is_empty() {
            log::debug!("没有配置卷，无需生成权限初始化脚本");
            return None;
        }

        let mut commands = Vec::new();
        let mut has_permission_config = false;

        for volume in &self.volumes {
            if let Some(ref perms) = volume.permissions {
                has_permission_config = true;

                let chown_cmd = if perms.recursive {
                    format!(
                        "chown -R {}:{} \"{}\"",
                        perms.uid, perms.gid, volume.source
                    )
                } else {
                    format!("chown {}:{} \"{}\"", perms.uid, perms.gid, volume.source)
                };

                commands.push(chown_cmd);
                log::debug!("添加权限设置命令: {}", commands.last().unwrap());
            }
        }

        if !has_permission_config {
            log::debug!("没有配置卷权限，无需生成权限初始化脚本");
            return None;
        }

        let script = format!(
            r#"#!/bin/sh
# Docker 容器权限初始化脚本
# 由 proxy-config 自动生成

set -e

echo "开始设置卷权限..."

{}

echo "卷权限设置完成"
"#,
            commands.join("\n")
        );

        log::debug!("生成的权限初始化脚本长度: {} 字节", script.len());
        Some(script)
    }

    /// 转换为 Docker Compose volumes 格式
    pub fn to_docker_compose_volumes(&self) -> Vec<String> {
        self.volumes
            .iter()
            .map(|v| format!("{}:{}", v.source, v.target))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_volumes_config_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("micro-app.volumes.yml");

        let yaml_content = r#"
volumes:
  - source: "./data"
    target: "/app/data"
    permissions:
      uid: 999
      gid: 999
      recursive: true
  - source: "./logs"
    target: "/var/log/app"
    permissions:
      uid: 1000
      gid: 1000
      recursive: false
run_as_user: "999:999"
"#;

        std::fs::write(&config_path, yaml_content).unwrap();

        let config = VolumesConfig::from_file(&config_path).unwrap();
        assert_eq!(config.volumes.len(), 2);
        assert_eq!(config.volumes[0].source, "./data");
        assert_eq!(config.volumes[0].target, "/app/data");
        assert_eq!(config.volumes[0].permissions.as_ref().unwrap().uid, 999);
        assert_eq!(config.volumes[0].permissions.as_ref().unwrap().gid, 999);
        assert_eq!(config.volumes[0].permissions.as_ref().unwrap().recursive, true);
        assert_eq!(config.volumes[1].source, "./logs");
        assert_eq!(config.volumes[1].target, "/var/log/app");
        assert_eq!(config.volumes[1].permissions.as_ref().unwrap().uid, 1000);
        assert_eq!(config.volumes[1].permissions.as_ref().unwrap().gid, 1000);
        assert_eq!(config.volumes[1].permissions.as_ref().unwrap().recursive, false);
        assert_eq!(config.run_as_user, Some("999:999".to_string()));
    }

    #[test]
    fn test_volumes_config_from_file_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.yml");

        let config = VolumesConfig::from_file(&config_path).unwrap();
        assert_eq!(config.volumes.len(), 0);
        assert!(config.run_as_user.is_none());
    }

    #[test]
    fn test_volumes_config_validate_success() {
        let config = VolumesConfig {
            volumes: vec![
                VolumeConfig {
                    source: "./data".to_string(),
                    target: "/app/data".to_string(),
                    permissions: Some(VolumePermissions {
                        uid: 999,
                        gid: 999,
                        recursive: true,
                    }),
                },
            ],
            run_as_user: Some("999:999".to_string()),
        };

        let result = config.validate("test-app");
        assert!(result.is_ok());
    }

    #[test]
    fn test_volumes_config_validate_empty_source() {
        let config = VolumesConfig {
            volumes: vec![VolumeConfig {
                source: "".to_string(),
                target: "/app/data".to_string(),
                permissions: None,
            }],
            run_as_user: None,
        };

        let result = config.validate("test-app");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("source 不能为空"));
    }

    #[test]
    fn test_volumes_config_validate_empty_target() {
        let config = VolumesConfig {
            volumes: vec![VolumeConfig {
                source: "./data".to_string(),
                target: "".to_string(),
                permissions: None,
            }],
            run_as_user: None,
        };

        let result = config.validate("test-app");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("target 不能为空"));
    }

    #[test]
    fn test_volumes_config_validate_empty_run_as_user() {
        let config = VolumesConfig {
            volumes: vec![],
            run_as_user: Some("".to_string()),
        };

        let result = config.validate("test-app");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("run_as_user 不能为空"));
    }

    #[test]
    fn test_generate_permission_init_script() {
        let config = VolumesConfig {
            volumes: vec![
                VolumeConfig {
                    source: "./data".to_string(),
                    target: "/app/data".to_string(),
                    permissions: Some(VolumePermissions {
                        uid: 999,
                        gid: 999,
                        recursive: true,
                    }),
                },
                VolumeConfig {
                    source: "./logs".to_string(),
                    target: "/var/log/app".to_string(),
                    permissions: Some(VolumePermissions {
                        uid: 1000,
                        gid: 1000,
                        recursive: false,
                    }),
                },
            ],
            run_as_user: Some("999:999".to_string()),
        };

        let script = config.generate_permission_init_script().unwrap();
        assert!(script.contains("#!/bin/sh"));
        assert!(script.contains("chown -R 999:999 \"./data\""));
        assert!(script.contains("chown 1000:1000 \"./logs\""));
    }

    #[test]
    fn test_generate_permission_init_script_no_volumes() {
        let config = VolumesConfig {
            volumes: vec![],
            run_as_user: None,
        };

        let script = config.generate_permission_init_script();
        assert!(script.is_none());
    }

    #[test]
    fn test_generate_permission_init_script_no_permissions() {
        let config = VolumesConfig {
            volumes: vec![VolumeConfig {
                source: "./data".to_string(),
                target: "/app/data".to_string(),
                permissions: None,
            }],
            run_as_user: None,
        };

        let script = config.generate_permission_init_script();
        assert!(script.is_none());
    }

    #[test]
    fn test_to_docker_compose_volumes() {
        let config = VolumesConfig {
            volumes: vec![
                VolumeConfig {
                    source: "./data".to_string(),
                    target: "/app/data".to_string(),
                    permissions: None,
                },
                VolumeConfig {
                    source: "./logs".to_string(),
                    target: "/var/log/app".to_string(),
                    permissions: None,
                },
            ],
            run_as_user: None,
        };

        let volumes = config.to_docker_compose_volumes();
        assert_eq!(volumes.len(), 2);
        assert_eq!(volumes[0], "./data:/app/data");
        assert_eq!(volumes[1], "./logs:/var/log/app");
    }

    #[test]
    fn test_default_recursive() {
        let config = VolumesConfig {
            volumes: vec![VolumeConfig {
                source: "./data".to_string(),
                target: "/app/data".to_string(),
                permissions: Some(VolumePermissions {
                    uid: 999,
                    gid: 999,
                    recursive: default_recursive(),
                }),
            }],
            run_as_user: None,
        };

        assert!(config.volumes[0].permissions.as_ref().unwrap().recursive);
    }
}
