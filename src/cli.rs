
//! 命令行接口模块
//!
//! 负责提供命令行交互接口

use crate::config::{AppType, ProxyConfig};
use crate::container;
use crate::discovery::{discover_micro_apps, get_micro_app_names, MicroApp};
use crate::dockerfile;
use crate::network::{generate_network_list, NetworkAddressInfo};
use crate::nginx;
use crate::script;
use crate::state::{calculate_directory_hash, StateManager};
use crate::{builder, compose, Error, Result};
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

/// micro_proxy - 微应用管理工具
#[derive(Parser, Debug)]
#[command(name = "micro_proxy")]
#[command(author = "Your Name <your.email@example.com>")]
#[command(version = crate::VERSION)]
#[command(about = "用于管理微应用的工具", long_about = None)]
struct Cli {
    /// 配置文件路径
    #[arg(short, long, default_value = "./proxy-config.yml")]
    config: PathBuf,

    /// 显示详细日志
    #[arg(short, long)]
    verbose: bool,

    /// 子命令
    #[command(subcommand)]
    command: Commands,
}

/// 子命令
#[derive(Subcommand, Debug)]
enum Commands {
    /// 启动所有微应用
    Start {
        /// 强制重新构建所有镜像
        #[arg(long)]
        force_rebuild: bool,
    },
    /// 停止所有微应用
    Stop,
    /// 清理所有微应用
    Clean {
        /// 强制清理，不询问确认
        #[arg(long)]
        force: bool,
        /// 同时清理Docker网络
        #[arg(long)]
        network: bool,
    },
    /// 查看状态
    Status,
    /// 查看网络地址
    Network {
        /// 指定输出文件路径（覆盖配置文件中的设置）
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

/// 运行CLI
///
/// # 参数
/// - `args`: 命令行参数
///
/// # 返回
/// 返回运行结果
pub fn run(args: &[String]) -> Result<()> {
    let cli = Cli::parse_from(args);

    // 初始化日志
    let log_level = if cli.verbose { "debug" } else { "info" };

    // 使用dumbo_log初始化日志系统，同时输出到文件和控制台
    // 日志文件名与Cargo.toml中的包名称保持一致
    let log_file_name = format!("{}.log", env!("CARGO_PKG_NAME"));
    let log_path = PathBuf::from(&log_file_name);
    if let Err(e) = dumbo_log::init_log_with_console(&log_path, None, true) {
        log::error!("初始化日志系统失败: {}", e);
        return Err(Error::Config(format!("初始化日志系统失败: {}", e)));
    }

    log::info!("micro_proxy v{} 启动", crate::VERSION);
    log::debug!("配置文件: {:?}", cli.config);

    // 读取配置
    let config = ProxyConfig::from_file(&cli.config)?;

    // 执行子命令
    match cli.command {
        Commands::Start { force_rebuild } => {
            execute_start(&config, force_rebuild)?;
        }
        Commands::Stop => {
            execute_stop(&config)?;
        }
        Commands::Clean { force, network } => {
            execute_clean(&config, force, network)?;
        }
        Commands::Status => {
            execute_status(&config)?;
        }
        Commands::Network { output } => {
            execute_network(&config, output)?;
        }
    }

    Ok(())
}

/// 执行docker-compose命令
///
/// 优先使用 `docker compose`（新版本），如果失败则尝试 `docker-compose`（旧版本）
///
/// # 参数
/// - `args`: 命令参数
///
/// # 返回
/// 返回命令执行结果
fn run_docker_compose(args: &[&str]) -> Result<()> {
    log::debug!("尝试执行 docker compose 命令: {:?}", args);

    // 首先尝试使用 docker compose（新版本）
    let result = Command::new("docker").arg("compose").args(args).status();

    match result {
        Ok(status) => {
            if status.success() {
                log::debug!("docker compose 命令执行成功");
                Ok(())
            } else {
                let error = format!("docker compose 命令执行失败，退出码: {:?}", status.code());
                log::error!("{}", error);
                Err(Error::Container(error))
            }
        }
        Err(e) => {
            log::warn!("docker compose 命令不可用，尝试使用 docker-compose: {}", e);

            // 尝试使用 docker-compose（旧版本）
            let result = Command::new("docker-compose").args(args).status();

            match result {
                Ok(status) => {
                    if status.success() {
                        log::debug!("docker-compose 命令执行成功");
                        Ok(())
                    } else {
                        let error =
                            format!("docker-compose 命令执行失败，退出码: {:?}", status.code());
                        log::error!("{}", error);
                        Err(Error::Container(error))
                    }
                }
                Err(e) => {
                    let error = format!("docker-compose 命令也不可用: {}", e);
                    log::error!("{}", error);
                    Err(Error::Container(error))
                }
            }
        }
    }
}

/// 获取微应用信息
///
/// 对于 Static 和 Api 类型，从扫描结果中查找
/// 对于 Internal 类型，从配置的 path 创建微应用信息
///
/// # 参数
/// - `app_config`: 应用配置
/// - `micro_apps`: 扫描发现的微应用列表
///
/// # 返回
/// 返回微应用信息
fn get_micro_app_info(
    app_config: &crate::config::AppConfig,
    micro_apps: &[MicroApp],
) -> Result<MicroApp> {
    match app_config.app_type {
        AppType::Static | AppType::Api => {
            // 从扫描结果中查找
            micro_apps
                .iter()
                .find(|app| app.name == app_config.name)
                .cloned()
                .ok_or_else(|| Error::Config(format!("未找到微应用: {}", app_config.name)))
        }
        AppType::Internal => {
            // 从配置的 path 创建微应用信息
            let path = app_config.path.as_ref().ok_or_else(|| {
                Error::Config(format!(
                    "Internal 应用 '{}' 必须配置 path 字段",
                    app_config.name
                ))
            })?;

            let path_buf = PathBuf::from(path);
            let dockerfile = path_buf.join("Dockerfile");
            let env_file = path_buf.join(".env");
            let setup_script = {
                let script_path = path_buf.join("setup.sh");
                if script_path.exists() {
                    Some(script_path)
                } else {
                    None
                }
            };
            let clean_script = {
                let script_path = path_buf.join("clean.sh");
                if script_path.exists() {
                    Some(script_path)
                } else {
                    None
                }
            };

            Ok(MicroApp {
                name: app_config.name.clone(),
                path: path_buf,
                env_file,
                dockerfile,
                setup_script,
                clean_script,
            })
        }
    }
}

/// 计算相对路径
///
/// 计算目标路径相对于基准路径的相对路径
///
/// # 参数
/// - `base_path`: 基准路径（通常是当前工作目录）
/// - `target_path`: 目标路径
///
/// # 返回
/// 返回相对路径字符串
fn calculate_relative_path(base_path: &PathBuf, target_path: &PathBuf) -> Result<String> {
    log::debug!("计算相对路径: 基准={:?}, 目标={:?}", base_path, target_path);

    // 获取绝对路径
    let base_abs = base_path.canonicalize().map_err(|e| {
        log::error!("获取基准路径的绝对路径失败: {:?}, 错误: {}", base_path, e);
        Error::Config(format!("获取基准路径的绝对路径失败: {}", e))
    })?;

    let target_abs = target_path.canonicalize().map_err(|e| {
        log::error!("获取目标路径的绝对路径失败: {:?}, 错误: {}", target_path, e);
        Error::Config(format!("获取目标路径的绝对路径失败: {}", e))
    })?;

    // 计算相对路径
    let relative_path = pathdiff::diff_paths(&target_abs, &base_abs).ok_or_else(|| {
        log::error!(
            "无法计算相对路径: 基准={:?}, 目标={:?}",
            base_abs,
            target_abs
        );
        Error::Config("无法计算相对路径".to_string())
    })?;

    let relative_str = relative_path
        .to_str()
        .ok_or_else(|| {
            log::error!("相对路径包含无效字符: {:?}", relative_path);
            Error::Config("相对路径包含无效字符".to_string())
        })?
        .to_string();

    log::debug!("计算得到的相对路径: {}", relative_str);

    Ok(relative_str)
}

/// 执行启动命令
fn execute_start(config: &ProxyConfig, force_rebuild: bool) -> Result<()> {
    log::info!("开始启动微应用...");

    // 当 force_rebuild 为 true 时，启用 no_cache 以确保完全重建
    let no_cache = force_rebuild;
    if no_cache {
        log::info!("强制重建模式已启用，将使用 --no-cache 参数构建镜像");
    }

    // 1. 扫描微应用
    let micro_apps = discover_micro_apps(&config.scan_dirs)?;
    let discovered_names = get_micro_app_names(&micro_apps);

    // 2. 验证配置
    config.validate(&discovered_names)?;

    // 3. 创建Docker网络
    log::info!("创建Docker网络: {}", config.network_name);
    crate::network::create_network(&config.network_name)?;

    // 4. 初始化状态管理器
    let mut state_manager = StateManager::new(&config.state_file_path);
    state_manager.load()?;

    // 5. 处理每个配置的应用
    let mut network_infos = Vec::new();
    let mut env_files = HashMap::new();

    // 获取当前工作目录（docker-compose.yml 所在目录）
    let current_dir = std::env::current_dir().map_err(|e| {
        log::error!("获取当前工作目录失败: {}", e);
        Error::Config(format!("获取当前工作目录失败: {}", e))
    })?;

    log::debug!("当前工作目录: {:?}", current_dir);

    for app_config in &config.apps {
        log::info!("处理应用: {} ({:?})", app_config.name, app_config.app_type);

        // 获取微应用信息
        let micro_app = get_micro_app_info(app_config, &micro_apps)?;

        // 解析Dockerfile
        let dockerfile_info = dockerfile::parse_dockerfile(&micro_app.dockerfile)?;
        if dockerfile_info.exposed_ports.is_empty() {
            log::warn!("应用 '{}' 的Dockerfile中没有EXPOSE指令", app_config.name);
        }

        // 计算目录hash
        let current_hash = calculate_directory_hash(&micro_app.path)?;

        // 判断是否需要重新构建
        let needs_rebuild =
            force_rebuild || state_manager.needs_rebuild(&app_config.name, &current_hash);

        if needs_rebuild {
            log::info!("应用 '{}' 需要重新构建", app_config.name);

            // 执行setup脚本
            if let Some(ref setup_script) = micro_app.setup_script {
                log::info!("执行setup脚本: {:?}", setup_script);
                script::execute_setup_script(setup_script, &micro_app.path)?;
            }

            // 构建镜像（当 force_rebuild 为 true 时使用 no_cache）
            let image_name = format!("{}:latest", app_config.name);
            builder::build_image(
                &image_name,
                &micro_app.dockerfile,
                &micro_app.path,
                Some(&micro_app.env_file),
                no_cache,
            )?;

            // 更新状态
            state_manager.update_state(&app_config.name, current_hash, true);
        } else {
            log::info!("应用 '{}' 无需重新构建", app_config.name);
        }

        // 收集环境变量文件路径（如果存在）
        if micro_app.env_file.exists() {
            log::debug!(
                "应用 '{}' 的 .env 文件存在: {:?}",
                app_config.name,
                micro_app.env_file
            );
            // 计算相对于当前工作目录的相对路径
            let relative_env_path = calculate_relative_path(&current_dir, &micro_app.env_file)?;
            env_files.insert(app_config.name.clone(), relative_env_path);
            log::info!(
                "为应用 '{}' 添加环境变量文件: {}",
                app_config.name,
                env_files.get(&app_config.name).unwrap()
            );
        } else {
            log::debug!("应用 '{}' 的 .env 文件不存在", app_config.name);
        }

        // 创建网络地址信息
        let network_info = NetworkAddressInfo::new(
            app_config.name.clone(),
            app_config.container_name.clone(),
            app_config.container_port,
            &app_config.routes,
            config.nginx_host_port,
            &app_config.app_type,
        );
        network_infos.push(network_info);
    }

    // 6. 生成nginx配置
    log::info!("生成nginx配置...");
    let nginx_config = nginx::generate_nginx_config(
        &config.apps,
        &config.web_root,
        &config.cert_dir,
        &config.domain,
    )?;
    nginx::save_nginx_config(&nginx_config, &config.nginx_config_path)?;

    // 7. 生成docker-compose配置
    log::info!("生成docker-compose配置...");
    let compose_config = compose::generate_compose_config(
        &config.apps,
        &config.network_name,
        config.nginx_host_port,
        &env_files,
        &config.web_root,
        &config.cert_dir,
        &config.domain,
    )?;
    compose::save_compose_config(&compose_config, &config.compose_config_path)?;

    // 8. 生成网络地址列表
    log::info!("生成网络地址列表...");
    generate_network_list(
        &network_infos,
        &config.network_name,
        config.nginx_host_port,
        &config.network_list_path,
    )?;

    // 9. 保存状态
    state_manager.save()?;

    // 10. 停止并删除现有容器（确保使用最新配置）
    log::info!("停止并删除现有容器...");
    let down_args = vec!["-f", &config.compose_config_path, "down"];
    // 忽略down命令的错误，因为可能容器不存在
    let _ = run_docker_compose(&down_args);

    // 11. 启动容器
    log::info!("启动容器...");
    let compose_args = vec!["-f", &config.compose_config_path, "up", "-d"];
    run_docker_compose(&compose_args)?;

    log::info!("所有微应用启动成功！");
    log::info!("Nginx统一入口: http://localhost:{}", config.nginx_host_port);

    Ok(())
}

/// 执行停止命令
fn execute_stop(config: &ProxyConfig) -> Result<()> {
    log::info!("停止所有微应用...");

    let compose_args = vec!["-f", &config.compose_config_path, "stop"];
    run_docker_compose(&compose_args)?;

    log::info!("所有微应用已停止");
    Ok(())
}

/// 执行清理命令
fn execute_clean(config: &ProxyConfig, force: bool, clean_network: bool) -> Result<()> {
    log::info!("清理所有微应用...");

    // 如果不是强制清理，询问确认
    if !force {
        println!("确定要清理所有微应用吗？这将删除所有容器和镜像。");
        print!("输入 'yes' 确认: ");
        use std::io::Write;
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).map_err(|e| {
            log::error!("读取输入失败: {}", e);
            Error::Config(format!("读取输入失败: {}", e))
        })?;

        if input.trim() != "yes" {
            log::info!("取消清理操作");
            return Ok(());
        }
    }

    // 停止并删除容器
    log::info!("停止并删除容器...");
    let compose_args = vec!["-f", &config.compose_config_path, "down"];
    run_docker_compose(&compose_args)?;

    // 删除镜像
    log::info!("删除镜像...");
    for app_config in &config.apps {
        let image_name = format!("{}:latest", app_config.name);
        builder::remove_image(&image_name)?;
    }

    // 执行clean脚本
    let micro_apps = discover_micro_apps(&config.scan_dirs)?;
    for app_config in &config.apps {
        // 获取微应用信息（包括 Internal 类型）
        let micro_app = get_micro_app_info(app_config, &micro_apps)?;

        if let Some(ref clean_script) = micro_app.clean_script {
            log::info!("执行clean脚本: {:?}", clean_script);
            if let Err(e) = script::execute_clean_script(clean_script, &micro_app.path) {
                log::warn!("执行clean脚本失败: {}", e);
            }
        }
    }

    // 删除状态文件
    log::info!("删除状态文件...");
    if std::fs::remove_file(&config.state_file_path).is_ok() {
        log::info!("状态文件已删除");
    }

    // 删除网络
    if clean_network {
        log::info!("删除Docker网络...");
        crate::network::remove_network(&config.network_name)?;
    }

    log::info!("清理完成");
    Ok(())
}

/// 执行状态查看命令
fn execute_status(config: &ProxyConfig) -> Result<()> {
    log::info!("查看微应用状态...");

    println!("=== 微应用状态 ===\n");

    // 检查容器状态
    for app_config in &config.apps {
        let status = container::get_container_status(&app_config.container_name)?;
        let running = container::is_container_running(&app_config.container_name)?;

        println!("应用: {} ({:?})", app_config.name, app_config.app_type);
        println!("  容器: {}", app_config.container_name);
        println!("  状态: {:?}", status);
        println!("  运行中: {}", running);
        println!();
    }

    // 检查镜像状态
    println!("=== 镜像状态 ===\n");
    for app_config in &config.apps {
        let image_name = format!("{}:latest", app_config.name);
        let exists = builder::image_exists(&image_name)?;
        println!(
            "镜像: {} - {}",
            image_name,
            if exists { "存在" } else { "不存在" }
        );
    }

    Ok(())
}

/// 执行网络地址查看命令
fn execute_network(config: &ProxyConfig, output: Option<PathBuf>) -> Result<()> {
    log::info!("查看网络地址...");

    // 扫描微应用
    let micro_apps = discover_micro_apps(&config.scan_dirs)?;
    let discovered_names = get_micro_app_names(&micro_apps);

    // 验证配置
    config.validate(&discovered_names)?;

    // 生成网络地址信息
    let mut network_infos = Vec::new();
    for app_config in &config.apps {
        let network_info = NetworkAddressInfo::new(
            app_config.name.clone(),
            app_config.container_name.clone(),
            app_config.container_port,
            &app_config.routes,
            config.nginx_host_port,
            &app_config.app_type,
        );
        network_infos.push(network_info);
    }

    // 确定输出路径
    let output_path = output
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| config.network_list_path.clone());

    // 生成网络地址列表
    generate_network_list(
        &network_infos,
        &config.network_name,
        config.nginx_host_port,
        &output_path,
    )?;

    println!("网络地址列表已生成: {}", output_path);

    // 同时打印到控制台
    println!("\n=== 网络地址信息 ===\n");
    for info in &network_infos {
        println!("{}", info.format());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse() {
        let args = vec![
            "micro_proxy".to_string(),
            "--config".to_string(),
            "./test.yml".to_string(),
            "start".to_string(),
        ];

        let cli = Cli::parse_from(&args);
        assert_eq!(cli.config, PathBuf::from("./test.yml"));
        assert!(matches!(cli.command, Commands::Start { .. }));
    }

    #[test]
    fn test_cli_parse_verbose() {
        let args = vec![
            "micro_proxy".to_string(),
            "-v".to_string(),
            "status".to_string(),
        ];

        let cli = Cli::parse_from(&args);
        assert!(cli.verbose);
        assert!(matches!(cli.command, Commands::Status));
    }
}
