
//! 应用发现模块
//!
//! 负责扫描微应用目录，发现包含 micro-app.yml 和 Dockerfile 的微应用

use crate::micro_app_config::MicroAppConfig;
use crate::volumes_config::VolumesConfig;
use crate::{Error, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// 微应用信息
#[derive(Debug, Clone)]
pub struct MicroApp {
    /// 微应用名称（唯一标识，由 scan_dir 相对路径推导）
    pub name: String,

    /// 微应用路径
    pub path: PathBuf,

    /// 微应用配置（从 micro-app.yml 加载）
    pub config: MicroAppConfig,

    /// 卷配置（从 micro-app.volumes.yml 加载，可选）
    pub volumes_config: VolumesConfig,

    /// Dockerfile 路径
    pub dockerfile: PathBuf,

    /// 环境变量文件路径
    pub env_file: PathBuf,

    /// 预构建脚本（可选）
    pub setup_script: Option<PathBuf>,

    /// 清理脚本（可选）
    pub clean_script: Option<PathBuf>,
}

impl MicroApp {
    /// 从目录创建微应用信息
    ///
    /// # 参数
    /// - `name`: 微应用唯一名称（由 scan_dir 相对路径推导）
    /// - `path`: 微应用绝对路径
    ///
    /// # 返回
    /// 返回 MicroApp 实例
    pub fn from_directory(name: String, path: PathBuf) -> Result<Self> {
        let micro_app_yml = path.join("micro-app.yml");
        let micro_app_volumes_yml = path.join("micro-app.volumes.yml");
        let dockerfile = path.join("Dockerfile");
        let env_file = path.join(".env");
        let setup_script = path.join("setup.sh");
        let clean_script = path.join("clean.sh");

        // 加载 micro-app.yml
        let config = MicroAppConfig::from_file(&micro_app_yml)?;

        // 验证配置
        config.validate(&name)?;

        // 加载 micro-app.volumes.yml（如果存在）
        log::debug!(
            "尝试加载卷配置: {:?}",
            micro_app_volumes_yml
        );
        let volumes_config = VolumesConfig::from_file(&micro_app_volumes_yml)?;
        
        // 验证卷配置
        volumes_config.validate(&name)?;

        Ok(MicroApp {
            name,
            path,
            config,
            volumes_config,
            dockerfile,
            env_file,
            setup_script: if setup_script.exists() {
                Some(setup_script)
            } else {
                None
            },
            clean_script: if clean_script.exists() {
                Some(clean_script)
            } else {
                None
            },
        })
    }

    /// 验证微应用是否有效
    pub fn validate(&self) -> Result<()> {
        log::debug!("验证微应用：{}", self.name);

        // 检查 Dockerfile 是否存在
        if !self.dockerfile.exists() {
            log::error!("微应用 '{}' 缺少 Dockerfile", self.name);
            return Err(Error::Discovery(format!(
                "微应用 '{}' 缺少 Dockerfile",
                self.name
            )));
        }

        // 检查.env 文件是否存在
        if !self.env_file.exists() {
            log::warn!("微应用 '{}' 缺少 .env 文件", self.name);
        }

        // 验证卷配置
        if let Err(e) = self.volumes_config.validate(&self.name) {
            log::error!("微应用 '{}' 的卷配置验证失败: {}", self.name, e);
            return Err(e);
        }

        log::debug!("微应用 '{}' 验证通过", self.name);
        Ok(())
    }

    /// 转换为 AppConfig
    pub fn to_app_config(&self) -> crate::config::AppConfig {
        use crate::config::{AppConfig, AppType};

        let app_type = match self.config.app_type.as_str() {
            "static" => AppType::Static,
            "api" => AppType::Api,
            "internal" => AppType::Internal,
            _ => AppType::Static, // 默认值，理论上不会到达这里
        };

        AppConfig {
            name: self.name.clone(),
            routes: self.config.routes.clone(),
            container_name: self.config.container_name.clone(),
            container_port: self.config.container_port,
            app_type,
            description: self.config.description.clone(),
            nginx_extra_config: self.config.nginx_extra_config.clone(),
            path: Some(self.path.to_str().unwrap_or("").to_string()),
            docker_volumes: self.volumes_config.to_docker_compose_volumes(),
            run_as_user: self.volumes_config.run_as_user.clone(),
            proxy_connect_timeout: self.config.proxy_connect_timeout,
            proxy_read_timeout: self.config.proxy_read_timeout,
            proxy_send_timeout: self.config.proxy_send_timeout,
        }
    }
}

/// 生成唯一的微应用名称
///
/// 基于 scan_dir 和应用目录的相对路径生成唯一名称
/// 格式：{scan_dir_relative_path}_{app_dir_name}
/// 例如：apps_craftaidhub_front
///
/// # 参数
/// - `scan_dir`: 扫描目录路径
/// - `app_dir`: 应用目录路径
///
/// # 返回
/// 返回唯一的微应用名称
fn generate_unique_app_name(scan_dir: &Path, app_dir: &Path) -> Result<String> {
    log::debug!("生成微应用名称 - scan_dir: {:?}, app_dir: {:?}", scan_dir, app_dir);

    // 获取应用目录的最后一级名称
    let app_name = app_dir
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| {
            log::error!("无法获取应用目录名称：{:?}", app_dir);
            Error::Discovery(format!("无法获取应用目录名称：{:?}", app_dir))
        })?
        .to_string();

    log::debug!("应用目录名称：{}", app_name);

    // 计算应用目录相对于 scan_dir 的路径
    let relative_path = match app_dir.strip_prefix(scan_dir) {
        Ok(p) => p,
        Err(e) => {
            log::error!(
                "无法计算相对路径 - scan_dir: {:?}, app_dir: {:?}, 错误：{}",
                scan_dir,
                app_dir,
                e
            );
            // 尝试使用父目录作为备选方案
            return Err(Error::Discovery(format!(
                "无法计算相对路径：scan_dir={:?}, app_dir={:?}, 错误：{}",
                scan_dir, app_dir, e
            )));
        }
    };

    log::debug!("相对路径：{:?}", relative_path);

    // 获取相对路径的所有组件（不包括最后一级目录名）
    let mut path_components: Vec<&str> = relative_path
        .iter()
        .filter_map(|p| p.to_str())
        .collect();

    log::debug!("路径组件：{:?}", path_components);

    // 移除最后一级目录名（即 app_name），因为我们单独处理它
    if !path_components.is_empty() && path_components.last() == Some(&app_name.as_str()) {
        path_components.pop();
        log::debug!("移除最后一级目录名后的路径组件：{:?}", path_components);
    }

    // 将相对路径转换为字符串，并用下划线连接各层级
    let unique_name = if path_components.is_empty() {
        // 如果相对路径为空（即应用目录就是 scan_dir 的直接子目录），仅使用应用名称
        log::debug!("应用目录为 scan_dir 直接子目录，仅使用目录名：{}", app_name);
        app_name
    } else {
        // 组合相对路径和最后一级目录名
        let path_str = path_components.join("_");
        let unique_name = format!("{}_{}", path_str, app_name);
        log::debug!("生成的唯一应用名称：{}", unique_name);
        unique_name
    };

    Ok(unique_name)
}

/// 发现微应用
///
/// # 参数
/// - `scan_dirs`: 扫描目录列表
///
/// # 返回
/// 返回发现的微应用列表
///
/// # 错误
/// - 如果扫描到相同名称的目录，返回错误
/// - 如果 container_name 存在重复，返回错误
pub fn discover_micro_apps(scan_dirs: &[String]) -> Result<Vec<MicroApp>> {
    log::info!("开始扫描微应用目录...");
    log::debug!("扫描目录：{:?}", scan_dirs);

    let mut micro_apps: Vec<MicroApp> = Vec::new();
    let mut app_names = HashSet::new();
    let mut container_names = HashSet::new();

    for scan_dir in scan_dirs {
        let dir_path = Path::new(scan_dir);

        if !dir_path.exists() {
            log::warn!("扫描目录不存在：{:?}", scan_dir);
            continue;
        }

        log::debug!("正在扫描目录：{:?}", scan_dir);

        // 遍历目录
        let entries = match std::fs::read_dir(dir_path) {
            Ok(e) => e,
            Err(e) => {
                log::error!("读取目录失败：{:?}, 错误：{}", scan_dir, e);
                return Err(Error::Discovery(format!("读取目录 {:?} 失败：{}", scan_dir, e)));
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    log::error!("读取目录项失败：{}", e);
                    return Err(Error::Discovery(format!("读取目录项失败：{}", e)));
                }
            };

            let path = entry.path();

            // 只处理目录
            if !path.is_dir() {
                log::trace!("跳过非目录项：{:?}", path);
                continue;
            }

            // 检查是否包含 micro-app.yml
            let micro_app_yml = path.join("micro-app.yml");
            if !micro_app_yml.exists() {
                log::trace!("目录 {:?} 不包含 micro-app.yml，跳过", path);
                continue;
            }

            log::debug!("发现微应用目录：{:?}", path);

            // 生成唯一的微应用名称（基于相对路径）
            let name = match generate_unique_app_name(dir_path, &path) {
                Ok(n) => {
                    log::debug!("生成的微应用名称：{}", n);
                    n
                }
                Err(e) => {
                    log::warn!("生成微应用名称失败：{}, 跳过目录：{:?}", e, path);
                    continue;
                }
            };

            // 检查名称是否重复
            if app_names.contains(&name) {
                log::error!("发现重复的微应用名称：'{}', 路径：{:?}", name, path);
                return Err(Error::Discovery(format!(
                    "发现重复的微应用名称：'{}', 请确保所有扫描目录中的微应用名称唯一",
                    name
                )));
            }

            // 创建微应用信息
            let micro_app = match MicroApp::from_directory(name.clone(), path) {
                Ok(app) => app,
                Err(e) => {
                    log::warn!("微应用 '{}' 加载失败：{}, 跳过", name, e);
                    continue;
                }
            };

            // 验证微应用
            if let Err(e) = micro_app.validate() {
                log::warn!("微应用 '{}' 验证失败：{}, 跳过", name, e);
                continue;
            }

            // 检查 container_name 是否重复
            let container_name = micro_app.config.container_name.clone();
            if container_names.contains(&container_name) {
                log::error!(
                    "发现重复的容器名称：'{}', 微应用：'{}'",
                    container_name,
                    name
                );
                return Err(Error::Discovery(format!(
                    "发现重复的容器名称：'{}', 微应用：'{}'",
                    container_name, name
                )));
            }

            // 添加到集合
            app_names.insert(name.clone());
            container_names.insert(container_name);
            log::info!(
                "发现有效微应用：{} (容器：{})",
                name,
                micro_app.config.container_name
            );
            micro_apps.push(micro_app);
        }
    }

    log::info!("扫描完成，共发现 {} 个微应用", micro_apps.len());
    Ok(micro_apps)
}

/// 获取微应用名称列表
///
/// # 参数
/// - `micro_apps`: 微应用列表
///
/// # 返回
/// 返回微应用名称列表
pub fn get_micro_app_names(micro_apps: &[MicroApp]) -> Vec<String> {
    micro_apps.iter().map(|app| app.name.clone()).collect()
}

/// 将微应用列表转换为 AppConfig 列表
///
/// # 参数
/// - `micro_apps`: 微应用列表
///
/// # 返回
/// 返回 AppConfig 列表
pub fn to_app_configs(micro_apps: &[MicroApp]) -> Vec<crate::config::AppConfig> {
    micro_apps.iter().map(|app| app.to_app_config()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_generate_unique_app_name_same_level() {
        let temp_dir = TempDir::new().unwrap();
        let scan_dir = temp_dir.path();
        let app_dir = scan_dir.join("craftaidhub_front");
        std::fs::create_dir(&app_dir).unwrap();

        let name = generate_unique_app_name(scan_dir, &app_dir).unwrap();
        assert_eq!(name, "craftaidhub_front");
    }

    #[test]
    fn test_generate_unique_app_name_nested() {
        let temp_dir = TempDir::new().unwrap();
        let scan_dir = temp_dir.path();
        let apps_dir = scan_dir.join("apps");
        let app_dir = apps_dir.join("craftaidhub_front");
        std::fs::create_dir_all(&app_dir).unwrap();

        let name = generate_unique_app_name(scan_dir, &app_dir).unwrap();
        assert_eq!(name, "apps_craftaidhub_front");
    }

    #[test]
    fn test_generate_unique_app_name_deeply_nested() {
        let temp_dir = TempDir::new().unwrap();
        let scan_dir = temp_dir.path();
        let services_dir = scan_dir.join("services");
        let backend_dir = services_dir.join("backend");
        let app_dir = backend_dir.join("api");
        std::fs::create_dir_all(&app_dir).unwrap();

        let name = generate_unique_app_name(scan_dir, &app_dir).unwrap();
        assert_eq!(name, "services_backend_api");
    }

    #[test]
    fn test_micro_app_from_directory() {
        let temp_dir = TempDir::new().unwrap();
        let app_path = temp_dir.path().join("test-app");

        std::fs::create_dir(&app_path).unwrap();
        std::fs::write(app_path.join("Dockerfile"), "FROM nginx").unwrap();
        
        let micro_app_yml = r#"
routes: ["/"]
container_name: "test-container"
container_port: 80
app_type: "static"
description: "Test app"
"#;
        std::fs::write(app_path.join("micro-app.yml"), micro_app_yml).unwrap();
        std::fs::write(app_path.join(".env"), "PORT=80").unwrap();
        std::fs::write(app_path.join("setup.sh"), "#!/bin/bash\necho setup").unwrap();

        let micro_app = MicroApp::from_directory("test-app".to_string(), app_path.clone()).unwrap();

        assert_eq!(micro_app.name, "test-app");
        assert_eq!(micro_app.path, app_path);
        assert_eq!(micro_app.config.container_name, "test-container");
        assert_eq!(micro_app.config.container_port, 80);
        assert_eq!(micro_app.config.app_type, "static");
        assert!(micro_app.setup_script.is_some());
        assert!(micro_app.clean_script.is_none());
        // 检查卷配置为空
        assert_eq!(micro_app.volumes_config.volumes.len(), 0);
        assert!(micro_app.volumes_config.run_as_user.is_none());
    }

    #[test]
    fn test_micro_app_from_directory_with_volumes() {
        let temp_dir = TempDir::new().unwrap();
        let app_path = temp_dir.path().join("test-app");

        std::fs::create_dir(&app_path).unwrap();
        std::fs::write(app_path.join("Dockerfile"), "FROM nginx").unwrap();
        
        let micro_app_yml = r#"
routes: ["/"]
container_name: "test-container"
container_port: 80
app_type: "static"
description: "Test app"
"#;
        std::fs::write(app_path.join("micro-app.yml"), micro_app_yml).unwrap();

        let volumes_yml = r#"
volumes:
  - source: "./data"
    target: "/app/data"
    permissions:
      uid: 999
      gid: 999
      recursive: true
run_as_user: "999:999"
"#;
        std::fs::write(app_path.join("micro-app.volumes.yml"), volumes_yml).unwrap();

        let micro_app = MicroApp::from_directory("test-app".to_string(), app_path).unwrap();

        assert_eq!(micro_app.name, "test-app");
        assert_eq!(micro_app.volumes_config.volumes.len(), 1);
        assert_eq!(micro_app.volumes_config.volumes[0].source, "./data");
        assert_eq!(micro_app.volumes_config.run_as_user, Some("999:999".to_string()));
    }

    #[test]
    fn test_micro_app_validate_success() {
        let temp_dir = TempDir::new().unwrap();
        let app_path = temp_dir.path().join("test-app");

        std::fs::create_dir(&app_path).unwrap();
        std::fs::write(app_path.join("Dockerfile"), "FROM nginx").unwrap();
        
        let micro_app_yml = r#"
routes: ["/"]
container_name: "test-container"
container_port: 80
app_type: "static"
"#;
        std::fs::write(app_path.join("micro-app.yml"), micro_app_yml).unwrap();

        let micro_app = MicroApp::from_directory("test-app".to_string(), app_path).unwrap();
        let result = micro_app.validate();

        assert!(result.is_ok());
    }

    #[test]
    fn test_micro_app_validate_no_dockerfile() {
        let temp_dir = TempDir::new().unwrap();
        let app_path = temp_dir.path().join("test-app");

        std::fs::create_dir(&app_path).unwrap();
        
        let micro_app_yml = r#"
routes: ["/"]
container_name: "test-container"
container_port: 80
app_type: "static"
"#;
        std::fs::write(app_path.join("micro-app.yml"), micro_app_yml).unwrap();

        let micro_app = MicroApp::from_directory("test-app".to_string(), app_path).unwrap();
        let result = micro_app.validate();

        assert!(result.is_err());
    }

    #[test]
    fn test_discover_micro_apps() {
        let temp_dir = TempDir::new().unwrap();

        // 创建有效的微应用
        let app1_path = temp_dir.path().join("app1");
        std::fs::create_dir(&app1_path).unwrap();
        std::fs::write(app1_path.join("Dockerfile"), "FROM nginx").unwrap();
        let micro_app_yml1 = r#"
routes: ["/"]
container_name: "container1"
container_port: 80
app_type: "static"
"#;
        std::fs::write(app1_path.join("micro-app.yml"), micro_app_yml1).unwrap();

        // 创建无效的微应用（没有 Dockerfile）
        let app2_path = temp_dir.path().join("app2");
        std::fs::create_dir(&app2_path).unwrap();
        let micro_app_yml2 = r#"
routes: ["/"]
container_name: "container2"
container_port: 80
app_type: "static"
"#;
        std::fs::write(app2_path.join("micro-app.yml"), micro_app_yml2).unwrap();

        // 创建没有 micro-app.yml 的目录（应该被跳过）
        let app3_path = temp_dir.path().join("app3");
        std::fs::create_dir(&app3_path).unwrap();
        std::fs::write(app3_path.join("Dockerfile"), "FROM nginx").unwrap();

        let scan_dirs = vec![temp_dir.path().to_str().unwrap().to_string()];
        let result = discover_micro_apps(&scan_dirs);

        assert!(result.is_ok());
        let apps = result.unwrap();
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].name, "app1");
    }

    #[test]
    fn test_discover_micro_apps_duplicate_name() {
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();

        // 在第一个目录创建 app1
        let app1_path = temp_dir1.path().join("app1");
        std::fs::create_dir(&app1_path).unwrap();
        std::fs::write(app1_path.join("Dockerfile"), "FROM nginx").unwrap();
        let micro_app_yml1 = r#"
routes: ["/"]
container_name: "container1"
container_port: 80
app_type: "static"
"#;
        std::fs::write(app1_path.join("micro-app.yml"), micro_app_yml1).unwrap();

        // 在第二个目录也创建 app1（同名）
        let app1_path2 = temp_dir2.path().join("app1");
        std::fs::create_dir(&app1_path2).unwrap();
        std::fs::write(app1_path2.join("Dockerfile"), "FROM nginx").unwrap();
        let micro_app_yml2 = r#"
routes: ["/api"]
container_name: "container2"
container_port: 8080
app_type: "api"
"#;
        std::fs::write(app1_path2.join("micro-app.yml"), micro_app_yml2).unwrap();

        let scan_dirs = vec![
            temp_dir1.path().to_str().unwrap().to_string(),
            temp_dir2.path().to_str().unwrap().to_string(),
        ];
        let result = discover_micro_apps(&scan_dirs);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("重复的微应用名称"));
    }

    #[test]
    fn test_discover_micro_apps_duplicate_container_name() {
        let temp_dir = TempDir::new().unwrap();

        // 创建第一个微应用
        let app1_path = temp_dir.path().join("app1");
        std::fs::create_dir(&app1_path).unwrap();
        std::fs::write(app1_path.join("Dockerfile"), "FROM nginx").unwrap();
        let micro_app_yml1 = r#"
routes: ["/"]
container_name: "same-container"
container_port: 80
app_type: "static"
"#;
        std::fs::write(app1_path.join("micro-app.yml"), micro_app_yml1).unwrap();

        // 创建第二个微应用（相同的 container_name）
        let app2_path = temp_dir.path().join("app2");
        std::fs::create_dir(&app2_path).unwrap();
        std::fs::write(app2_path.join("Dockerfile"), "FROM nginx").unwrap();
        let micro_app_yml2 = r#"
routes: ["/api"]
container_name: "same-container"
container_port: 8080
app_type: "api"
"#;
        std::fs::write(app2_path.join("micro-app.yml"), micro_app_yml2).unwrap();

        let scan_dirs = vec![temp_dir.path().to_str().unwrap().to_string()];
        let result = discover_micro_apps(&scan_dirs);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("重复的容器名称"));
    }

    #[test]
    fn test_get_micro_app_names() {
        let temp_dir = TempDir::new().unwrap();

        let app1_path = temp_dir.path().join("app1");
        std::fs::create_dir(&app1_path).unwrap();
        std::fs::write(app1_path.join("Dockerfile"), "FROM nginx").unwrap();
        let micro_app_yml1 = r#"
routes: ["/"]
container_name: "container1"
container_port: 80
app_type: "static"
"#;
        std::fs::write(app1_path.join("micro-app.yml"), micro_app_yml1).unwrap();

        let app2_path = temp_dir.path().join("app2");
        std::fs::create_dir(&app2_path).unwrap();
        std::fs::write(app2_path.join("Dockerfile"), "FROM nginx").unwrap();
        let micro_app_yml2 = r#"
routes: ["/api"]
container_name: "container2"
container_port: 8080
app_type: "api"
"#;
        std::fs::write(app2_path.join("micro-app.yml"), micro_app_yml2).unwrap();

        let scan_dirs = vec![temp_dir.path().to_str().unwrap().to_string()];
        let apps = discover_micro_apps(&scan_dirs).unwrap();
        let names = get_micro_app_names(&apps);

        assert_eq!(names, vec!["app2".to_string(), "app1".to_string()]);
    }

    #[test]
    fn test_to_app_configs() {
        let temp_dir = TempDir::new().unwrap();

        let app1_path = temp_dir.path().join("app1");
        std::fs::create_dir(&app1_path).unwrap();
        std::fs::write(app1_path.join("Dockerfile"), "FROM nginx").unwrap();
        let micro_app_yml1 = r#"
routes: ["/"]
container_name: "container1"
container_port: 80
app_type: "static"
description: "Test app"
"#;
        std::fs::write(app1_path.join("micro-app.yml"), micro_app_yml1).unwrap();

        let volumes_yml = r#"
volumes:
  - source: "./data"
    target: "/app/data"
    permissions:
      uid: 999
      gid: 999
      recursive: true
run_as_user: "999:999"
"#;
        std::fs::write(app1_path.join("micro-app.volumes.yml"), volumes_yml).unwrap();

        let scan_dirs = vec![temp_dir.path().to_str().unwrap().to_string()];
        let micro_apps = discover_micro_apps(&scan_dirs).unwrap();
        let app_configs = to_app_configs(&micro_apps);

        assert_eq!(app_configs.len(), 1);
        assert_eq!(app_configs[0].name, "app1");
        assert_eq!(app_configs[0].container_name, "container1");
        assert_eq!(app_configs[0].container_port, 80);
        assert_eq!(app_configs[0].routes, vec!["/"]);
        assert_eq!(app_configs[0].description, Some("Test app".to_string()));
        assert_eq!(app_configs[0].docker_volumes, vec!["./data:/app/data"]);
        assert_eq!(app_configs[0].run_as_user, Some("999:999".to_string()));
        assert!(app_configs[0].path.is_some());
    }
}
