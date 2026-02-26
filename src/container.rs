//! 容器管理模块
//!
//! 负责管理容器的生命周期

use crate::{Error, Result};
use std::process::Command;

/// 创建容器
///
/// # 参数
/// - `container_name`: 容器名称
/// - `image_name`: 镜像名称
/// - `network_name`: 网络名称
/// - `port_mappings`: 端口映射列表（可选）
/// - `env_vars`: 环境变量列表（可选）
///
/// # 返回
/// 返回创建结果
pub fn create_container(
    container_name: &str,
    image_name: &str,
    network_name: &str,
    port_mappings: Option<Vec<(u16, u16)>>,
    env_vars: Option<Vec<String>>,
) -> Result<()> {
    log::info!("创建容器: {}", container_name);
    log::debug!("镜像: {}, 网络: {}", image_name, network_name);

    // 构建docker create命令
    let mut cmd = Command::new("docker");
    cmd.arg("create")
        .arg("--name")
        .arg(container_name)
        .arg("--network")
        .arg(network_name);

    // 添加端口映射
    if let Some(mappings) = port_mappings {
        for (host_port, container_port) in mappings {
            cmd.arg("-p")
                .arg(format!("{}:{}", host_port, container_port));
            log::debug!("端口映射: {} -> {}", host_port, container_port);
        }
    }

    // 添加环境变量
    if let Some(vars) = env_vars {
        for var in vars {
            cmd.arg("-e").arg(&var);
            log::debug!("环境变量: {}", var);
        }
    }

    cmd.arg(image_name);

    log::debug!("执行命令: {:?}", cmd);

    // 执行命令
    let output = cmd.output().map_err(|e| {
        log::error!("执行docker create命令失败: {}", e);
        Error::Container(format!("执行docker create命令失败: {}", e))
    })?;

    // 检查退出状态
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::error!("创建容器失败: {}", container_name);
        log::error!("错误输出:\n{}", stderr);
        return Err(Error::Container(format!(
            "创建容器 {} 失败:\n{}",
            container_name, stderr
        )));
    }

    log::info!("容器创建成功: {}", container_name);
    Ok(())
}

/// 启动容器
///
/// # 参数
/// - `container_name`: 容器名称
///
/// # 返回
/// 返回启动结果
pub fn start_container(container_name: &str) -> Result<()> {
    log::info!("启动容器: {}", container_name);

    let output = Command::new("docker")
        .arg("start")
        .arg(container_name)
        .output()
        .map_err(|e| {
            log::error!("执行docker start命令失败: {}", e);
            Error::Container(format!("执行docker start命令失败: {}", e))
        })?;

    // 检查退出状态
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::error!("启动容器失败: {}", container_name);
        log::error!("错误输出:\n{}", stderr);
        return Err(Error::Container(format!(
            "启动容器 {} 失败:\n{}",
            container_name, stderr
        )));
    }

    log::info!("容器启动成功: {}", container_name);
    Ok(())
}

/// 停止容器
///
/// # 参数
/// - `container_name`: 容器名称
///
/// # 返回
/// 返回停止结果
pub fn stop_container(container_name: &str) -> Result<()> {
    log::info!("停止容器: {}", container_name);

    let output = Command::new("docker")
        .arg("stop")
        .arg(container_name)
        .output()
        .map_err(|e| {
            log::error!("执行docker stop命令失败: {}", e);
            Error::Container(format!("执行docker stop命令失败: {}", e))
        })?;

    // 检查退出状态
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::warn!("停止容器失败: {}", container_name);
        log::warn!("错误输出: {}", stderr);
        // 不返回错误，因为容器可能已经停止
        return Ok(());
    }

    log::info!("容器停止成功: {}", container_name);
    Ok(())
}

/// 删除容器
///
/// # 参数
/// - `container_name`: 容器名称
///
/// # 返回
/// 返回删除结果
pub fn remove_container(container_name: &str) -> Result<()> {
    log::info!("删除容器: {}", container_name);

    let output = Command::new("docker")
        .arg("rm")
        .arg("-f")
        .arg(container_name)
        .output()
        .map_err(|e| {
            log::error!("执行docker rm命令失败: {}", e);
            Error::Container(format!("执行docker rm命令失败: {}", e))
        })?;

    // 检查退出状态
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::warn!("删除容器失败: {}", container_name);
        log::warn!("错误输出: {}", stderr);
        // 不返回错误，因为容器可能不存在
        return Ok(());
    }

    log::info!("容器删除成功: {}", container_name);
    Ok(())
}

/// 查询容器状态
///
/// # 参数
/// - `container_name`: 容器名称
///
/// # 返回
/// 返回容器状态，如果容器不存在则返回 None
pub fn get_container_status(container_name: &str) -> Result<Option<String>> {
    log::debug!("查询容器状态: {}", container_name);

    let output = Command::new("docker")
        .arg("ps")
        .arg("-a")
        .arg("--filter")
        .arg(format!("name={}", container_name))
        .arg("--format")
        .arg("{{.Status}}")
        .output()
        .map_err(|e| {
            log::error!("执行docker ps命令失败: {}", e);
            Error::Container(format!("执行docker ps命令失败: {}", e))
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let status = stdout.trim();

    if status.is_empty() {
        log::debug!("容器不存在: {}", container_name);
        Ok(None)
    } else {
        log::debug!("容器 {} 状态: {}", container_name, status);
        Ok(Some(status.to_string()))
    }
}

/// 检查容器是否运行中
///
/// # 参数
/// - `container_name`: 容器名称
///
/// # 返回
/// 如果容器正在运行，返回 true，否则返回 false
pub fn is_container_running(container_name: &str) -> Result<bool> {
    log::debug!("检查容器是否运行: {}", container_name);

    let output = Command::new("docker")
        .arg("ps")
        .arg("--filter")
        .arg(format!("name={}", container_name))
        .arg("--filter")
        .arg("status=running")
        .arg("--format")
        .arg("{{.Names}}")
        .output()
        .map_err(|e| {
            log::error!("执行docker ps命令失败: {}", e);
            Error::Container(format!("执行docker ps命令失败: {}", e))
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let running = stdout.trim().contains(container_name);

    log::debug!("容器 {} 运行中: {}", container_name, running);
    Ok(running)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_container_status_nonexistent() {
        // 这个测试需要Docker环境，在实际环境中运行
        // 这里只是展示测试结构
        let result = get_container_status("nonexistent-container-test");
        // 在CI环境中可能没有Docker，所以不断言结果
        let _ = result;
    }
}
