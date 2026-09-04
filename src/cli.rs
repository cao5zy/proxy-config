//! 命令行接口模块
//!
//! 负责提供命令行交互接口

use crate::config::{AppType, ProxyConfig};
use crate::container;
use crate::deployment::{
    active_runtime_apps, deployment_state_path, image_reference, runtime_apps_with_candidate,
    DeploymentStore,
};
use crate::discovery::{discover_micro_apps, get_micro_app_names, to_app_configs, MicroApp};
use crate::network::{generate_network_list, NetworkAddressInfo};
use crate::nginx;
use crate::script;
use crate::state::{calculate_directory_hash, StateManager};
use crate::volumes_config::VolumesConfig;
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
    Start,
    /// 构建镜像并登记为候选版本，不影响运行中的站点
    Build {
        /// 指定要构建的应用；省略时构建全部应用
        apps: Vec<String>,
        /// 禁用 Docker 构建缓存
        #[arg(long)]
        no_cache: bool,
    },
    /// 将已构建的候选镜像部署到指定应用
    Deploy {
        app: String,
        #[arg(long)]
        image: String,
    },
    /// 回滚指定应用到上一活动镜像
    Rollback { app: String },
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
        Commands::Start => execute_start(&config)?,
        Commands::Build { apps, no_cache } => execute_build(&config, &apps, no_cache)?,
        Commands::Deploy { app, image } => execute_deploy(&config, &app, &image)?,
        Commands::Rollback { app } => execute_rollback(&config, &app)?,
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

            // 对于 Internal 类型，从 micro_app_config 加载配置
            let micro_app_config =
                crate::micro_app_config::MicroAppConfig::from_file(path_buf.join("micro-app.yml"))?;

            // 加载卷配置
            log::debug!("尝试加载 Internal 应用 '{}' 的卷配置", app_config.name);
            let volumes_config = VolumesConfig::from_file(path_buf.join("micro-app.volumes.yml"))?;

            // 验证卷配置
            volumes_config.validate(&app_config.name)?;

            Ok(MicroApp {
                name: app_config.name.clone(),
                path: path_buf,
                config: micro_app_config,
                volumes_config,
                dockerfile,
                env_file,
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

/// `start` 只恢复已选择的活动镜像，不扫描源码，也不构建镜像。
fn execute_start(config: &ProxyConfig) -> Result<()> {
    let apps = config.load_required_apps()?;
    let deployments = load_deployments_with_legacy_migration(config, &apps)?;
    let (runtime_apps, images) = active_runtime_apps(&apps, deployments.states())?;
    write_runtime_configs(config, &runtime_apps, &images)?;
    deployments.save()?;
    crate::network::create_network(&config.network_name)?;
    let network_infos = runtime_apps
        .iter()
        .map(|app| {
            NetworkAddressInfo::new(
                app.name.clone(),
                app.container_name.clone(),
                app.container_port,
                &app.routes,
                config.nginx_host_port,
                &app.app_type,
            )
        })
        .collect::<Vec<_>>();
    generate_network_list(
        &network_infos,
        &config.network_name,
        config.nginx_host_port,
        &config.network_list_path,
    )?;
    let args = vec!["-f", &config.compose_config_path, "up", "-d"];
    run_docker_compose(&args)
}

/// 构建镜像并登记候选版本；此操作不生成运行配置，也不操作容器。
fn execute_build(config: &ProxyConfig, requested_apps: &[String], no_cache: bool) -> Result<()> {
    let micro_apps = discover_micro_apps(&config.scan_dirs)?;
    let discovered_names = get_micro_app_names(&micro_apps);
    let apps = to_app_configs(&micro_apps);
    config.validate(&apps, &discovered_names)?;
    config.save_apps(&apps)?;
    let mut deployments = DeploymentStore::new(deployment_state_path(&config.state_file_path));
    deployments.load()?;
    let mut state_manager = StateManager::new(&config.state_file_path);
    state_manager.load()?;
    for app in &apps {
        if !requested_apps.is_empty() && !requested_apps.contains(&app.name) {
            continue;
        }
        let micro_app = get_micro_app_info(app, &micro_apps)?;
        let source_hash = calculate_directory_hash(&micro_app.path)?;
        let image = image_reference(&app.name, &source_hash);
        if !builder::image_exists(&image)? {
            if let Some(script_path) = &micro_app.setup_script {
                script::execute_setup_script(script_path, &micro_app.path)?;
            }
            builder::build_image(
                &image,
                &micro_app.dockerfile,
                &micro_app.path,
                Some(&micro_app.env_file),
                no_cache,
            )?;
        }
        state_manager.update_state(&app.name, source_hash, true);
        deployments.set_candidate(&app.name, image.clone());
        println!("已构建候选镜像: {}", image);
    }
    if requested_apps
        .iter()
        .any(|name| !apps.iter().any(|app| &app.name == name))
    {
        return Err(Error::Config("指定的应用未被发现".to_string()));
    }
    state_manager.save()?;
    deployments.save()
}

fn execute_deploy(config: &ProxyConfig, app_name: &str, image: &str) -> Result<()> {
    let apps = config.load_apps()?;
    let target = config
        .get_app_config(&apps, app_name)
        .ok_or_else(|| Error::Config(format!("未找到应用: {}", app_name)))?;
    if !builder::image_exists(image)? {
        return Err(Error::Build(format!("镜像不存在: {}", image)));
    }
    let app_path = target.path.as_ref().ok_or_else(|| {
        Error::Config(format!("应用 '{}' 缺少路径配置，无法初始化卷权限", app_name))
    })?;
    let app_path = PathBuf::from(app_path);
    let volumes_config = VolumesConfig::from_file(app_path.join("micro-app.volumes.yml"))?;
    volumes_config.validate(app_name)?;
    setup_volume_permissions(app_name, &app_path, &volumes_config)?;
    let mut deployments = load_deployments_with_legacy_migration(config, &apps)?;
    deployments.set_candidate(app_name, image.to_string());
    let old_container = deployments
        .state(app_name)
        .and_then(|state| state.active_container.clone());
    let candidate_container =
        crate::deployment::candidate_container_name(&target.container_name, image);
    if old_container.as_deref() == Some(candidate_container.as_str()) {
        return Err(Error::State(format!(
            "镜像 '{}' 已是应用 '{}' 的活动版本",
            image, app_name
        )));
    }
    let (runtime_apps, images, _) =
        runtime_apps_with_candidate(&apps, deployments.states(), app_name, image)?;
    write_runtime_configs(config, &runtime_apps, &images)?;
    crate::network::create_network(&config.network_name)?;
    let args = vec![
        "-f",
        &config.compose_config_path,
        "up",
        "-d",
        "--no-deps",
        &candidate_container,
    ];
    run_docker_compose(&args)?;
    if !wait_for_healthy(&candidate_container)? {
        container::remove_container(&candidate_container)?;
        let (active_apps, active_images) = active_runtime_apps(&apps, deployments.states())?;
        write_runtime_configs(config, &active_apps, &active_images)?;
        return Err(Error::Container(format!(
            "候选容器 '{}' 未通过健康检查",
            candidate_container
        )));
    }
    let nginx_result = if container::is_container_running("proxy-nginx")? {
        container::reload_nginx()
    } else {
        let args = vec!["-f", &config.compose_config_path, "up", "-d", "nginx"];
        run_docker_compose(&args)
    };
    if let Err(error) = nginx_result {
        container::remove_container(&candidate_container)?;
        let (active_apps, active_images) = active_runtime_apps(&apps, deployments.states())?;
        write_runtime_configs(config, &active_apps, &active_images)?;
        return Err(error);
    }
    deployments.activate(app_name, image.to_string(), candidate_container.clone())?;
    deployments.save()?;
    if let Some(old) = old_container {
        container::remove_container(&old)?;
    }
    println!("应用 '{}' 已切换至镜像 {}", app_name, image);
    Ok(())
}

fn execute_rollback(config: &ProxyConfig, app_name: &str) -> Result<()> {
    let apps = config.load_apps()?;
    let deployments = load_deployments_with_legacy_migration(config, &apps)?;
    let image = deployments
        .state(app_name)
        .and_then(|state| state.previous_image.clone())
        .ok_or_else(|| Error::State(format!("应用 '{}' 没有可回滚镜像", app_name)))?;
    execute_deploy(config, app_name, &image)
}

fn load_deployments_with_legacy_migration(
    config: &ProxyConfig,
    apps: &[crate::config::AppConfig],
) -> Result<DeploymentStore> {
    let mut deployments = DeploymentStore::new(deployment_state_path(&config.state_file_path));
    deployments.load()?;
    let mut changed = false;
    for app in apps {
        changed |= deployments.migrate_legacy(
            &app.name,
            &app.container_name,
            builder::image_exists(&format!("{}:latest", app.name))?,
        );
    }
    if changed {
        deployments.save()?;
    }
    Ok(deployments)
}

fn collect_env_files(apps: &[crate::config::AppConfig]) -> Result<HashMap<String, String>> {
    let current_dir = std::env::current_dir()
        .map_err(|e| Error::Config(format!("获取当前工作目录失败: {}", e)))?;
    let mut env_files = HashMap::new();
    for app in apps {
        if let Some(path) = &app.path {
            let env = PathBuf::from(path).join(".env");
            if env.exists() {
                env_files.insert(
                    app.name.clone(),
                    calculate_relative_path(&current_dir, &env)?,
                );
            }
        }
    }
    Ok(env_files)
}

fn write_runtime_configs(
    config: &ProxyConfig,
    apps: &[crate::config::AppConfig],
    images: &HashMap<String, String>,
) -> Result<()> {
    let nginx_config =
        nginx::generate_nginx_config(apps, &config.web_root, &config.cert_dir, &config.domain)?;
    nginx::save_nginx_config(&nginx_config, &config.nginx_config_path)?;
    let compose_config = compose::generate_compose_config_with_images(
        apps,
        &config.network_name,
        config.nginx_host_port,
        &collect_env_files(apps)?,
        images,
        &config.web_root,
        &config.cert_dir,
        &config.domain,
    )?;
    compose::save_compose_config(&compose_config, &config.compose_config_path)
}

fn wait_for_healthy(container_name: &str) -> Result<bool> {
    for _ in 0..30 {
        if container::is_container_healthy(container_name)? {
            return Ok(true);
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
    Ok(false)
}

/// 在目标容器启动前设置卷权限。
///
/// 此函数只能由 deploy 调用；build 不得修改宿主机卷数据。
/// 先尝试直接执行，如果 chown 失败则尝试 sudo。
fn setup_volume_permissions(
    app_name: &str,
    app_path: &std::path::Path,
    volumes_config: &VolumesConfig,
) -> Result<()> {
    if let Some(script) = volumes_config.generate_permission_init_script(app_path) {
            log::info!("设置应用 '{}' 的卷权限...", app_name);

            // 将脚本写入临时文件
            let temp_dir = std::env::temp_dir();
            let script_path = temp_dir.join(format!("micro_proxy_vol_perms_{}.sh", app_name));
            std::fs::write(&script_path, &script).map_err(|e| {
                log::error!("写入权限初始化脚本失败: {}", e);
                Error::Config(format!("写入权限初始化脚本失败: {}", e))
            })?;

            // 设置可执行权限
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755)).ok();
            }

            // 执行脚本，先尝试直接运行，失败则尝试 sudo
            let result = std::process::Command::new("bash")
                .arg(script_path.to_str().unwrap_or(""))
                .output();

            match result {
                Ok(output) if output.status.success() => {
                    log::info!("应用 '{}' 卷权限设置成功", app_name);
                }
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    log::warn!(
                        "权限脚本执行失败（stderr: {}），尝试使用 sudo 重试...",
                        stderr.trim()
                    );
                    let sudo_result = std::process::Command::new("sudo")
                        .arg("bash")
                        .arg(script_path.to_str().unwrap_or(""))
                        .output();
                    match sudo_result {
                        Ok(sudo_output) if sudo_output.status.success() => {
                            log::info!("应用 '{}' 卷权限设置成功（通过 sudo）", app_name);
                        }
                        _ => {
                            let sudo_stderr = sudo_result
                                .as_ref()
                                .map(|o| String::from_utf8_lossy(&o.stderr).to_string())
                                .unwrap_or_else(|_| "unknown".to_string());
                            log::warn!(
                                "应用 '{}' 卷权限设置失败（sudo 也失败）: {}",
                                app_name,
                                sudo_stderr.trim()
                            );
                            log::warn!("请手动执行: sudo bash {}", script_path.display());
                        }
                    }
                }
                Err(e) => {
                    log::warn!(
                        "无法执行权限脚本: {}，请手动执行: sudo bash {}",
                        e,
                        script_path.display()
                    );
                }
            }

            // 清理临时文件
            let _ = std::fs::remove_file(&script_path);
    }
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

    // 加载动态配置以获取应用列表
    let apps = config.load_apps()?;

    // 清理未部署的候选镜像；活动和可回滚镜像必须保留。
    log::info!("清理未部署的候选镜像...");
    let mut deployments = DeploymentStore::new(deployment_state_path(&config.state_file_path));
    deployments.load()?;
    for state in deployments.states().values() {
        if let Some(image) = &state.candidate_image {
            builder::remove_image(image)?;
        }
    }

    // 执行clean脚本
    let micro_apps = discover_micro_apps(&config.scan_dirs)?;
    for app_config in &apps {
        // 获取微应用信息（包括 Internal 类型）
        let micro_app = get_micro_app_info(app_config, &micro_apps)?;

        if let Some(ref clean_script) = micro_app.clean_script {
            log::info!("执行clean脚本: {:?}", clean_script);
            if let Err(e) = script::execute_clean_script(clean_script, &micro_app.path) {
                log::warn!("执行clean脚本失败: {}", e);
            }
        }
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

    // 加载动态配置
    let apps = config.load_apps()?;

    println!("=== 微应用状态 ===\n");

    // 检查容器状态
    for app_config in &apps {
        let status = container::get_container_status(&app_config.container_name)?;
        let running = container::is_container_running(&app_config.container_name)?;

        println!("应用: {} ({:?})", app_config.name, app_config.app_type);
        println!("  容器: {}", app_config.container_name);
        println!("  状态: {:?}", status);
        println!("  运行中: {}", running);
        println!();
    }

    println!("=== 部署镜像状态 ===\n");
    let deployments = load_deployments_with_legacy_migration(config, &apps)?;
    for app in &apps {
        println!("应用: {}", app.name);
        if let Some(state) = deployments.state(&app.name) {
            for (label, image) in [
                ("活动", &state.active_image),
                ("可回滚", &state.previous_image),
                ("候选", &state.candidate_image),
            ] {
                if let Some(image) = image {
                    println!(
                        "  {}镜像: {} ({})",
                        label,
                        image,
                        if builder::image_exists(image)? {
                            "存在"
                        } else {
                            "不存在"
                        }
                    );
                }
            }
        } else {
            println!("  尚未部署");
        }
    }

    Ok(())
}

/// 执行网络地址查看命令
fn execute_network(config: &ProxyConfig, output: Option<PathBuf>) -> Result<()> {
    log::info!("查看网络地址...");

    // 扫描微应用
    let micro_apps = discover_micro_apps(&config.scan_dirs)?;
    let discovered_names = get_micro_app_names(&micro_apps);

    // 转换为 AppConfig
    let apps = to_app_configs(&micro_apps);

    // 验证配置
    config.validate(&apps, &discovered_names)?;

    // 生成网络地址信息
    let mut network_infos = Vec::new();
    for app_config in &apps {
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

    #[test]
    fn test_cli_parse_build_and_deploy() {
        let build = Cli::parse_from(["micro_proxy", "build", "api", "--no-cache"]);
        assert!(matches!(build.command, Commands::Build { apps, no_cache } if apps == ["api"] && no_cache));

        let deploy = Cli::parse_from([
            "micro_proxy",
            "deploy",
            "api",
            "--image",
            "api:sha-0123456789ab",
        ]);
        assert!(matches!(deploy.command, Commands::Deploy { app, image } if app == "api" && image == "api:sha-0123456789ab"));
    }
}
