//! move_next モジュールのテストコード
//!
//! このテストは、`move_next`モジュールの主要な機能をテストします：
//! - `get_adjacent_unused_extendable_pillar()` - 隣接する未使用の拡張可能な柱の取得
//! - `path_to_wall()` - 2つの柱間の経路セルをWALLに変更
//!
//! # テスト手順
//! 1. 5x5でDBを初期化
//! 2. MAZE_CELL_STATUS_VIEWの内容をログ出力
//! 3. 開始点Aを選択
//! 4. MAZE_CELL_STATUS_VIEWをログ出力
//! 5. USED_START_POINTS_VIEWをログ出力
//! 6. 開始点Bを取得
//! 7. MAZE_CELL_STATUS_VIEWをログ出力
//! 8. USED_START_POINTS_VIEWをログ出力
//! 9. 開始点A,B間のPATHをWALLに変更
//! 10. 中間点がWALLになっていることを確認
//! 11. 開始点A,BがDIRECT_CONNECTで存在することを確認
//!
//! # 実行方法
//! このテストを実行するには、以下の環境セットアップが必要です：
//!
//! 1. `.env` ファイルに `DATABASE_URL` を設定
//!    ```
//!    DATABASE_URL=mysql://user:password@localhost/maze_maker_db
//!    ```
//!
//! 2. MySQLサーバーが起動していることを確認
//!
//! 3. テストデータベースが存在することを確認
//!
//! 4. 以下のコマンドでテストを実行：
//!    ```sh
//!    cargo test --test move_next_test -- --nocapture --ignored
//!    ```
//!
//! # トラブルシューティング
//!
//! ## ConnectionAcquire(Timeout) エラーが発生する場合
//! - MySQLが起動しているか確認
//! - DATABASE_URLが正しいか確認
//! - ファイアウォール設定を確認
//! - MySQL接続タイムアウトを確認（デフォルト8秒）

use log::info;
use sea_orm::EntityTrait;
use sea_orm::TransactionTrait;

#[tokio::test]
#[test_log::test]
#[ignore] // DATABASE_URL が必要なため
async fn test_get_adjacent_unused_extendable_pillar_and_path_to_wall()
-> Result<(), Box<dyn std::error::Error>> {
    // ロギングの初期化
    let _ = env_logger::builder().is_test(true).try_init();

    // DB接続を確立
    info!("Attempting to connect to database...");
    let db = match rs_maze_maker::database::connector::establish_connection(None).await {
        Ok(conn) => {
            info!("✓ Database connection established");
            conn
        }
        Err(e) => {
            eprintln!("\n[ERROR] Failed to connect to database!");
            eprintln!("[ERROR] Error: {:?}\n", e);
            eprintln!("Please ensure:");
            eprintln!("  1. DATABASE_URL is set in .env file");
            eprintln!("  2. MySQLサーバーが起動しているか確認してください");
            eprintln!("  3. Example: DATABASE_URL=mysql://user:password@localhost/maze_maker_db\n");
            return Err(format!(
                "Database connection failed. See above for details. Error: {:?}",
                e
            )
            .into());
        }
    };

    // 1. initialize_db()で、5x5でDBを初期化する。
    info!("Step 1: Initializing database with 5x5 grid");
    rs_maze_maker::database::initializer::initialize_db(&db, 5, 5).await?;

    // 2. MAZE_CELL_STATUS_VIEWの内容をinfo!でログ出力する。
    info!("Step 2: Logging MAZE_CELL_STATUS_VIEW after initialization");
    let cell_statuses = rs_maze_maker::database::entities::maze_cell_status_view::Entity::find()
        .all(&db)
        .await?;
    for status in &cell_statuses {
        info!(
            "  Cell status: x={}, y={}, type={}, owner_thread_id={:?}",
            status.x, status.y, status.cell_type, status.cell_owner_thread_id
        );
    }

    // 3. select_random_start_point_from_db()で、開始点を取得する。(開始点A)
    info!("Step 3: Selecting random start point A");
    let tid_a = rs_maze_maker::maze::maze_thread_identifier::MazeThreadIdentifier::new();
    let start_point_a =
        rs_maze_maker::maze::independent_transaction_routines::select_random_start_point_from_db(
            &db, &tid_a,
        )
        .await?;
    info!(
        "  Start point A: ({}, {})",
        start_point_a.x(),
        start_point_a.y()
    );

    // 4. MAZE_CELL_STATUS_VIEWの内容をinfo!でログ出力する。
    info!("Step 4: Logging MAZE_CELL_STATUS_VIEW after selecting start point A");
    let cell_statuses = rs_maze_maker::database::entities::maze_cell_status_view::Entity::find()
        .all(&db)
        .await?;
    for status in &cell_statuses {
        info!(
            "  Cell status: x={}, y={}, type={}, owner_thread_id={:?}",
            status.x, status.y, status.cell_type, status.cell_owner_thread_id
        );
    }

    // 5. USED_START_POINTS_VIEWの内容をinfo!でログ出力する。
    info!("Step 5: Logging USED_START_POINTS_VIEW after selecting start point A");
    let used_start_points =
        rs_maze_maker::database::entities::used_start_points_view::Entity::find()
            .all(&db)
            .await?;
    for point in &used_start_points {
        info!(
            "  Used start point: x={}, y={}, outside_wall_connect_type={}",
            point.x, point.y, point.outside_wall_connect_type
        );
    }

    // トランザクションを開始して、次のステップを実行
    let txn1 = db.begin().await?;
    // 6. get_adjacent_unused_extendable_pillar()に上記3で取得した開始点を引き渡して次の開始点を取得する。(開始点B)
    info!("Step 6: Getting adjacent unused extendable pillar (start point B)");
    let start_point_b = rs_maze_maker::maze::move_next::get_adjacent_unused_extendable_pillar(
        &txn1,
        &tid_a,
        &start_point_a,
    )
    .await?;
    txn1.commit().await?;

    // 7. MAZE_CELL_STATUS_VIEWの内容をinfo!でログ出力する。
    info!("Step 7: Logging MAZE_CELL_STATUS_VIEW after getting start point B");
    let cell_statuses = rs_maze_maker::database::entities::maze_cell_status_view::Entity::find()
        .all(&db)
        .await?;
    for status in &cell_statuses {
        info!(
            "  Cell status: x={}, y={}, type={}, owner_thread_id={:?}",
            status.x, status.y, status.cell_type, status.cell_owner_thread_id
        );
    }

    // 8. USED_START_POINTS_VIEWの内容をinfo!でログ出力する。
    info!("Step 8: Logging USED_START_POINTS_VIEW after getting start point B");
    let used_start_points =
        rs_maze_maker::database::entities::used_start_points_view::Entity::find()
            .all(&db)
            .await?;
    for point in &used_start_points {
        info!(
            "  Used start point: x={}, y={}, outside_wall_connect_type={}",
            point.x, point.y, point.outside_wall_connect_type
        );
    }

    // 新しいトランザクションで path_to_wall を実行
    let txn2 = db.begin().await?;

    // 9. path_to_wall()で開始点A,開始点Bの間のPATHをWALLに変更する。(中間点)
    info!("Step 9: Calling path_to_wall() to change PATH to WALL between start points A and B");
    let intermediate_point =
        rs_maze_maker::maze::move_next::path_to_wall(&txn2, &tid_a, &start_point_a, &start_point_b)
            .await?;
    info!(
        "  Intermediate point (changed to WALL): ({}, {})",
        intermediate_point.x(),
        intermediate_point.y()
    );

    txn2.commit().await?;

    // 10. MAZE_CELL_STATUS_VIEWにて、中間点がWALLになっているいことを確認する。
    info!("Step 10: Verifying intermediate point is WALL in MAZE_CELL_STATUS_VIEW");
    let cell_statuses = rs_maze_maker::database::entities::maze_cell_status_view::Entity::find()
        .all(&db)
        .await?;

    let intermediate_status = cell_statuses
        .iter()
        .find(|s| s.x == intermediate_point.x() && s.y == intermediate_point.y())
        .ok_or("Intermediate point not found in MAZE_CELL_STATUS_VIEW")?;

    info!(
        "  Intermediate point status: x={}, y={}, type={}, owner_thread_id={:?}",
        intermediate_status.x,
        intermediate_status.y,
        intermediate_status.cell_type,
        intermediate_status.cell_owner_thread_id
    );

    assert_eq!(
        intermediate_status.cell_type,
        rs_maze_maker::database::initializer::MazeCellTypeEnum::WALL.to_string(),
        "Intermediate point should be WALL type"
    );
    assert!(
        intermediate_status.cell_owner_thread_id.is_some(),
        "Intermediate point should have owner thread ID"
    );

    // 11. USED_START_POINTS_VIEWに、開始点A,Bのレコードが存在していることを確認する。また、それらのレコードのUSED_START_POINTS_VIEWがDIRECT_CONNECTであることを確認する。
    info!(
        "Step 11: Verifying start points A and B exist in USED_START_POINTS_VIEW with DIRECT_CONNECT"
    );
    let used_start_points =
        rs_maze_maker::database::entities::used_start_points_view::Entity::find()
            .all(&db)
            .await?;

    info!("  Total used start points: {}", used_start_points.len());
    for point in &used_start_points {
        info!(
            "  Used start point: x={}, y={}, outside_wall_connect_type={}",
            point.x, point.y, point.outside_wall_connect_type
        );
    }

    // 開始点Aの確認
    let point_a_used = used_start_points
        .iter()
        .find(|p| p.x == start_point_a.x() && p.y == start_point_a.y())
        .ok_or("Start point A not found in USED_START_POINTS_VIEW")?;

    assert_eq!(
        point_a_used.outside_wall_connect_type,
        rs_maze_maker::database::initializer::OutsideWallConnectTypeEnum::DIRECT_CONNECT
            .to_string(),
        "Start point A should have DIRECT_CONNECT status"
    );
    info!("  ✓ Start point A exists with DIRECT_CONNECT");

    // 開始点Bの確認
    let point_b_used = used_start_points
        .iter()
        .find(|p| p.x == start_point_b.x() && p.y == start_point_b.y())
        .ok_or("Start point B not found in USED_START_POINTS_VIEW")?;

    assert_eq!(
        point_b_used.outside_wall_connect_type,
        rs_maze_maker::database::initializer::OutsideWallConnectTypeEnum::DIRECT_CONNECT
            .to_string(),
        "Start point B should have DIRECT_CONNECT status"
    );
    info!("  ✓ Start point B exists with DIRECT_CONNECT");

    info!("✓ All test assertions passed!");
    Ok(())
}
