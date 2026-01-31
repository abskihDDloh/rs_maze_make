use clap::Parser;
use dotenv::dotenv;
use log::{LevelFilter, error, info, warn};

use rs_maze_maker::common::database::connector::establish_connection;
use rs_maze_maker::common::database::initializer::initialize_db;

use crate::maze::connect_to_outside_wall::connect_all_not_connected_threads_to_outside_wall;
use crate::maze::maze_thread::{
    execute_maze_threads_threads_to_outside_wall, maze_thread_function,
};

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
        default_value_t = 0,
        help = "迷路生成用のスレッド数を指定します。デフォルトはCPUコア数の半分か4のうち大きい方。最大64まで指定可能です。"
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

    // max_threadsが0の場合、CPUコア数の半分か4のうち大きい方を設定
    // max_threadsが64より大きい場合、64に設定
    let thread_limit = if max_threads == 0 {
        std::cmp::max(num_cpus::get() as u32 / 2, 4)
    } else if max_threads > 64 {
        64
    } else {
        max_threads
    } as usize;

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

    info!("Database connection established successfully. Initializing database...");

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

    info!(
        "Initialize complete. Starting maze generation with {} threads...",
        thread_limit
    );

    execute_maze_threads_threads_to_outside_wall(&db_conn, thread_limit)
        .await
        .unwrap_or_else(|e| {
            error!("Maze generation failed: {}", e);
            std::process::exit(3);
        });

    let generate_wall_time_val =
        rs_maze_maker::common::util::get_end_time_and_elapsed_time(start_time);
    info!(
        "Starting to connect all not connected threads to outside wall. Time after maze generation (UNIXTIME): {} Elapsed time (seconds): {}",
        generate_wall_time_val.0, generate_wall_time_val.1
    );

    connect_all_not_connected_threads_to_outside_wall(&db_conn, thread_limit)
        .await
        .unwrap_or_else(|e| {
            error!(
                "Failed to connect all not connected threads to outside wall: {}",
                e
            );
        });

    let end_time_val = rs_maze_maker::common::util::get_end_time_and_elapsed_time(start_time);
    info!(
        "Application ended with args: {:?} End time (UNIXTIME): {} Elapsed time (seconds): {}",
        args, end_time_val.0, end_time_val.1
    );
}
