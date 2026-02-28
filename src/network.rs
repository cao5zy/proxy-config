//! 网络管理模块
//!
//! 负责管理Docker网络和网络地址

use crate::{config::AppType, Error, Result};
use std::process::Command;

/// 创建Docker网络
///
/// # 参数
/// - `network_name`: 网络名称
///
/// # 返回
/// 返回创建结果
pub fn create_network(network_name: &str) -> Result<()> {
    log::info!("创建Docker网络: {}", network_name);

    // 检查网络是否已存在
    if network_exists(network_name)? {
        log::info!("网络已存在: {}", network_name);
        return Ok(());
    }

    let output = Command::new("docker")
        .arg("network")
        .arg("create")
        .arg(network_name)
        .output()
        .map_err(|e| {
            log::error!("执行docker network create命令失败: {}", e);
            Error::Network(format!("执行docker network create命令失败: {}", e))
        })?;

    // 检查退出状态
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::error!("创建网络失败: {}", network_name);
        log::error!("错误输出:\n{}", stderr);
        return Err(Error::Network(format!(
            "创建网络 {} 失败:\n{}",
            network_name, stderr
        )));
    }

    log::info!("Docker网络创建成功: {}", network_name);
    Ok(())
}

/// 删除Docker网络
///
/// # 参数
/// - `network_name`: 网络名称
///
/// # 返回
/// 返回删除结果
pub fn remove_network(network_name: &str) -> Result<()> {
    log::info!("删除Docker网络: {}", network_name);

    // 检查网络是否存在
    if !network_exists(network_name)? {
        log::info!("网络不存在: {}", network_name);
        return Ok(());
    }

    let output = Command::new("docker")
        .arg("network")
        .arg("rm")
        .arg(network_name)
        .output()
        .map_err(|e| {
            log::error!("执行docker network rm命令失败: {}", e);
            Error::Network(format!("执行docker network rm命令失败: {}", e))
        })?;

    // 检查退出状态
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::warn!("删除网络失败: {}", network_name);
        log::warn!("错误输出: {}", stderr);
        // 不返回错误，因为网络可能不存在
        return Ok(());
    }

    log::info!("Docker网络删除成功: {}", network_name);
    Ok(())
}

/// 检查网络是否存在
///
/// # 参数
/// - `network_name`: 网络名称
///
/// # 返回
/// 如果网络存在，返回 true，否则返回 false
pub fn network_exists(network_name: &str) -> Result<bool> {
    log::debug!("检查网络是否存在: {}", network_name);

    let output = Command::new("docker")
        .arg("network")
        .arg("ls")
        .arg("--filter")
        .arg(format!("name={}", network_name))
        .arg("--format")
        .arg("{{.Name}}")
        .output()
        .map_err(|e| {
            log::error!("执行docker network ls命令失败: {}", e);
            Error::Network(format!("执行docker network ls命令失败: {}", e))
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Docker 的 --filter name= 是模糊匹配，所以我们需要在结果中做精确匹配
    // 将输出按行分割，然后检查是否有完全匹配的网络名称
    let exists = stdout.lines().any(|line| line.trim() == network_name);

    log::debug!("网络 {} 存在: {}", network_name, exists);
    Ok(exists)
}

/// 网络地址信息
#[derive(Debug, Clone)]
pub struct NetworkAddressInfo {
    /// 微应用名称
    pub app_name: String,
    /// 容器名称
    pub container_name: String,
    /// 网络地址（即容器名称，在Docker网络中作为主机名）
    pub network_address: String,
    /// 容器端口
    pub container_port: u16,
    /// 通过nginx访问的URL列表（Internal 类型为空）
    pub accessible_urls: Vec<String>,
}

impl NetworkAddressInfo {
    /// 创建新的网络地址信息
    ///
    /// # 参数
    /// - `app_name`: 微应用名称
    /// - `container_name`: 容器名称
    /// - `container_port`: 容器端口
    /// - `routes`: 路由列表
    /// - `nginx_host_port`: nginx主机端口
    /// - `app_type`: 应用类型
    ///
    /// # 返回
    /// 返回网络地址信息对象
    pub fn new(
        app_name: String,
        container_name: String,
        container_port: u16,
        routes: &[String],
        nginx_host_port: u16,
        app_type: &AppType,
    ) -> Self {
        let network_address = app_name.clone();

        // Internal 类型没有可访问的 URL
        let accessible_urls = if *app_type == AppType::Internal {
            log::debug!("Internal 应用 '{}' 没有可访问的 URL", app_name);
            Vec::new()
        } else {
            routes
                .iter()
                .map(|route| {
                    if route == "/" {
                        format!("http://localhost:{}", nginx_host_port)
                    } else {
                        format!("http://localhost:{}{}", nginx_host_port, route)
                    }
                })
                .collect()
        };

        NetworkAddressInfo {
            app_name,
            container_name,
            network_address,
            container_port,
            accessible_urls,
        }
    }

    /// 格式化为文本
    ///
    /// # 返回
    /// 返回格式化后的文本
    pub fn format(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("微应用名称: {}\n", self.app_name));
        output.push_str(&format!("容器名称: {}\n", self.container_name));
        output.push_str(&format!("网络地址: {}\n", self.network_address));
        output.push_str(&format!("容器端口: {}\n", self.container_port));

        if self.accessible_urls.is_empty() {
            output.push_str("访问地址: 无（内部服务）\n");
        } else {
            output.push_str("访问地址:\n");
            for url in &self.accessible_urls {
                output.push_str(&format!("  - {}\n", url));
            }
        }

        output
    }
}

/// 生成网络地址列表文件
///
/// # 参数
/// - `infos`: 网络地址信息列表
/// - `network_name`: 网络名称
/// - `nginx_host_port`: nginx主机端口
/// - `output_path`: 输出文件路径
///
/// # 返回
/// 返回生成结果
pub fn generate_network_list(
    infos: &[NetworkAddressInfo],
    network_name: &str,
    nginx_host_port: u16,
    output_path: &str,
) -> Result<()> {
    log::info!("生成网络地址列表文件: {}", output_path);

    let mut content = String::new();

    // 添加文件头
    content.push_str("# 微应用网络地址列表\n");
    content.push_str(&format!(
        "# 生成时间: {}\n",
        chrono::Utc::now().to_rfc3339()
    ));
    content.push_str(&format!("# 网络名称: {}\n", network_name));
    content.push_str(&format!(
        "# Nginx统一入口: http://localhost:{}\n",
        nginx_host_port
    ));
    content.push_str("\n");

    // 添加每个应用的信息
    for info in infos {
        content.push_str(&info.format());
        content.push_str("\n");
    }

    // 添加微应用间通信示例
    content.push_str("# 微应用间通信示例\n");
    if infos.len() > 1 {
        for i in 0..infos.len() {
            for j in 0..infos.len() {
                if i != j {
                    content.push_str(&format!(
                        "# {} 可以通过 http://{}:{} 访问 {}\n",
                        infos[i].app_name,
                        infos[j].network_address,
                        infos[j].container_port,
                        infos[j].app_name
                    ));
                }
            }
        }
    }

    // 写入文件
    std::fs::write(output_path, content).map_err(|e| {
        log::error!("写入网络地址列表文件失败: {}, 错误: {}", output_path, e);
        Error::Network(format!("写入网络地址列表文件失败: {}", e))
    })?;

    log::info!("网络地址列表文件生成成功: {}", output_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_address_info_new_static() {
        let info = NetworkAddressInfo::new(
            "test-app".to_string(),
            "test-container".to_string(),
            80,
            &["/".to_string(), "/api".to_string()],
            8080,
            &AppType::Static,
        );

        assert_eq!(info.app_name, "test-app");
        assert_eq!(info.container_name, "test-container");
        assert_eq!(info.network_address, "test-app");
        assert_eq!(info.container_port, 80);
        assert_eq!(info.accessible_urls.len(), 2);
        assert_eq!(info.accessible_urls[0], "http://localhost:8080");
        assert_eq!(info.accessible_urls[1], "http://localhost:8080/api");
    }

    #[test]
    fn test_network_address_info_new_internal() {
        let info = NetworkAddressInfo::new(
            "redis".to_string(),
            "redis-container".to_string(),
            6379,
            &[],
            8080,
            &AppType::Internal,
        );

        assert_eq!(info.app_name, "redis");
        assert_eq!(info.container_name, "redis-container");
        assert_eq!(info.network_address, "redis");
        assert_eq!(info.container_port, 6379);
        assert_eq!(info.accessible_urls.len(), 0);
    }

    #[test]
    fn test_network_address_info_format_static() {
        let info = NetworkAddressInfo::new(
            "test-app".to_string(),
            "test-container".to_string(),
            80,
            &["/".to_string()],
            8080,
            &AppType::Static,
        );

        let formatted = info.format();
        assert!(formatted.contains("微应用名称: test-app"));
        assert!(formatted.contains("容器名称: test-container"));
        assert!(formatted.contains("网络地址: test-app"));
        assert!(formatted.contains("容器端口: 80"));
        assert!(formatted.contains("访问地址:"));
        assert!(formatted.contains("http://localhost:8080"));
    }

    #[test]
    fn test_network_address_info_format_internal() {
        let info = NetworkAddressInfo::new(
            "redis".to_string(),
            "redis-container".to_string(),
            6379,
            &[],
            8080,
            &AppType::Internal,
        );

        let formatted = info.format();
        assert!(formatted.contains("微应用名称: redis"));
        assert!(formatted.contains("容器名称: redis-container"));
        assert!(formatted.contains("网络地址: redis"));
        assert!(formatted.contains("容器端口: 6379"));
        assert!(formatted.contains("访问地址: 无（内部服务）"));
    }

    #[test]
    fn test_generate_network_list() {
        let infos = vec![
            NetworkAddressInfo::new(
                "app1".to_string(),
                "container1".to_string(),
                80,
                &["/".to_string()],
                8080,
                &AppType::Static,
            ),
            NetworkAddressInfo::new(
                "redis".to_string(),
                "redis-container".to_string(),
                6379,
                &[],
                8080,
                &AppType::Internal,
            ),
        ];

        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let result = generate_network_list(
            &infos,
            "test-network",
            8080,
            temp_file.path().to_str().unwrap(),
        );

        assert!(result.is_ok());

        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("微应用网络地址列表"));
        assert!(content.contains("网络名称: test-network"));
        assert!(content.contains("微应用名称: app1"));
        assert!(content.contains("微应用名称: redis"));
        assert!(content.contains("微应用间通信示例"));
        assert!(content.contains("访问地址: 无（内部服务）"));
    }
}
