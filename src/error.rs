//! 错误类型定义

use thiserror::Error;

/// proxy-config 的错误类型
#[derive(Error, Debug)]
pub enum Error {
    #[error("配置错误: {0}")]
    Config(String),

    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML解析错误: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Docker错误: {0}")]
    Docker(String),

    #[error("脚本执行错误: {0}")]
    Script(String),

    #[error("网络错误: {0}")]
    Network(String),

    #[error("发现错误: {0}")]
    Discovery(String),

    #[error("构建错误: {0}")]
    Build(String),

    #[error("容器错误: {0}")]
    Container(String),

    #[error("状态错误: {0}")]
    State(String),

    #[error("Dockerfile解析错误: {0}")]
    Dockerfile(String),

    #[error("Nginx配置错误: {0}")]
    Nginx(String),

    #[error("Compose配置错误: {0}")]
    Compose(String),
}

/// Result 类型别名
pub type Result<T> = std::result::Result<T, Error>;
