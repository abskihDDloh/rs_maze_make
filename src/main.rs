mod maze_field;
mod maze_point;
mod maze_point_status;
mod maze_thread;

use clap::{Parser, arg, command};
use log::{LevelFilter, error, info};
use std::{
    fs,
    path::PathBuf,
    sync::{Arc, RwLock},
};
use threadpool::ThreadPool;

use crate::{
    maze_field::MazePoints, maze_point_status::MazePointStatus, maze_thread::maze_thread_func,
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

    #[arg(short = 'd', long = "debug", help = "デバッグモードを有効にします。")]
    debug: bool,
}

fn get_workers_limit() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

fn make_maze(
    x_size: u32,
    y_size: u32,
) -> Result<Arc<RwLock<MazePoints>>, Box<dyn std::error::Error>> {
    let maze_points = MazePoints::initialize_maze_points(x_size, y_size)?;

    info!("Maze initialized with size {}x{}", x_size, y_size);
    let num_threads = get_workers_limit();
    let pool = ThreadPool::new(num_threads);
    info!("ThreadPool created with {} threads", num_threads);

    for thread_id in 0..num_threads {
        let maze_points_clone = Arc::clone(&maze_points);

        pool.execute(move || {
            info!("Starting maze thread {}", thread_id);

            match maze_thread_func(&maze_points_clone) {
                Ok(()) => {
                    info!("Maze thread {} completed successfully", thread_id);
                }
                Err(e) => {
                    error!("Maze thread {} failed: {}", thread_id, e);
                }
            }
        });
    }
    pool.join();
    info!("All maze generation threads completed");

    Ok(maze_points)
}

fn save_maze_result_as_png(
    maze_points: &Arc<RwLock<MazePoints>>,
    file_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let maze_guard = maze_points.read().map_err(|_| {
        Box::new(std::io::Error::other(
            "Failed to acquire read lock for result display",
        ))
    })?;

    let (width, height) = (maze_guard.x_size(), maze_guard.y_size());
    let maze = maze_guard.get_all_maze_point_clone();
    let mut img = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::new(width, height);

    for (point, status) in maze {
        let color = match status {
            MazePointStatus::Path => image::Rgba([255u8, 255u8, 255u8, 255u8]), // 白
            MazePointStatus::Wall(..) => image::Rgba([0u8, 0u8, 0u8, 255u8]),   // 黒
        };
        img.put_pixel(point.x(), point.y(), color);
    }
    img.save(file_path)?;

    Ok(())
}

fn start(x_size: u32, y_size: u32, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // canonicalizeではなく、PathBufを直接使用
    let full_path = PathBuf::from(file_path);

    // 親ディレクトリが存在しない場合は作成
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let maze_points = make_maze(x_size, y_size)?;
    save_maze_result_as_png(&maze_points, &full_path)?;
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
    let now = chrono::Local::now();
    let default_file_name = format!("{}.png", now.format("%Y%m%d%H%M%S"));
    start(args.x_size, args.y_size, &default_file_name).unwrap_or_else(|e| {
        error!("Failed to start maze generation: {}", e);
        std::process::exit(1);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_start_function_basic() {
        // ログ初期化（テスト用）
        let _ = env_logger::builder()
            .filter_level(LevelFilter::Debug)
            .is_test(true)
            .try_init();

        // 一時ディレクトリを作成
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("sample.png");
        let file_path_str = file_path
            .to_str()
            .expect("Failed to convert path to string");

        // ファイルを事前に作成（空ファイル）
        fs::write(&file_path, b"").expect("Failed to create sample file");

        // start関数を実行
        let result = start(5, 5, file_path_str);

        // 結果の検証
        assert!(result.is_ok(), "start() should succeed: {:?}", result);

        // ファイルが存在することを確認
        assert!(file_path.exists(), "Output file should exist");

        // ファイルサイズが0より大きいことを確認
        let metadata = fs::metadata(&file_path).expect("Failed to get file metadata");
        assert!(metadata.len() > 0, "Output file should not be empty");

        // 一時ディレクトリは自動的にクリーンアップされる
    }

    #[test]
    fn test_start_function_with_different_sizes() {
        let _ = env_logger::builder()
            .filter_level(LevelFilter::Debug)
            .is_test(true)
            .try_init();

        let test_cases = vec![(5, 5), (7, 7), (9, 9)];

        for (x_size, y_size) in test_cases {
            let temp_dir = TempDir::new().expect("Failed to create temp directory");
            let file_path = temp_dir
                .path()
                .join(format!("test_{}x{}.png", x_size, y_size));
            let file_path_str = file_path
                .to_str()
                .expect("Failed to convert path to string");

            // ファイルを事前に作成
            fs::write(&file_path, b"").expect("Failed to create test file");

            // start関数を実行
            let result = start(x_size, y_size, file_path_str);

            // 結果の検証
            assert!(
                result.is_ok(),
                "start({}, {}) should succeed: {:?}",
                x_size,
                y_size,
                result
            );

            // ファイルが存在することを確認
            assert!(
                file_path.exists(),
                "Output file for {}x{} should exist",
                x_size,
                y_size
            );
        }
    }

    #[test]
    fn test_start_function_invalid_file_path() {
        let _ = env_logger::builder()
            .filter_level(LevelFilter::Debug)
            .is_test(true)
            .try_init();

        // 存在しないディレクトリのパス
        let invalid_path = "/non_existent_directory/sample.png";

        // start関数を実行（エラーが期待される）
        let result = start(5, 5, invalid_path);

        // エラーが返されることを確認
        assert!(result.is_err(), "start() should fail with invalid path");
    }

    #[test]
    fn test_save_maze_result_as_png() {
        let _ = env_logger::builder()
            .filter_level(LevelFilter::Debug)
            .is_test(true)
            .try_init();

        // 迷路を生成
        let maze_points = make_maze(5, 5).expect("Failed to create maze");

        // 一時ディレクトリを作成
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test_save.png");

        // ファイルを事前に作成
        fs::write(&file_path, b"").expect("Failed to create test file");

        // PNG保存関数をテスト
        let result = save_maze_result_as_png(&maze_points, &file_path);

        // 結果の検証
        assert!(
            result.is_ok(),
            "save_maze_result_as_png should succeed: {:?}",
            result
        );

        // ファイルが存在することを確認
        assert!(file_path.exists(), "PNG file should exist");

        // ファイルサイズが0より大きいことを確認
        let metadata = fs::metadata(&file_path).expect("Failed to get file metadata");
        assert!(metadata.len() > 0, "PNG file should not be empty");
    }

    #[test]
    fn test_get_workers_limit() {
        let workers = get_workers_limit();

        // ワーカー数は1以上であることを確認
        assert!(workers >= 1, "Worker count should be at least 1");

        // ワーカー数が合理的な範囲内であることを確認（最大128とする）
        assert!(workers <= 128, "Worker count should be reasonable");
    }
}
