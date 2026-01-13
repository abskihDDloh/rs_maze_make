use crate::database::entities::maze_cell_status_view;
use image::{ImageBuffer, Rgb, RgbImage};
use sea_orm::{DatabaseConnection, EntityTrait};
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// MAZE_CELL_STATUS_VIEWのデータをPNGファイルとして可視化する
pub async fn visualize_maze_to_png(
    db: &DatabaseConnection,
    file_path: &PathBuf,
) -> Result<String, Box<dyn Error>> {
    // 1. MAZE_CELL_STATUS_VIEWのレコードをすべて取得する
    let records = maze_cell_status_view::Entity::find().all(db).await?;

    if records.is_empty() {
        return Err("データがありません".into());
    }

    // 座標の最大値を取得
    let max_x = records.iter().map(|r| r.x).max().unwrap_or(0);
    let max_y = records.iter().map(|r| r.y).max().unwrap_or(0);

    // 2. CELL_TYPE, CELL_OWNER_THREAD_ID列ごとにRGBの重複のない一覧を設定する
    // (CELL_TYPE, Option<CELL_OWNER_THREAD_ID>) の組み合わせごとに色を割り当てる
    let mut color_map: HashMap<(String, Option<u64>), Rgb<u8>> = HashMap::new();
    let mut color_index = 0u32;

    for record in &records {
        let key = (record.cell_type.clone(), record.cell_owner_thread_id);

        if let std::collections::hash_map::Entry::Vacant(e) = color_map.entry(key) {
            let color = if record.cell_type == "PATH" {
                // CELL_TYPEがPATHの場合は一律白
                Rgb([255, 255, 255])
            } else if record.cell_owner_thread_id.is_none() {
                // Nullの場合は一律黒
                Rgb([0, 0, 0])
            } else {
                // その他の場合は、色インデックスから一意な色を生成
                generate_unique_color(color_index)
            };

            e.insert(color);
            color_index += 1;
        }
    }

    // 3. X, Y列をX, Y座標、2で生成した色で各レコードの内容をpngファイルにして保存する
    let img_width = (max_x + 1) as u32;
    let img_height = (max_y + 1) as u32;
    let mut img: RgbImage = ImageBuffer::new(img_width, img_height);

    // 画像全体を黒で初期化
    for pixel in img.pixels_mut() {
        *pixel = Rgb([0, 0, 0]);
    }

    // 各セルに色を設定
    for record in &records {
        let key = (record.cell_type.clone(), record.cell_owner_thread_id);
        if let Some(&color) = color_map.get(&key) {
            img.put_pixel(record.x as u32, record.y as u32, color);
        }
    }

    // ファイル名はunixtime
    let unix_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let filename = format!("{}.png", unix_time);

    // 画像を保存
    img.save(file_path)?;

    Ok(filename)
}

/// 色インデックスから一意な色を生成する
/// HSV色空間を使用して、視覚的に区別しやすい色を生成
fn generate_unique_color(index: u32) -> Rgb<u8> {
    // 黄金比を使って色相を分散させる
    let golden_ratio_conjugate = 0.618033988749895;
    let hue = (index as f64 * golden_ratio_conjugate) % 1.0;

    // 彩度と明度を固定（見やすさのため）
    let saturation = 0.7;
    let value = 0.9;

    hsv_to_rgb(hue, saturation, value)
}

/// HSV色空間からRGB色空間への変換
fn hsv_to_rgb(h: f64, s: f64, v: f64) -> Rgb<u8> {
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

    Rgb([r, g, b])
}

#[cfg(test)]
mod tests {
    use crate::maze::maze_thread::maze_thread_function;

    use super::*;

    #[test]
    fn test_generate_unique_color() {
        // 複数の色を生成して、それぞれ異なることを確認
        let color1 = generate_unique_color(0);
        let color2 = generate_unique_color(1);
        let color3 = generate_unique_color(2);

        assert_ne!(color1, color2);
        assert_ne!(color2, color3);
        assert_ne!(color1, color3);
    }

    #[test]
    fn test_hsv_to_rgb() {
        // 赤 (H=0, S=1, V=1)
        let red = hsv_to_rgb(0.0, 1.0, 1.0);
        assert_eq!(red, Rgb([255, 0, 0]));

        // 白 (H=0, S=0, V=1)
        let white = hsv_to_rgb(0.0, 0.0, 1.0);
        assert_eq!(white, Rgb([255, 255, 255]));

        // 黒 (H=0, S=0, V=0)
        let black = hsv_to_rgb(0.0, 0.0, 0.0);
        assert_eq!(black, Rgb([0, 0, 0]));
    }

    #[tokio::test]
    #[test_log::test]
    #[ignore] // DATABASE_URL が必要なため
    async fn test_maze_thread_completes_successfully_with_visualize() {
        // ホームディレクトリを取得する。
        let home_dir = dirs::home_dir().expect("Failed to get home directory");
        // unixtimeを取得する。
        let unix_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        // ファイルパスを作成する。
        let file_path = home_dir.join(format!("maze_visualization_{}.png", unix_time));

        let db = crate::database::connector::establish_connection(None)
            // DB初期化（129x129グリッド）
            .await
            .expect("Failed to connect to database");
        crate::database::initializer::initialize_db(&db, 129, 129)
            .await
            .expect("Failed to initialize database");

        // LocalSetを使用して8つのタスクを並行実行
        let local_set = tokio::task::LocalSet::new();

        local_set
            .run_until(async {
                let mut handles = vec![];
                for _ in 0..8 {
                    let db_clone = db.clone();
                    let handle =
                        tokio::task::spawn_local(
                            async move { maze_thread_function(&db_clone).await },
                        );
                    handles.push(handle);
                }

                // すべてのタスクの完了を待つ
                for handle in handles {
                    let result = handle.await;
                    assert!(result.is_ok(), "Task join failed: {:?}", result.err());
                    let maze_result = result.unwrap();
                    assert!(
                        maze_result.is_ok(),
                        "maze_thread failed: {:?}",
                        maze_result.err()
                    );
                }
            })
            .await;

        // visualize_maze_to_png()を実行。
        let filename = visualize_maze_to_png(&db, &file_path).await;
        assert!(
            filename.is_ok(),
            "visualize_maze_to_png failed: {:?}",
            filename.err()
        );
        eprintln!("Generated PNG file: {}", filename.unwrap());
    }
}
