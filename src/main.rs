//! micro_proxy 主入口

use micro_proxy::cli;
use std::env;

fn main() {
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();

    // 如果没有参数，显示帮助
    if args.len() == 1 {
        cli::run(&["micro_proxy".to_string(), "--help".to_string()]).unwrap_or_else(|e| {
            eprintln!("错误: {}", e);
            std::process::exit(1);
        });
        return;
    }

    // 运行CLI
    if let Err(e) = cli::run(&args) {
        eprintln!("错误: {}", e);
        std::process::exit(1);
    }
}
