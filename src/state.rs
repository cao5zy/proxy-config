//! 状态管理模块
//!
//! 负责记录和管理微应用的状态

use crate::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// 应用状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    /// 应用名称
    pub app_name: String,

    /// 目录hash值
    pub hash: String,

    /// 最后构建时间
    #[serde(with = "chrono::serde::ts_seconds")]
    pub last_built: DateTime<Utc>,

    /// 镜像是否存在
    pub image_exists: bool,
}

/// 状态管理器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateManager {
    /// 状态文件路径
    state_file_path: PathBuf,

    /// 应用状态映射
    states: HashMap<String, AppState>,
}

impl StateManager {
    /// 创建新的状态管理器
    ///
    /// # 参数
    /// - `state_file_path`: 状态文件路径
    ///
    /// # 返回
    /// 返回状态管理器实例
    pub fn new<P: AsRef<Path>>(state_file_path: P) -> Self {
        let state_file_path = state_file_path.as_ref().to_path_buf();
        log::info!("创建状态管理器，状态文件: {:?}", state_file_path);

        StateManager {
            state_file_path,
            states: HashMap::new(),
        }
    }

    /// 从文件加载状态
    ///
    /// # 返回
    /// 返回加载后的状态管理器
    pub fn load(&mut self) -> Result<()> {
        log::info!("正在加载状态文件: {:?}", self.state_file_path);

        if !self.state_file_path.exists() {
            log::warn!("状态文件不存在，将创建新的状态文件");
            self.states = HashMap::new();
            return Ok(());
        }

        let content = fs::read_to_string(&self.state_file_path).map_err(|e| {
            log::error!("读取状态文件失败: {:?}, 错误: {}", self.state_file_path, e);
            Error::State(format!(
                "读取状态文件 {:?} 失败: {}",
                self.state_file_path, e
            ))
        })?;

        self.states = serde_yaml::from_str(&content).map_err(|e| {
            log::error!("解析状态文件失败: {:?}, 错误: {}", self.state_file_path, e);
            Error::State(format!(
                "解析状态文件 {:?} 失败: {}",
                self.state_file_path, e
            ))
        })?;

        log::info!("状态文件加载成功，共 {} 个应用状态", self.states.len());
        Ok(())
    }

    /// 保存状态到文件
    ///
    /// # 返回
    /// 返回保存结果
    pub fn save(&self) -> Result<()> {
        log::info!("正在保存状态文件: {:?}", self.state_file_path);

        let content = serde_yaml::to_string(&self.states).map_err(|e| {
            log::error!("序列化状态失败: {}", e);
            Error::State(format!("序列化状态失败: {}", e))
        })?;

        fs::write(&self.state_file_path, content).map_err(|e| {
            log::error!("写入状态文件失败: {:?}, 错误: {}", self.state_file_path, e);
            Error::State(format!(
                "写入状态文件 {:?} 失败: {}",
                self.state_file_path, e
            ))
        })?;

        log::info!("状态文件保存成功");
        Ok(())
    }

    /// 获取应用状态
    ///
    /// # 参数
    /// - `app_name`: 应用名称
    ///
    /// # 返回
    /// 返回应用状态的引用，如果不存在则返回 None
    pub fn get_state(&self, app_name: &str) -> Option<&AppState> {
        self.states.get(app_name)
    }

    /// 更新应用状态
    ///
    /// # 参数
    /// - `app_name`: 应用名称
    /// - `hash`: 目录hash值
    /// - `image_exists`: 镜像是否存在
    pub fn update_state(&mut self, app_name: &str, hash: String, image_exists: bool) {
        log::debug!("更新应用状态: {}, hash: {}", app_name, hash);

        let state = AppState {
            app_name: app_name.to_string(),
            hash,
            last_built: Utc::now(),
            image_exists,
        };

        self.states.insert(app_name.to_string(), state);
    }

    /// 删除应用状态
    ///
    /// # 参数
    /// - `app_name`: 应用名称
    pub fn remove_state(&mut self, app_name: &str) {
        log::debug!("删除应用状态: {}", app_name);
        self.states.remove(app_name);
    }

    /// 检查应用是否需要重新构建
    ///
    /// # 参数
    /// - `app_name`: 应用名称
    /// - `current_hash`: 当前目录hash值
    ///
    /// # 返回
    /// 如果需要重新构建，返回 true，否则返回 false
    pub fn needs_rebuild(&self, app_name: &str, current_hash: &str) -> bool {
        if let Some(state) = self.get_state(app_name) {
            let needs_rebuild = state.hash != current_hash;
            log::debug!(
                "应用 {} 需要重新构建: {} (旧hash: {}, 新hash: {})",
                app_name,
                needs_rebuild,
                state.hash,
                current_hash
            );
            needs_rebuild
        } else {
            log::debug!("应用 {} 没有状态记录，需要构建", app_name);
            true
        }
    }

    /// 获取所有应用状态
    ///
    /// # 返回
    /// 返回所有应用状态的迭代器
    pub fn get_all_states(&self) -> impl Iterator<Item = &AppState> {
        self.states.values()
    }
}

/// 计算目录的hash值
///
/// # 参数
/// - `path`: 目录路径
///
/// # 返回
/// 返回目录的hash值
pub fn calculate_directory_hash<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path.as_ref();
    log::debug!("正在计算目录hash: {:?}", path);

    let mut hasher = Sha256::new();

    // 遍历目录
    for entry in walkdir::WalkDir::new(path)
        .follow_links(true)
        .sort_by_file_name()
    {
        let entry = entry.map_err(|e| {
            log::error!("遍历目录失败: {}", e);
            Error::State(format!("遍历目录失败: {}", e))
        })?;

        // 跳过.git目录
        if entry.file_name() == ".git" {
            continue;
        }

        // 更新hash
        let path_str = entry.path().to_string_lossy().as_bytes().to_vec();
        hasher.update(&path_str);

        // 如果是文件，更新文件内容hash
        if entry.file_type().is_file() {
            let content = fs::read(entry.path()).map_err(|e| {
                log::error!("读取文件失败: {:?}, 错误: {}", entry.path(), e);
                Error::State(format!("读取文件 {:?} 失败: {}", entry.path(), e))
            })?;
            hasher.update(&content);
        }
    }

    let hash = format!("{:x}", hasher.finalize());
    log::debug!("目录hash计算完成: {}", hash);
    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_state_manager_new() {
        let temp_file = NamedTempFile::new().unwrap();
        let manager = StateManager::new(temp_file.path());
        assert_eq!(manager.state_file_path, temp_file.path());
    }

    #[test]
    fn test_state_manager_save_and_load() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut manager = StateManager::new(temp_file.path());

        manager.update_state("test-app", "hash123".to_string(), true);
        manager.save().unwrap();

        let mut manager2 = StateManager::new(temp_file.path());
        manager2.load().unwrap();

        let state = manager2.get_state("test-app");
        assert!(state.is_some());
        assert_eq!(state.unwrap().hash, "hash123");
    }

    #[test]
    fn test_state_manager_needs_rebuild() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut manager = StateManager::new(temp_file.path());

        // 没有状态记录，需要构建
        assert!(manager.needs_rebuild("test-app", "hash123"));

        // 更新状态
        manager.update_state("test-app", "hash123".to_string(), true);

        // hash相同，不需要重新构建
        assert!(!manager.needs_rebuild("test-app", "hash123"));

        // hash不同，需要重新构建
        assert!(manager.needs_rebuild("test-app", "hash456"));
    }

    #[test]
    fn test_calculate_directory_hash() {
        let temp_dir = TempDir::new().unwrap();

        // 创建一些文件
        fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
        fs::write(temp_dir.path().join("file2.txt"), "content2").unwrap();

        let hash1 = calculate_directory_hash(temp_dir.path()).unwrap();

        // 修改文件内容
        fs::write(temp_dir.path().join("file1.txt"), "content1_modified").unwrap();

        let hash2 = calculate_directory_hash(temp_dir.path()).unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_calculate_directory_hash_consistent() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("file.txt"), "content").unwrap();

        let hash1 = calculate_directory_hash(temp_dir.path()).unwrap();
        let hash2 = calculate_directory_hash(temp_dir.path()).unwrap();

        assert_eq!(hash1, hash2);
    }
}
