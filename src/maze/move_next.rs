//! 迷路生成における次の進行地点の選択と経路生成を行うモジュール
//!
//! このモジュールは迷路の拡張処理で以下の処理を提供します：
//!
//! - 隣接する未使用の拡張可能な柱（開始点）の取得
//! - 現在位置から次の柱への経路となるセルをWALLに変更
//!
//! # 機能
//!
//! 迷路の深さ優先探索的な拡張を行う際に、現在の開始点から隣接する未使用の
//! 開始点を選択し、その間のパス部分を壁に変更します。

use log::{debug, info, warn};
use rand::Rng;
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
extern crate strum;

use crate::database::initializer::{MazeCellTypeEnum, OutsideWallConnectTypeEnum};
use crate::maze::maze_point::select_between_points_without_edge;

use crate::maze::maze_thread_utility::{
    check_unused_pillars, get_cell_status, get_pillar, is_point_outside_wall,
    is_this_thread_connect_outside_wall, select_my_thread_record_from_tx,
};
use crate::maze::{maze_point::MazePoint, maze_thread_identifier::MazeThreadIdentifier};

macro_rules! debug_get_adjacent_extendable_pillar {
    ($tid:expr, $current_pillar:expr, $unused_points:expr) => {
        format!(
            "[tid: {:?}, current_pillar: {:?}, unused_points: {:?}]",
            $tid, $current_pillar, $unused_points
        )
    };
}

/// 現在の柱から隣接する未使用の拡張可能な柱をランダムに選択します。
///
/// 距離2の隣接柱候補から未使用で拡張可能な柱を探し、以下の条件を満たす柱を返します：
///
/// - UNUSED_START_POINTS_VIEWに存在する（未使用）
/// - 外壁からのスレッドの場合、外壁開始点でない
/// - 隣接セル（PATHセル）が未利用で有効
/// - MAZE_FIELDでセルの所有権を成功裏に取得できる
///
/// # 引数
/// * `txn` - データベーストランザクション
/// * `tid` - スレッド識別子
/// * `current_pillar` - 現在の柱の座標
///
/// # 戻り値
/// 選択された隣接柱の座標、またはエラー
///
/// # エラー
/// - 拡張可能な隣接柱がない場合
/// - セルの所有権取得に失敗した場合
/// - データベースエラーが発生した場合
pub async fn get_adjacent_unused_extendable_pillar(
    txn: &DatabaseTransaction,
    tid: &MazeThreadIdentifier,
    current_pillar: &MazePoint,
) -> Result<MazePoint, Box<dyn std::error::Error>> {
    // THREAD_LISTから、MazeThreadIdentifierの内容(THREAD_ID,CREATE_UNIXTIME)に当てはまるレコードを取得する。ない場合はエラー。
    let thread_record = select_my_thread_record_from_tx(txn, tid).await?;

    //THREAD_FROM_OUTSIDE_WALL_VIEWから、MazeThreadIdentifierの内容に当てはまるレコードを取得する。
    let thread_from_outside_wall = is_this_thread_connect_outside_wall(txn, tid).await?;

    let adjacent_pillars = current_pillar.generate_adjacent_maze_points(2);
    debug!(
        "{} Adjacent pillars candidates: {:?}",
        debug_get_adjacent_extendable_pillar!(tid, current_pillar, "N/A"),
        adjacent_pillars
    );

    //UNUSED_START_POINTS_VIEWから、adjacent_pillarsの内容に(X AND Y)が当てはまるレコードをすべて取得する。
    let mut unused_points: Vec<crate::database::entities::unused_start_points_view::Model> =
        check_unused_pillars(txn, &adjacent_pillars).await?;

    loop {
        if unused_points.is_empty() {
            return Err(format!(
                "No extendable adjacent pillars available. {}",
                debug_get_adjacent_extendable_pillar!(tid, current_pillar, "None")
            )
            .into());
        }
        debug!(
            "{}",
            debug_get_adjacent_extendable_pillar!(tid, current_pillar, unused_points)
        );
        //unused_pointsの中からランダムで1個選択する。
        let mut rng = rand::rng();
        let random_index = rng.random_range(0..unused_points.len());
        let selected_point = unused_points[random_index].clone();
        //選択した要素は取り除く。
        unused_points.remove(random_index);

        debug!(
            "{} Removed_pillar: {:?}",
            debug_get_adjacent_extendable_pillar!(tid, current_pillar, unused_points),
            selected_point
        );

        let is_outside_wall_start_point =
            is_point_outside_wall(txn, &MazePoint::new(selected_point.x, selected_point.y)).await;

        // OUTSIDE_WALL_START_POINTS_VIEWに、選択した要素の(X AND Y)が当てはまるレコードが存在するか確認する。
        if thread_from_outside_wall && is_outside_wall_start_point {
            // 外壁から来た壁は外壁にはゆかないようにする。
            info!(
                "{} this thread start from outside wall. Selected pillar is outside wall start point, skipping: {:?}",
                debug_get_adjacent_extendable_pillar!(tid, current_pillar, unused_points),
                selected_point
            );
            continue; // 次の候補へ
        }

        // 選択した要素ともとの要素の間にあるセルを計算する。
        let selected_maze_point = MazePoint::new(selected_point.x, selected_point.y);
        let bitween_cell = select_between_points_without_edge(current_pillar, &selected_maze_point);
        if bitween_cell.len() != 1 {
            warn!(
                "Invalid number of between cells: current_pillar=({},{}) selected_point=({},{}) between_cells={:?}",
                current_pillar.x(),
                current_pillar.y(),
                selected_maze_point.x(),
                selected_maze_point.y(),
                bitween_cell
            );
            continue; // 次の候補へ
        }
        let adjacent_cells_candidate = bitween_cell[0];

        // MAZE_CELL_STATUS_VIEWから、adjacent_cells_candidateの内容に(X AND Y)が当てはまるレコードを取得する。
        let adjacent_cell_status = get_cell_status(txn, &adjacent_cells_candidate).await?;

        // 取得したレコードが未利用のPATHでなければ次の候補へ
        if adjacent_cell_status.is_empty()
            || adjacent_cell_status[0].cell_type != MazeCellTypeEnum::PATH.to_string()
            || adjacent_cell_status[0].cell_owner_thread_id.is_some()
        {
            warn!(
                "Adjacent cell is not PATH: current_pillar=({},{}) selected_point=({},{}) adjacent_cell=({},{}) status={:?}",
                current_pillar.x(),
                current_pillar.y(),
                selected_maze_point.x(),
                selected_maze_point.y(),
                adjacent_cells_candidate.x(),
                adjacent_cells_candidate.y(),
                adjacent_cell_status
            );
            continue; // 次の候補へ
        }
        // selected_point.cell_id に当てはまるMAZE_FIELDのCELL_OWNER_THREAD_IDがNullの場合にかぎり、CELL_OWNER_THREAD_IDをthread_record.idに更新する。
        // エラーの場合は次の候補へ。
        let result = get_pillar(txn, selected_point.cell_id, thread_record.id).await;
        if result.is_err() {
            warn!(
                "{} Failed to get pillar: {:?}, error: {:?}",
                debug_get_adjacent_extendable_pillar!(tid, current_pillar, unused_points),
                selected_point,
                result.err()
            );
            continue; // 次の候補へ
        }

        //outside_wall_start_pointsが空でない場合は、THREAD_LISTのOUTSIDE_WALL_CONNECT_TYPEがNOT_CONNECTである場合に限り、tidの内容に当てはまるレコードのOUTSIDE_WALL_CONNECT_TYPEをDIRECT_CONNECTに更新する。
        if is_outside_wall_start_point
            && thread_record.outside_wall_connect_type
                == OutsideWallConnectTypeEnum::NOT_CONNECT.to_string()
        {
            let update_model = crate::database::entities::thread_list::ActiveModel {
                id: sea_orm::ActiveValue::Set(thread_record.id),
                outside_wall_connect_type: sea_orm::ActiveValue::Set(
                    OutsideWallConnectTypeEnum::DIRECT_CONNECT.to_string(),
                ),
                ..Default::default()
            };
            crate::database::entities::thread_list::Entity::update(update_model)
                .exec(txn)
                .await?;
            debug!(
                "{} Selected_pillar: {:?}, thread_record.outside_wall_connect_type will be updated to DIRECT_CONNECT",
                debug_get_adjacent_extendable_pillar!(tid, current_pillar, unused_points),
                selected_point
            );
        }
        return Ok(selected_maze_point);
    }
}

/// 2つの柱間の経路セルをWALLに変更します。
///
/// 現在の柱から次の柱への間にあるパスセルをWALLセルに変更し、
/// スレッドの所有権を設定します。
///
/// # 引数
/// * `txn` - データベーストランザクション
/// * `tid` - スレッド識別子
/// * `current_pillar` - 現在の柱の座標
/// * `next_pillar` - 次の柱の座標
///
/// # 戻り値
/// 変更されたセルの座標（中間点）、またはエラー
///
/// # エラー
/// - スレッドレコードが見つからない場合
/// - 中間地点のセル数が1でない場合（隣接していない座標）
/// - 中間地点が未使用のPATHセルでない場合
/// - データベースエラーが発生した場合
pub async fn path_to_wall(
    txn: &DatabaseTransaction,
    tid: &MazeThreadIdentifier,
    current_pillar: &MazePoint,
    next_pillar: &MazePoint,
) -> Result<MazePoint, Box<dyn std::error::Error>> {
    // THREAD_LISTから、MazeThreadIdentifierの内容(THREAD_ID,CREATE_UNIXTIME)に当てはまるレコードを取得する。ない場合はエラー。
    let thread = select_my_thread_record_from_tx(txn, tid).await?;
    let thread_id_from_table: u64 = thread.id;

    //current_pillarとnext_pillarの中間地点を計算する。
    let bitween_cells = select_between_points_without_edge(current_pillar, next_pillar);
    // 2個以上取得された場合はエラー。
    if bitween_cells.len() != 1 {
        // エラーメッセージにcurrent_pillar,next_pillar,bitween_cellsの内容を含める。
        return Err(format!("Invalid number of between cells: current_pillar=({},{}) next_pillar=({},{}) between_cells={:?}", current_pillar.x(), current_pillar.y(), next_pillar.x(), next_pillar.y(), bitween_cells).into());
    }
    let path_cell = bitween_cells[0];
    // 取得したレコードが未利用のPATHでなければエラー。
    let cell_status = get_cell_status(txn, &path_cell).await?;
    if cell_status.is_empty()
        || cell_status[0].cell_type != MazeCellTypeEnum::PATH.to_string()
        || cell_status[0].cell_owner_thread_id.is_some()
    {
        return Err(format!("Between cell is not unused PATH: current_pillar=({},{}) next_pillar=({},{}) between_cell=({},{}) status={:?}",
        current_pillar.x(),
        current_pillar.y(),
        next_pillar.x(),
        next_pillar.y(),
        bitween_cells[0].x(),
        bitween_cells[0].y(),
        cell_status
    ).into());
    }

    // MAZE_FIELDのIDがcell_statusから取得したIDで、CELL_OWNER_THREAD_IDがNullで、CELL_TYPEがPATHの場合に限り、該当するレコードののCELL_TYPEをPATHからWALLに変更し、CELL_OWNER_THREAD_IDにthread_record.idを設定する。
    let update_model = crate::database::entities::maze_field::ActiveModel {
        id: sea_orm::ActiveValue::Set(cell_status[0].cell_id),
        cell_type: sea_orm::ActiveValue::Set(MazeCellTypeEnum::WALL.to_string()),
        cell_owner_thread_id: sea_orm::ActiveValue::Set(Some(thread_id_from_table)),
        ..Default::default()
    };
    let result = crate::database::entities::maze_field::Entity::update(update_model)
        .filter(
            crate::database::entities::maze_field::Column::CellOwnerThreadId
                .is_null()
                .and(
                    crate::database::entities::maze_field::Column::CellType
                        .eq(MazeCellTypeEnum::PATH.to_string()),
                )
                .and(crate::database::entities::maze_field::Column::Id.eq(cell_status[0].cell_id)),
        )
        .exec(txn)
        .await?;

    Ok(path_cell)
}

#[cfg(test)]
mod tests {
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

    use crate::database;

    #[tokio::test]
    #[test_log::test]
    #[ignore] // DATABASE_URL が必要なため
    async fn test_get_adjacent_unused_extendable_pillar_and_path_to_wall()
    -> Result<(), Box<dyn std::error::Error>> {
        // ロギングの初期化
        let _ = env_logger::builder().is_test(true).try_init();

        // DB接続を確立
        info!("Attempting to connect to database...");
        let db = match crate::database::connector::establish_connection(None).await {
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
                eprintln!(
                    "  3. Example: DATABASE_URL=mysql://user:password@localhost/maze_maker_db\n"
                );
                return Err(format!(
                    "Database connection failed. See above for details. Error: {:?}",
                    e
                )
                .into());
            }
        };

        // 1. initialize_db()で、5x5でDBを初期化する。
        info!("Step 1: Initializing database with 5x5 grid");
        crate::database::initializer::initialize_db(&db, 5, 5).await?;

        // 2. MAZE_CELL_STATUS_VIEWの内容をinfo!でログ出力する。
        info!("Step 2: Logging MAZE_CELL_STATUS_VIEW after initialization");
        let cell_statuses = crate::database::entities::maze_cell_status_view::Entity::find()
            .all(db.as_ref())
            .await?;
        for status in &cell_statuses {
            info!(
                "  Cell status: x={}, y={}, type={}, owner_thread_id={:?}",
                status.x, status.y, status.cell_type, status.cell_owner_thread_id
            );
        }

        // 3. select_random_start_point_from_db()で、開始点を取得する。(開始点A)
        info!("Step 3: Selecting random start point A");
        let tid_a = crate::maze::maze_thread_identifier::MazeThreadIdentifier::new();
        let start_point_a =
            crate::maze::independent_transaction_routines::select_random_start_point_from_db(
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
        let cell_statuses = crate::database::entities::maze_cell_status_view::Entity::find()
            .all(db.as_ref())
            .await?;
        for status in &cell_statuses {
            info!(
                "  Cell status: x={}, y={}, type={}, owner_thread_id={:?}",
                status.x, status.y, status.cell_type, status.cell_owner_thread_id
            );
        }

        // 5. USED_START_POINTS_VIEWの内容をinfo!でログ出力する。
        info!("Step 5: Logging USED_START_POINTS_VIEW after selecting start point A");
        let used_start_points = crate::database::entities::used_start_points_view::Entity::find()
            .all(db.as_ref())
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
        let start_point_b = crate::maze::move_next::get_adjacent_unused_extendable_pillar(
            &txn1,
            &tid_a,
            &start_point_a,
        )
        .await?;
        txn1.commit().await?;

        // 7. MAZE_CELL_STATUS_VIEWの内容をinfo!でログ出力する。
        info!("Step 7: Logging MAZE_CELL_STATUS_VIEW after getting start point B");
        let cell_statuses = crate::database::entities::maze_cell_status_view::Entity::find()
            .all(db.as_ref())
            .await?;
        for status in &cell_statuses {
            info!(
                "  Cell status: x={}, y={}, type={}, owner_thread_id={:?}",
                status.x, status.y, status.cell_type, status.cell_owner_thread_id
            );
        }

        // 8. USED_START_POINTS_VIEWの内容をinfo!でログ出力する。
        info!("Step 8: Logging USED_START_POINTS_VIEW after getting start point B");
        let used_start_points = crate::database::entities::used_start_points_view::Entity::find()
            .all(db.as_ref())
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
            crate::maze::move_next::path_to_wall(&txn2, &tid_a, &start_point_a, &start_point_b)
                .await?;
        info!(
            "  Intermediate point (changed to WALL): ({}, {})",
            intermediate_point.x(),
            intermediate_point.y()
        );

        txn2.commit().await?;

        // 10. MAZE_CELL_STATUS_VIEWにて、中間点がWALLになっているいことを確認する。
        info!("Step 10: Verifying intermediate point is WALL in MAZE_CELL_STATUS_VIEW");
        let cell_statuses = crate::database::entities::maze_cell_status_view::Entity::find()
            .all(db.as_ref())
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
            crate::database::initializer::MazeCellTypeEnum::WALL.to_string(),
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
        let used_start_points = database::entities::used_start_points_view::Entity::find()
            .all(db.as_ref())
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
            crate::database::initializer::OutsideWallConnectTypeEnum::DIRECT_CONNECT.to_string(),
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
            crate::database::initializer::OutsideWallConnectTypeEnum::DIRECT_CONNECT.to_string(),
            "Start point B should have DIRECT_CONNECT status"
        );
        info!("  ✓ Start point B exists with DIRECT_CONNECT");

        info!("✓ All test assertions passed!");
        Ok(())
    }
}
