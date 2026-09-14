//! 镜像构建模块
//!
//! 负责构建微应用的Docker镜像

use crate::{Error, Result};
use std::path::Path;
use std::process::Command;

/// 从 `docker load` 的标准输出提取带标签的镜像引用。
fn loaded_image_references(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| line.trim().strip_prefix("Loaded image: "))
        .map(ToString::to_string)
        .collect()
}

/// 将 `docker save` 生成的镜像归档导入本机 Docker 镜像库，返回归档内保留的标签。
pub fn load_image_archive<P: AsRef<Path>>(archive_path: P) -> Result<Vec<String>> {
    let archive_path = archive_path.as_ref();
    if !archive_path.is_file() {
        return Err(Error::Build(format!(
            "镜像归档不存在或不是文件: {:?}",
            archive_path
        )));
    }

    let output = Command::new("docker")
        .arg("load")
        .arg("-i")
        .arg(archive_path)
        .output()
        .map_err(|e| Error::Build(format!("执行 docker load 失败: {}", e)))?;
    if !output.status.success() {
        return Err(Error::Build(format!(
            "导入镜像归档 {:?} 失败: {}",
            archive_path,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    let references = loaded_image_references(&String::from_utf8_lossy(&output.stdout));
    if references.is_empty() {
        return Err(Error::Build(format!(
            "镜像归档 {:?} 未包含带标签的镜像引用",
            archive_path
        )));
    }
    Ok(references)
}

/// 构建Docker镜像
///
/// # 参数
/// - `image_name`: 镜像名称
/// - `dockerfile_path`: Dockerfile路径
/// - `build_context`: 构建上下文路径
/// - `env_file`: 环境变量文件路径（可选）
/// - `no_cache`: 是否禁用构建缓存
///
/// # 返回
/// 返回构建结果
pub fn build_image<P: AsRef<Path>>(
    image_name: &str,
    dockerfile_path: P,
    build_context: P,
    env_file: Option<P>,
    no_cache: bool,
) -> Result<()> {
    let dockerfile_path = dockerfile_path.as_ref();
    let build_context = build_context.as_ref();

    log::info!("开始构建Docker镜像: {}", image_name);
    log::debug!("Dockerfile路径: {:?}", dockerfile_path);
    log::debug!("构建上下文: {:?}", build_context);
    log::debug!("禁用缓存: {}", no_cache);

    // 检查Dockerfile是否存在
    if !dockerfile_path.exists() {
        log::error!("Dockerfile不存在: {:?}", dockerfile_path);
        return Err(Error::Build(format!(
            "Dockerfile不存在: {:?}",
            dockerfile_path
        )));
    }

    // 检查构建上下文是否存在
    if !build_context.exists() {
        log::error!("构建上下文不存在: {:?}", build_context);
        return Err(Error::Build(format!(
            "构建上下文不存在: {:?}",
            build_context
        )));
    }

    // 构建docker build命令
    let mut cmd = Command::new("docker");
    cmd.arg("build")
        .arg("-t")
        .arg(image_name)
        .arg("-f")
        .arg(dockerfile_path);

    // 如果需要禁用缓存，添加--no-cache参数
    if no_cache {
        cmd.arg("--no-cache");
        log::info!("已启用 --no-cache 参数，将不使用构建缓存");
    }

    // 如果有环境变量文件，添加--build-arg参数
    if let Some(env_file) = env_file {
        let env_file = env_file.as_ref();
        if env_file.exists() {
            log::debug!("使用环境变量文件: {:?}", env_file);
            // 读取环境变量文件
            let env_content = std::fs::read_to_string(env_file).map_err(|e| {
                log::error!("读取环境变量文件失败: {:?}, 错误: {}", env_file, e);
                Error::Build(format!("读取环境变量文件失败: {}", e))
            })?;

            // 解析环境变量并添加为构建参数
            for line in env_content.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') {
                    if let Some((key, value)) = line.split_once('=') {
                        cmd.arg("--build-arg").arg(format!("{}={}", key, value));
                        log::debug!("添加构建参数: {}={}", key, value);
                    }
                }
            }
        }
    }

    cmd.arg(build_context);

    log::debug!("执行命令: {:?}", cmd);

    // 执行构建命令
    let output = cmd.output().map_err(|e| {
        log::error!("执行docker build命令失败: {}", e);
        Error::Build(format!("执行docker build命令失败: {}", e))
    })?;

    // 检查退出状态
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::error!("Docker镜像构建失败: {}", image_name);
        log::error!("错误输出:\n{}", stderr);
        return Err(Error::Build(format!(
            "Docker镜像构建失败: {}\n{}",
            image_name, stderr
        )));
    }

    // 输出构建日志
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() {
        log::info!("构建输出:\n{}", stdout);
    }

    log::info!("Docker镜像构建成功: {}", image_name);
    Ok(())
}

/// 删除Docker镜像
///
/// # 参数
/// - `image_name`: 镜像名称
///
/// # 返回
/// 返回删除结果
pub fn remove_image(image_name: &str) -> Result<()> {
    log::info!("删除Docker镜像: {}", image_name);

    let output = Command::new("docker")
        .arg("rmi")
        .arg("-f")
        .arg(image_name)
        .output()
        .map_err(|e| {
            log::error!("执行docker rmi命令失败: {}", e);
            Error::Build(format!("执行docker rmi命令失败: {}", e))
        })?;

    // 检查退出状态
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::warn!("删除Docker镜像失败: {}", image_name);
        log::warn!("错误输出: {}", stderr);
        // 不返回错误，因为镜像可能不存在
        return Ok(());
    }

    log::info!("Docker镜像删除成功: {}", image_name);
    Ok(())
}

/// 检查Docker镜像是否存在
///
/// # 参数
/// - `image_name`: 镜像名称
///
/// # 返回
/// 如果镜像存在，返回 true，否则返回 false
pub fn image_exists(image_name: &str) -> Result<bool> {
    log::debug!("检查Docker镜像是否存在: {}", image_name);

    let output = Command::new("docker")
        .arg("images")
        .arg("-q")
        .arg(image_name)
        .output()
        .map_err(|e| {
            log::error!("执行docker images命令失败: {}", e);
            Error::Build(format!("执行docker images命令失败: {}", e))
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let exists = !stdout.trim().is_empty();

    log::debug!("Docker镜像 {} 存在: {}", image_name, exists);
    Ok(exists)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_loaded_image_references_extracts_tagged_images_only() {
        let output = "Loaded image: my-api:sha-a1b2c3\nLoaded image ID: sha256:abc\nLoaded image: helper:1.0\n";
        assert_eq!(
            loaded_image_references(output),
            vec!["my-api:sha-a1b2c3", "helper:1.0"]
        );
    }

    #[test]
    fn test_build_image_no_dockerfile() {
        let temp_dir = TempDir::new().unwrap();
        let dockerfile_path = temp_dir.path().join("Dockerfile");

        let result = build_image(
            "test-image",
            dockerfile_path.as_path(),
            temp_dir.path(),
            None::<&Path>,
            false,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_build_image_no_context() {
        let temp_dir = TempDir::new().unwrap();
        let dockerfile_path = temp_dir.path().join("Dockerfile");
        let context_path = temp_dir.path().join("nonexistent");

        let result = build_image(
            "test-image",
            dockerfile_path.as_path(),
            context_path.as_path(),
            None::<&Path>,
            false,
        );
        assert!(result.is_err());
    }
}
