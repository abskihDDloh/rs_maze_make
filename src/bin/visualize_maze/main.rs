use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
    time::SystemTime,
};

use clap::Parser;
use log::{LevelFilter, error, info};

use crate::visualization::visualize_maze_to_png;
use rs_maze_maker::common::database::connector::establish_connection;

mod visualization;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(
        short = 'f',
        long = "file_path",
        default_value_t = String::new(),
        help = "保存先のファイルのパスを指定します。指定がない場合はユーザのホームディレクトリに実行日時で保存されます。"
    )]
    file_path: String,
    #[arg(
        short = 'c',
        long = "color",
        default_value_t = false,
        help = "迷路生成スレッドごとに壁の色を変更します。"
    )]
    color: bool,
}

async fn start(
    file_path_str: String,
    use_thread_color: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = if file_path_str.is_empty() {
        // ホームディレクトリを取得する。
        let home_dir = dirs::home_dir().ok_or("Home directory not found.")?;
        // unixtimeを取得する。
        let unix_time = rs_maze_maker::common::util::get_now_unix_time();
        // ファイルパスを作成する。
        home_dir.join(format!("maze_visualization_{}.png", unix_time))
    } else {
        Path::new(&file_path_str).to_path_buf()
    };
    let db = establish_connection(None).await?;
    visualize_maze_to_png(&db, &file_path, use_thread_color).await?;
    info!("Maze visualization saved to {}", file_path.display());
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // ログ初期化
    env_logger::init();

    info!("Application started with args: {:?} ", args);

    let file_path_str = args.file_path;

    let use_thread_color = args.color;

    let res = start(file_path_str, use_thread_color).await;
    match res {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Error occurred: {}", e);
            Err(e)
        }
    }
}
