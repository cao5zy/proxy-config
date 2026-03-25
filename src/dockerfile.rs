//! Dockerfile解析模块
//!
//! 负责解析Dockerfile，检查配置

use crate::{Error, Result};
use regex::Regex;
use std::fs;

/// Dockerfile解析结果
#[derive(Debug, Clone)]
pub struct DockerfileInfo {
    /// 暴露的端口列表
    pub exposed_ports: Vec<u16>,
}

/// 解析Dockerfile
///
/// # 参数
/// - `path`: Dockerfile路径
///
/// # 返回
/// 返回解析结果
pub fn parse_dockerfile<P: AsRef<std::path::Path>>(path: P) -> Result<DockerfileInfo> {
    let path = path.as_ref();
    log::info!("正在解析Dockerfile: {:?}", path);

    let content = fs::read_to_string(path).map_err(|e| {
        log::error!("读取Dockerfile失败: {:?}, 错误: {}", path, e);
        Error::Dockerfile(format!("读取Dockerfile {:?} 失败: {}", path, e))
    })?;

    let info = parse_dockerfile_content(&content)?;

    log::info!("Dockerfile解析完成，暴露端口: {:?}", info.exposed_ports);
    Ok(info)
}

/// 解析Dockerfile内容（纯函数）
///
/// # 参数
/// - `content`: Dockerfile内容
///
/// # 返回
/// 返回解析结果
fn parse_dockerfile_content(content: &str) -> Result<DockerfileInfo> {
    let mut exposed_ports = Vec::new();

    // 编译正则表达式匹配EXPOSE指令
    let expose_regex = Regex::new(r"(?i)^\s*EXPOSE\s+(\d+(?:\s+\d+)*)")
        .map_err(|e| Error::Dockerfile(format!("编译正则表达式失败: {}", e)))?;

    // 逐行解析
    for line in content.lines() {
        if let Some(captures) = expose_regex.captures(line) {
            if let Some(ports_str) = captures.get(1) {
                for port_str in ports_str.as_str().split_whitespace() {
                    if let Ok(port) = port_str.parse::<u16>() {
                        exposed_ports.push(port);
                        log::debug!("发现EXPOSE端口: {}", port);
                    }
                }
            }
        }
    }

    Ok(DockerfileInfo { exposed_ports })
}

/// 检查Dockerfile是否包含EXPOSE指令
///
/// # 参数
/// - `path`: Dockerfile路径
///
/// # 返回
/// 如果包含EXPOSE指令，返回 true，否则返回 false
pub fn has_expose_instruction<P: AsRef<std::path::Path>>(path: P) -> Result<bool> {
    let info = parse_dockerfile(path)?;
    Ok(!info.exposed_ports.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_dockerfile_content_with_expose() {
        let content = r#"
FROM nginx:alpine
EXPOSE 80 443
COPY . /usr/share/nginx/html
"#;

        let result = parse_dockerfile_content(content).unwrap();
        assert_eq!(result.exposed_ports, vec![80, 443]);
    }

    #[test]
    fn test_parse_dockerfile_content_without_expose() {
        let content = r#"
FROM nginx:alpine
COPY . /usr/share/nginx/html
"#;

        let result = parse_dockerfile_content(content).unwrap();
        assert!(result.exposed_ports.is_empty());
    }

    #[test]
    fn test_parse_dockerfile_content_case_insensitive() {
        let content = r#"
FROM nginx:alpine
expose 80
Expose 443
COPY . /usr/share/nginx/html
"#;

        let result = parse_dockerfile_content(content).unwrap();
        assert_eq!(result.exposed_ports, vec![80, 443]);
    }

    #[test]
    fn test_parse_dockerfile_content_with_whitespace() {
        let content = r#"
FROM nginx:alpine
  EXPOSE   80   443  
COPY . /usr/share/nginx/html
"#;

        let result = parse_dockerfile_content(content).unwrap();
        assert_eq!(result.exposed_ports, vec![80, 443]);
    }

    #[test]
    fn test_parse_dockerfile_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"
FROM nginx:alpine
EXPOSE 80
COPY . /usr/share/nginx/html
"#
        )
        .unwrap();

        let result = parse_dockerfile(temp_file.path()).unwrap();
        assert_eq!(result.exposed_ports, vec![80]);
    }

    #[test]
    fn test_has_expose_instruction_true() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"
FROM nginx:alpine
EXPOSE 80
"#
        )
        .unwrap();

        let result = has_expose_instruction(temp_file.path()).unwrap();
        assert!(result);
    }

    #[test]
    fn test_has_expose_instruction_false() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"
FROM nginx:alpine
"#
        )
        .unwrap();

        let result = has_expose_instruction(temp_file.path()).unwrap();
        assert!(!result);
    }
}
