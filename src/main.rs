mod maze;
use clap::{Parser, arg, command};
use log::{LevelFilter, error, info};
use rand::Rng;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    sync::{Arc, RwLock},
};
use threadpool::ThreadPool;

use crate::maze::{
    maze_cell::{
        maze_point::{point::MazePoint, point_status::MazePointStatus},
        wall::wall_identifier::WallIdentifier,
    },
    maze_field::field::Field,
    maze_thread::{maze_generate_monitor_thread, maze_generate_thread},
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
    x_size: u32,
    #[arg(
        short = 'y',
        long = "y_size",
        default_value = "5",
        help = "y方向のピクセル数を指定します。"
    )]
    y_size: u32,
    #[arg(
        short = 'f',
        long = "file_path",
        default_value = "",
        help = "保存先のファイルのパスを指定します。指定がない場合はユーザのホームディレクトリに実行日時で保存されます。"
    )]
    file_path: String,
    #[arg(short = 'd', long = "debug", help = "デバッグモードを有効にします。")]
    debug: bool,
}

fn get_workers_limit() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

fn make_maze(x_size: u32, y_size: u32) -> Result<Arc<RwLock<Field>>, Box<dyn std::error::Error>> {
    let maze_points = Field::initialize_maze_points(x_size, y_size)?;

    info!("Maze initialized with size {}x{}", x_size, y_size);
    let num_threads = get_workers_limit();
    let pool = ThreadPool::new(num_threads + 1); // 監視スレッド用に+1
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
    width: u32,
    height: u32,
    file_path: &PathBuf,
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
    let mut img = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::new(width, height);
    let mut previous_color_list: HashSet<image::Rgba<u8>> = HashSet::new();
    let mut wall_color_list: HashMap<WallIdentifier, image::Rgba<u8>> = HashMap::new();

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
    for (point, status) in maze {
        let path_color = image::Rgba([255u8, 255u8, 255u8, 255u8]); // 白
        let color = if status.is_wall() {
            if let Some(identifier) = status.get_wall_identifier() {
                wall_color_list.get(identifier).unwrap_or(&path_color)
            } else {
                &path_color
            }
        } else {
            &path_color
        };
        img.put_pixel(point.x(), point.y(), *color);
    }

    img.save(file_path)?;

    Ok(())
}

fn start(x_size: u32, y_size: u32, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    
    if file_path.is_empty() {
        // ファイルパスが空の場合はホームディレクトリに保存
        let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
        let default_file_name = format!("{}.png", chrono::Local::now().format("%Y%m%d%H%M%S"));
        let full_path = home_dir.join(default_file_name);
        return start(x_size, y_size, full_path.to_str().unwrap());
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
    let maze_points = make_maze(x_size, y_size)?;

    let maze_guard = maze_points.read().map_err(|_| {
        Box::new(std::io::Error::other(
            "Failed to acquire read lock for maze points",
        )) as Box<dyn std::error::Error>
    })?;
    save_maze_result_as_png(
        maze_guard.get_all_maze_points_clone(),
        maze_guard.x_size(),
        maze_guard.y_size(),
        &full_path,
    )?;

    Ok(())
}

fn main() {
    let args = Args::parse();

    let log_level = if args.debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .init();

    info!("Application started with args: {:?}", args);
    let default_file_name = args.file_path.clone();
    start(args.x_size, args.y_size, &default_file_name).unwrap_or_else(|e| {
        error!("Failed to start maze generation: {}", e);
        std::process::exit(1);
    });
}
