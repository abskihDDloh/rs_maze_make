use clap::Parser;
use dotenv::dotenv;
use log::{LevelFilter, error, info, warn};

use rs_maze_maker::common::database::connector::establish_connection;
use rs_maze_maker::common::database::initializer::initialize_db;

use crate::maze::maze_thread::maze_thread_function;

mod maze;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(
        short = 'x',
        long = "x_size",
        default_value_t = 5,
        help = "x方向のピクセル数を指定します。"
    )]
    x_size: u64,
    #[arg(
        short = 'y',
        long = "y_size",
        default_value_t = 5,
        help = "y方向のピクセル数を指定します。"
    )]
    y_size: u64,
    #[arg(
        short = 't',
        long = "number_of_threads",
        default_value_t = 5,
        help = "迷路生成用のスレッド数を指定します。デフォルトはCPUコア数か4のうち大きい方。最大64まで指定可能です。"
    )]
    number_of_threads: u32,
}

#[tokio::main]
async fn main() {
    // .envファイルから環境変数を読み込む
    dotenv().ok();
    
    // ログ初期化
    env_logger::init();

    let args = Args::parse();
    let x = args.x_size;
    let y = args.y_size;
    let max_threads = args.number_of_threads;

    let start_time = rs_maze_maker::common::util::get_now_unix_time();
    info!(
        "Application started with args: {:?} Start time (UNIXTIME): {}",
        args, start_time
    );

    let db_connect_res = establish_connection(Some(1)).await;
    let db_conn = match db_connect_res {
        Ok(conn) => conn,
        Err(e) => {
            let end_time_val =
                rs_maze_maker::common::util::get_end_time_and_elapsed_time(start_time);
            error!(
                "Failed to establish database connection: {} End time (UNIXTIME): {} Elapsed time (seconds): {}",
                e, end_time_val.0, end_time_val.1
            );
            std::process::exit(1);
        }
    };
    let initialize_db_res = initialize_db(&db_conn, x, y).await;
    match initialize_db_res {
        Ok(_) => info!("Database initialized successfully."),
        Err(e) => {
            let end_time_val =
                rs_maze_maker::common::util::get_end_time_and_elapsed_time(start_time);
            error!(
                "Failed to initialize database: {} End time (UNIXTIME): {} Elapsed time (seconds): {}",
                e, end_time_val.0, end_time_val.1
            );
            std::process::exit(2);
        }
    }

    let local_set = tokio::task::LocalSet::new();
    local_set
        .run_until(async {
            let mut handles = vec![];
            for _ in 0..max_threads {
                let db_clone = db_conn.clone();
                let handle =
                    tokio::task::spawn_local(async move { maze_thread_function(&db_clone).await });
                handles.push(handle);
            }
            // すべてのタスクの完了を待つ
            for handle in handles {
                let result = handle.await;
                // 利用可能な開始点がなくなった時点で必ずエラーになる。
                match result {
                    Ok(_) => info!("Maze generation task completed successfully."),
                    Err(e) => warn!("Maze generation task failed: {}", e),
                }
            }
        })
        .await;
    let end_time_val = rs_maze_maker::common::util::get_end_time_and_elapsed_time(start_time);
    info!(
        "Application ended with args: {:?} End time (UNIXTIME): {} Elapsed time (seconds): {}",
        args, end_time_val.0, end_time_val.1
    );
}
