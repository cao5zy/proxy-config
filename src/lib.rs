//! micro_proxy - 微应用管理工具
//!
//! 这个工具用于管理微应用的Docker镜像构建、容器管理、Nginx反向代理配置等。

pub mod builder;
pub mod cli;
pub mod compose;
pub mod config;
pub mod container;
pub mod discovery;
pub mod dockerfile;
pub mod network;
pub mod nginx;
pub mod script;
pub mod state;

pub use error::{Error, Result};

mod error;

/// 版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
