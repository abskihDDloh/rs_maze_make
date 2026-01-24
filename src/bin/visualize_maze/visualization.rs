use image::{ImageBuffer, Rgba, RgbaImage};

use log::info;
use rs_maze_maker::common::database::entities::maze_cell_status_view;
use sea_orm::{DatabaseConnection, EntityTrait};
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::{fs, path};

/// MAZE_CELL_STATUS_VIEWのデータをPNGファイルとして可視化する
///
/// # Arguments
/// * `db` - データベース接続
/// * `file_path` - 保存先ファイルパス
/// * `colorize_by_thread` - falseの場合は単純な色分け、trueの場合はスレッドID別に色分け
pub async fn visualize_maze_to_png(
    db: &DatabaseConnection,
    path_obj: &Path,
    colorize_by_thread: bool,
) -> Result<(), Box<dyn Error>> {
    // 1. MAZE_CELL_STATUS_VIEWのレコードをすべて取得する
    let records = maze_cell_status_view::Entity::find().all(db).await?;
    if records.is_empty() {
        return Err("No data available".into());
    }

    // 座標の最大値を取得
    let max_x = records.iter().map(|r| r.x).max().unwrap_or(0);
    let max_y = records.iter().map(|r| r.y).max().unwrap_or(0);

    // 2. フラグに基づいて色マップを生成
    let color_map: HashMap<(String, Option<u64>), Rgba<u8>> = if colorize_by_thread {
        create_color_map_by_thread(&records)
    } else {
        create_color_map_simple()
    };

    // 3. 画像を作成
    let img_width = (max_x + 1) as u32;
    let img_height = (max_y + 1) as u32;
    let mut img: RgbaImage = ImageBuffer::new(img_width, img_height);

    // 画像全体を透明で初期化
    for pixel in img.pixels_mut() {
        *pixel = Rgba([0, 0, 0, 0]);
    }

    // 各セルに色を設定
    for record in &records {
        let key = (record.cell_type.clone(), record.cell_owner_thread_id);
        if let Some(&color) = color_map.get(&key) {
            img.put_pixel(record.x as u32, record.y as u32, color);
        }
    }

    // 画像を保存
    img.save(path_obj)?;

    Ok(())
}

/// フラグ=false時の色マップ生成（単純な色分け）
fn create_color_map_simple() -> HashMap<(String, Option<u64>), Rgba<u8>> {
    let mut color_map: HashMap<(String, Option<u64>), Rgba<u8>> = HashMap::new();

    // 壁（WALL, PILLAR）は黒、透過率100%
    color_map.insert(("WALL".to_string(), None), Rgba([0, 0, 0, 255]));
    color_map.insert(("PILLAR".to_string(), None), Rgba([0, 0, 0, 255]));

    // 通路（PATH）は白、透過率100%
    color_map.insert(("PATH".to_string(), None), Rgba([255, 255, 255, 255]));

    // 始点と終点（START, END）は透過率50%の赤
    color_map.insert(("START".to_string(), None), Rgba([255, 0, 0, 128]));
    color_map.insert(("END".to_string(), None), Rgba([255, 0, 0, 128]));

    // 経路（ROUTE）は透過率50%の緑
    color_map.insert(("ROUTE".to_string(), None), Rgba([0, 255, 0, 128]));

    color_map
}

/// フラグ=true時の色マップ生成（スレッドID別色分け）
fn create_color_map_by_thread(
    records: &[maze_cell_status_view::Model],
) -> HashMap<(String, Option<u64>), Rgba<u8>> {
    let mut color_map: HashMap<(String, Option<u64>), Rgba<u8>> = HashMap::new();
    let mut color_index = 0u32;

    for record in records {
        let key = (record.cell_type.clone(), record.cell_owner_thread_id);

        if let std::collections::hash_map::Entry::Vacant(e) = color_map.entry(key) {
            let color = if record.cell_type == "PATH" {
                // 通路は白、透過率100%
                Rgba([255, 255, 255, 255])
            } else if record.cell_type == "START" || record.cell_type == "END" {
                // 始点と終点は透過率50%の赤
                Rgba([255, 0, 0, 128])
            } else if record.cell_type == "ROUTE" {
                // 経路は透過率50%の緑
                Rgba([0, 255, 0, 128])
            } else if record.cell_owner_thread_id.is_none() {
                // 壁でCELL_OWNER_THREAD_IDがnullの場合は黒
                Rgba([0, 0, 0, 255])
            } else {
                // その他の壁（WALL, PILLAR）でCELL_OWNER_THREAD_IDが存在する場合
                // スレッドIDごとに個別の色を生成
                generate_unique_color_rgba(color_index)
            };

            e.insert(color);
            color_index += 1;
        }
    }

    color_map
}

/// 色インデックスから一意な色を生成する（RGBA版）
/// HSV色空間を使用して、視覚的に区別しやすい色を生成
fn generate_unique_color_rgba(index: u32) -> Rgba<u8> {
    let golden_ratio_conjugate = 0.618033988749895;
    let hue = (index as f64 * golden_ratio_conjugate) % 1.0;

    // 彩度と明度を固定（見やすさのため）
    let saturation = 0.7;
    let value = 0.9;

    hsv_to_rgba(hue, saturation, value, 255)
}

/// HSV色空間からRGBA色空間への変換
fn hsv_to_rgba(h: f64, s: f64, v: f64, alpha: u8) -> Rgba<u8> {
    let c = v * s;
    let h_prime = h * 6.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
    let m = v - c;

    let (r_prime, g_prime, b_prime) = match h_prime as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        5 => (c, 0.0, x),
        _ => (c, x, 0.0),
    };

    let r = ((r_prime + m) * 255.0) as u8;
    let g = ((g_prime + m) * 255.0) as u8;
    let b = ((b_prime + m) * 255.0) as u8;

    Rgba([r, g, b, alpha])
}
