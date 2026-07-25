mod maze;
mod set_start_and_goal;
use clap::{Parser, arg, command};
use log::{LevelFilter, debug, error, info};
use rand::RngExt;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    sync::{Arc, RwLock},
};
use threadpool::ThreadPool;

use crate::{
    maze::{
        maze_cell::{
            maze_point::{point::MazePoint, point_status::MazePointStatus},
            wall::wall_identifier::WallIdentifier,
        },
        maze_field::field::Field,
        maze_thread::{maze_generate_monitor_thread, maze_generate_thread},
    },
    set_start_and_goal::{
        set_start_and_goal_point::StartAndGoalSetter, solve::resolve_path_from_start_to_goal,
    },
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(
        short = 'x',
        long = "x_size",
        default_value = "5",
        help = "x方向のピクセル数を指定します。"
    )]
    x_size: u64,
    #[arg(
        short = 'y',
        long = "y_size",
        default_value = "5",
        help = "y方向のピクセル数を指定します。"
    )]
    y_size: u64,
    #[arg(
        short = 'f',
        long = "file_path",
        default_value = "",
        help = "保存先のファイルのパスを指定します。指定がない場合はユーザのホームディレクトリに実行日時で保存されます。"
    )]
    file_path: String,
    #[arg(
        short = 's',
        long = "solve",
        help = "迷路を解くためのオプションです。設定すると出力に解答が含まれます。(スタート=(1,1), ゴール=(x_size-2,y_size-2)とする。)"
    )]
    solve: bool,
    #[arg(
        short = 'c',
        long = "color",
        help = "迷路生成スレッドごとに壁の色を変更します。"
    )]
    color: bool,
    #[arg(
        short = 't',
        long = "number_of_threads",
        default_value = "0",
        help = "迷路生成用のスレッド数を指定します。デフォルトはCPUコア数か4のうち大きい方。最大64まで指定可能です。"
    )]
    number_of_threads: u32,
    #[arg(short = 'd', long = "debug", help = "デバッグモードを有効にします。")]
    debug: bool,
}

fn get_workers_limit() -> u32 {
    //コア数-1を返す。1未満の場合は1を返す。
    let num_cpus = num_cpus::get() as u32;
    if num_cpus > 1 { num_cpus - 1 } else { 1 }
}

fn make_maze(
    x_size: u64,
    y_size: u64,
    max_threads: u32,
) -> Result<Arc<RwLock<Field>>, Box<dyn std::error::Error>> {
    let maze_points = Field::initialize_maze_points(x_size, y_size)?;

    info!("Maze initialized with size {}x{}", x_size, y_size);
    let num_threads_u32_i = std::cmp::max(max_threads, get_workers_limit()) + 1; // 監視スレッド用に+1
    let num_threads_u32 = std::cmp::min(num_threads_u32_i, 64); // 最大64まで
    //u32をusizeに変換。変換できない場合は警告を
    let num_threads: usize = num_threads_u32.try_into().unwrap_or(4);

    let pool = ThreadPool::new(num_threads);
    info!(
        "ThreadPool created with {} threads (+ 1 monitor thread)",
        num_threads
    );

    // 監視スレッドを追加
    {
        let maze_points_clone = Arc::clone(&maze_points);
        pool.execute(move || {
            info!("Starting maze monitor thread");

            match maze_generate_monitor_thread(&maze_points_clone) {
                Ok(progress) => {
                    info!(
                        "Maze monitor thread completed. Final progress: {:?}",
                        progress
                    );
                }
                Err(e) => {
                    error!("Maze monitor thread failed: {}", e);
                }
            }
        });
    }

    // 迷路生成スレッドを追加
    for thread_id in 0..num_threads {
        let maze_points_clone = Arc::clone(&maze_points);

        pool.execute(move || {
            info!("Starting maze generation thread {}", thread_id);

            match maze_generate_thread(&maze_points_clone) {
                Ok(()) => {
                    info!(
                        "Maze generation thread {} completed successfully",
                        thread_id
                    );
                }
                Err(e) => {
                    error!("Maze generation thread {} failed: {}", thread_id, e);
                }
            }
        });
    }

    pool.join();
    info!("All maze generation and monitor threads completed");

    Ok(maze_points)
}

fn save_maze_result_as_png(
    maze_points: HashMap<MazePoint, MazePointStatus>,
    width: u64,
    height: u64,
    file_path: &PathBuf,
    solve_flag: bool,
    color_flag: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let maze_i = maze_points.clone();
    let mut wall_identifiers: HashSet<WallIdentifier> = HashSet::new();
    for (_point, status) in maze_i {
        if status.is_wall()
            && let Some(identifier) = status.get_wall_identifier()
        {
            wall_identifiers.insert(*identifier);
        }
    }
    let maze = maze_points.clone();
    let mut img = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::new(
        width.try_into().unwrap(),
        height.try_into().unwrap(),
    );
    let mut previous_color_list: HashSet<image::Rgba<u8>> = HashSet::new();
    let mut wall_color_list: HashMap<WallIdentifier, image::Rgba<u8>> = HashMap::new();

    if color_flag {
        // フラグがONのときは識別子ごとに壁の色を変える。(RGB2-254の範囲でランダムな色を使用)
        // wall_identifiersの件数だけループ
        for wall_identifier in &wall_identifiers {
            loop {
                let mut rng = rand::rng();
                let wall_color = image::Rgba([
                    rng.random_range(2..=254),
                    rng.random_range(2..=254),
                    rng.random_range(2..=254),
                    255u8,
                ]);
                if !previous_color_list.contains(&wall_color) {
                    previous_color_list.insert(wall_color);
                    wall_color_list.insert(*wall_identifier, wall_color);
                    break;
                }
            }
        }
        info!(
            "{} {} {}",
            previous_color_list.len(),
            wall_identifiers.len(),
            wall_color_list.len()
        );
    }

    let start_and_goal_color = image::Rgba([255u8, 1u8, 1u8, 128u8]); // 赤
    let path_color = image::Rgba([255u8, 255u8, 255u8, 255u8]); // 白
    let solved_path_color = image::Rgba([1u8, 255u8, 1u8, 128u8]); // 緑
    let black_color = image::Rgba([0u8, 0u8, 0u8, 255u8]); // 黒
    for (point, status) in maze {
        let color = if status.is_wall() {
            if let Some(identifier) = status.get_wall_identifier() {
                wall_color_list.get(identifier).unwrap_or(&black_color)
            } else {
                &black_color
            }
        } else if status.is_start_or_end_path() {
            &start_and_goal_color
        } else if solve_flag && status.is_resolved_path() {
            &solved_path_color
        } else {
            &path_color
        };
        debug!(
            "Point: {:?}, Color: {:?}, Status: {:?}",
            point, color, status
        );
        img.put_pixel(
            point.x().try_into().map_err(|_| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("x coordinate is too large for image output: {}", point.x()),
                )) as Box<dyn std::error::Error>
            })?,
            point.y().try_into().map_err(|_| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("y coordinate is too large for image output: {}", point.y()),
                )) as Box<dyn std::error::Error>
            })?,
            *color,
        );
    }

    img.save(file_path)?;

    Ok(())
}

fn start(
    x_size: u64,
    y_size: u64,
    max_threads: u32,
    file_path: &str,
    color_flag: bool,
    solve_flag: bool,
    debug_flag: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if file_path.is_empty() {
        // ファイルパスが空の場合はホームディレクトリに保存
        let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
        let default_file_name = format!("{}.png", chrono::Local::now().format("%Y%m%d%H%M%S"));
        let full_path = home_dir.join(default_file_name);
        return start(
            x_size,
            y_size,
            max_threads,
            full_path.to_str().unwrap(),
            color_flag,
            solve_flag,
            debug_flag,
        );
    }

    // canonicalizeではなく、PathBufを直接使用
    let full_path = PathBuf::from(file_path);

    if full_path.is_file() {
        // ファイルが存在する場合の処理
        // エラー。すでにファイルが存在する。
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("File already exists: {}", full_path.display()),
        )));
    }

    if full_path.is_dir() {
        // ディレクトリが存在する場合の処理
        // エラー。ディレクトリはファイルではない。
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Path is a directory, not a file: {}", full_path.display()),
        )));
    }

    // 親ディレクトリが存在しない場合は作成
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let maze_percell = make_maze(x_size, y_size, max_threads)?;

    let maze_guard = maze_percell.read().map_err(|_| {
        Box::new(std::io::Error::other(
            "Failed to acquire read lock for maze points",
        )) as Box<dyn std::error::Error>
    })?;

    let mut setter = StartAndGoalSetter::new(maze_guard.get_all_maze_points_clone());
    let sg: set_start_and_goal::start_and_goal_point::StartAndGoalPoint =
        setter.set_start_and_goal_point()?;
    let mut maze_points = setter.get_all_maze_points_clone();

    if solve_flag {
        // 迷路を解く処理
        let solve_list = resolve_path_from_start_to_goal(&mut maze_points, &sg)?;
        info!("Solved path: {:?}", solve_list);
    }

    save_maze_result_as_png(
        maze_points,
        maze_guard.x_size(),
        maze_guard.y_size(),
        &full_path,
        solve_flag,
        color_flag,
    )?;

    Ok(())
}

fn main() {
    let args = Args::parse();
    let solve_flag = args.solve;
    let color_flag = args.color;
    let debug_flag = args.debug;
    let max_threads = args.number_of_threads;
    let log_level = if debug_flag {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .init();

    info!("Application started with args: {:?}", args);
    let default_file_name = args.file_path.clone();
    start(
        args.x_size,
        args.y_size,
        max_threads,
        &default_file_name,
        color_flag,
        solve_flag,
        debug_flag,
    )
    .unwrap_or_else(|e| {
        error!("Failed to start maze generation: {}", e);
        std::process::exit(1);
    });
}
