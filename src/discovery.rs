
//! 应用发现模块
//!
//! 负责扫描微应用目录，发现符合条件的微应用

use crate::{Error, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// 微应用信息
#[derive(Debug, Clone)]
pub struct MicroApp {
    /// 微应用名称（即文件夹名称）
    pub name: String,

    /// 微应用路径
    pub path: PathBuf,

    /// 环境变量文件路径
    pub env_file: PathBuf,

    /// Dockerfile路径
    pub dockerfile: PathBuf,

    /// 预构建脚本（可选）
    pub setup_script: Option<PathBuf>,

    /// 清理脚本（可选）
    pub clean_script: Option<PathBuf>,
}

impl MicroApp {
    /// 创建新的微应用信息
    ///
    /// # 参数
    /// - `name`: 微应用名称
    /// - `path`: 微应用路径
    ///
    /// # 返回
    /// 返回微应用信息对象
    pub fn new(name: String, path: PathBuf) -> Self {
        let env_file = path.join(".env");
        let dockerfile = path.join("Dockerfile");
        let setup_script = path.join("setup.sh");
        let clean_script = path.join("clean.sh");

        MicroApp {
            name,
            path,
            env_file,
            dockerfile,
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
        }
    }

    /// 验证微应用是否有效
    ///
    /// # 返回
    /// 如果微应用有效，返回 Ok(())，否则返回错误
    pub fn validate(&self) -> Result<()> {
        log::debug!("验证微应用: {}", self.name);

        // 检查Dockerfile是否存在
        if !self.dockerfile.exists() {
            log::error!("微应用 '{}' 缺少 Dockerfile", self.name);
            return Err(Error::Discovery(format!(
                "微应用 '{}' 缺少 Dockerfile",
                self.name
            )));
        }

        // 检查.env文件是否存在
        if !self.env_file.exists() {
            log::warn!("微应用 '{}' 缺少 .env 文件", self.name);
        }

        log::debug!("微应用 '{}' 验证通过", self.name);
        Ok(())
    }
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
pub fn discover_micro_apps(scan_dirs: &[String]) -> Result<Vec<MicroApp>> {
    log::info!("开始扫描微应用目录...");
    log::debug!("扫描目录: {:?}", scan_dirs);

    let mut micro_apps: Vec<MicroApp> = Vec::new();
    let mut app_names = HashSet::new();

    for scan_dir in scan_dirs {
        let dir_path = Path::new(scan_dir);

        if !dir_path.exists() {
            log::warn!("扫描目录不存在: {:?}", scan_dir);
            continue;
        }

        log::debug!("正在扫描目录: {:?}", scan_dir);

        // 遍历目录
        let entries = std::fs::read_dir(dir_path).map_err(|e| {
            log::error!("读取目录失败: {:?}, 错误: {}", scan_dir, e);
            Error::Discovery(format!("读取目录 {:?} 失败: {}", scan_dir, e))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                log::error!("读取目录项失败: {}", e);
                Error::Discovery(format!("读取目录项失败: {}", e))
            })?;

            let path = entry.path();

            // 只处理目录
            if !path.is_dir() {
                log::trace!("跳过非目录项: {:?}", path);
                continue;
            }

            // 获取目录名称作为微应用名称
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| {
                    log::error!("无法获取目录名称: {:?}", path);
                    Error::Discovery(format!("无法获取目录名称: {:?}", path))
                })?
                .to_string();

            log::debug!("发现微应用: {}", name);

            // 检查名称是否重复
            if app_names.contains(&name) {
                log::error!("发现重复的微应用名称: '{}', 路径: {:?}", name, path);
                log::error!("已存在的微应用路径: {:?}", micro_apps.iter().find(|app| app.name == name).map(|app| &app.path));
                return Err(Error::Discovery(format!(
                    "发现重复的微应用名称: '{}', 请确保所有扫描目录中的微应用名称唯一",
                    name
                )));
            }

            // 创建微应用信息
            let micro_app = MicroApp::new(name.clone(), path);

            // 验证微应用
            if let Err(e) = micro_app.validate() {
                log::warn!("微应用 '{}' 验证失败: {}, 跳过", name, e);
                continue;
            }

            // 添加到名称集合
            app_names.insert(name.clone());
            micro_apps.push(micro_app);
            log::info!("发现有效微应用: {}", name);
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_micro_app_new() {
        let temp_dir = TempDir::new().unwrap();
        let app_path = temp_dir.path().join("test-app");

        std::fs::create_dir(&app_path).unwrap();
        std::fs::write(app_path.join("Dockerfile"), "FROM nginx").unwrap();
        std::fs::write(app_path.join(".env"), "PORT=80").unwrap();
        std::fs::write(app_path.join("setup.sh"), "#!/bin/bash\necho setup").unwrap();

        let micro_app = MicroApp::new("test-app".to_string(), app_path.clone());

        assert_eq!(micro_app.name, "test-app");
        assert_eq!(micro_app.path, app_path);
        assert_eq!(micro_app.dockerfile, app_path.join("Dockerfile"));
        assert_eq!(micro_app.env_file, app_path.join(".env"));
        assert!(micro_app.setup_script.is_some());
        assert!(micro_app.clean_script.is_none());
    }

    #[test]
    fn test_micro_app_validate_success() {
        let temp_dir = TempDir::new().unwrap();
        let app_path = temp_dir.path().join("test-app");

        std::fs::create_dir(&app_path).unwrap();
        std::fs::write(app_path.join("Dockerfile"), "FROM nginx").unwrap();

        let micro_app = MicroApp::new("test-app".to_string(), app_path);
        let result = micro_app.validate();

        assert!(result.is_ok());
    }

    #[test]
    fn test_micro_app_validate_no_dockerfile() {
        let temp_dir = TempDir::new().unwrap();
        let app_path = temp_dir.path().join("test-app");

        std::fs::create_dir(&app_path).unwrap();

        let micro_app = MicroApp::new("test-app".to_string(), app_path);
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

        // 创建无效的微应用（没有Dockerfile）
        let app2_path = temp_dir.path().join("app2");
        std::fs::create_dir(&app2_path).unwrap();

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

        // 在第二个目录也创建 app1（同名）
        let app1_path2 = temp_dir2.path().join("app1");
        std::fs::create_dir(&app1_path2).unwrap();
        std::fs::write(app1_path2.join("Dockerfile"), "FROM nginx").unwrap();

        let scan_dirs = vec![
            temp_dir1.path().to_str().unwrap().to_string(),
            temp_dir2.path().to_str().unwrap().to_string(),
        ];
        let result = discover_micro_apps(&scan_dirs);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("重复的微应用名称"));
    }

    #[test]
    fn test_get_micro_app_names() {
        let apps = vec![
            MicroApp {
                name: "app1".to_string(),
                path: PathBuf::from("/app1"),
                env_file: PathBuf::from("/app1/.env"),
                dockerfile: PathBuf::from("/app1/Dockerfile"),
                setup_script: None,
                clean_script: None,
            },
            MicroApp {
                name: "app2".to_string(),
                path: PathBuf::from("/app2"),
                env_file: PathBuf::from("/app2/.env"),
                dockerfile: PathBuf::from("/app2/Dockerfile"),
                setup_script: None,
                clean_script: None,
            },
        ];

        let names = get_micro_app_names(&apps);
        assert_eq!(names, vec!["app1".to_string(), "app2".to_string()]);
    }
}
