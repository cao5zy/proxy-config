//! 脚本执行模块
//!
//! 负责执行微应用的setup.sh和clean.sh脚本

use crate::{Error, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// 执行脚本
///
/// # 参数
/// - `script_path`: 脚本路径
/// - `working_dir`: 工作目录
///
/// # 返回
/// 返回执行结果
pub fn execute_script<P: AsRef<Path> + std::fmt::Debug, W: AsRef<Path> + std::fmt::Debug>(
    script_path: P,
    working_dir: W,
) -> Result<()> {
    let script_path = script_path.as_ref();
    let working_dir = working_dir.as_ref();

    log::info!("正在执行脚本: {:?}", script_path);
    log::debug!("工作目录: {:?}", working_dir);

    // 检查脚本是否存在
    if !script_path.exists() {
        log::error!("脚本不存在: {:?}", script_path);
        return Err(Error::Script(format!("脚本不存在: {:?}", script_path)));
    }

    // 执行脚本
    let output = Command::new("bash")
        .arg(script_path)
        .current_dir(working_dir)
        .output()
        .map_err(|e| {
            log::error!("执行脚本失败: {:?}, 错误: {}", script_path, e);
            Error::Script(format!("执行脚本 {:?} 失败: {}", script_path, e))
        })?;

    // 检查退出状态
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::error!("脚本执行失败: {:?}", script_path);
        log::error!("错误输出: {}", stderr);
        return Err(Error::Script(format!(
            "脚本 {:?} 执行失败: {}",
            script_path, stderr
        )));
    }

    // 输出标准输出
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() {
        log::info!("脚本输出:\n{}", stdout);
    }

    log::info!("脚本执行成功: {:?}", script_path);
    Ok(())
}

/// 执行setup脚本
///
/// # 参数
/// - `script_path`: setup.sh脚本路径
/// - `working_dir`: 工作目录
///
/// # 返回
/// 返回执行结果
pub fn execute_setup_script<P: AsRef<Path> + std::fmt::Debug, W: AsRef<Path> + std::fmt::Debug>(
    script_path: P,
    working_dir: W,
) -> Result<()> {
    log::info!("执行预构建脚本: {:?}", script_path);
    execute_script(script_path, working_dir)
}

/// 执行clean脚本
///
/// # 参数
/// - `script_path`: clean.sh脚本路径
/// - `working_dir`: 工作目录
///
/// # 返回
/// 返回执行结果
pub fn execute_clean_script<P: AsRef<Path> + std::fmt::Debug, W: AsRef<Path> + std::fmt::Debug>(
    script_path: P,
    working_dir: W,
) -> Result<()> {
    log::info!("执行清理脚本: {:?}", script_path);
    execute_script(script_path, working_dir)
}

/// 检查脚本是否存在
///
/// # 参数
/// - `script_path`: 脚本路径
///
/// # 返回
/// 如果脚本存在，返回 true，否则返回 false
pub fn script_exists<P: AsRef<Path>>(script_path: P) -> bool {
    let script_path = script_path.as_ref();
    script_path.exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_script_exists() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("test.sh");

        assert!(!script_exists(&script_path));

        std::fs::write(&script_path, "#!/bin/bash\necho test").unwrap();
        assert!(script_exists(&script_path));
    }

    #[test]
    fn test_execute_script_success() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("test.sh");

        std::fs::write(&script_path, "#!/bin/bash\necho 'Hello, World!'").unwrap();

        let result = execute_script(&script_path, temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_script_failure() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("test.sh");

        std::fs::write(&script_path, "#!/bin/bash\nexit 1").unwrap();

        let result = execute_script(&script_path, temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_script_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("nonexistent.sh");

        let result = execute_script(&script_path, temp_dir.path());
        assert!(result.is_err());
    }
}
