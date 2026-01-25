use std::collections::{HashMap, HashSet};
use std::process::id;

use log::{debug, info, warn};
use rs_maze_maker::common::database::entities::thread_list;
use sea_orm::TransactionTrait;
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder};

use rs_maze_maker::common::database::entities::maze_cell_owner_view;
use rs_maze_maker::common::database::initializer::MazeCellTypeEnum;
use rs_maze_maker::common::database::initializer::OutsideWallConnectTypeEnum;
use rs_maze_maker::common::maze_point::MazePoint;

use crate::maze::maze_thread_utility::get_used_cell_status;
use crate::maze::move_next::path_to_wall;

/// THREAD_LISTからOUTSIDE_WALL_CONNECT_TYPE=NOT_CONNECTであるレコードを取得する。
pub async fn get_not_connect_thread_records(
    txn: &DatabaseTransaction,
) -> Result<Vec<thread_list::Model>, Box<dyn std::error::Error>> {
    let thread_records: Vec<thread_list::Model> = thread_list::Entity::find()
        .filter(
            thread_list::Column::OutsideWallConnectType
                .eq(OutsideWallConnectTypeEnum::NOT_CONNECT.to_string()),
        )
        .all(txn)
        .await?;
    Ok(thread_records)
}

/// 指定した迷路スレッドに属する柱セルに隣接する、かつ自分自身の柱セルではない柱セル、かつ外壁につながっている壁に属する柱セルの座標一覧を取得する。
/// # Arguments
/// * `db` - データベース接続参照
/// * `my_thread_id` - 自分の迷路スレッドID文字列
/// * `my_create_unixtime` - 自分の迷路スレッド生成UNIXタイムスタンプ
/// # Returns
/// * `HashMap<MazePoint, MazePoint>` - キーが隣接する柱セルの座標、値が自分自身の柱セルの座標のマップ。
async fn get_all_adjacent_not_myself_pillar(
    db: &sea_orm::DbConn,
    my_thread_id: String,
    my_create_unixtime: i64,
) -> Result<HashMap<MazePoint, MazePoint>, Box<dyn std::error::Error>> {
    let my_thread_id = my_thread_id.clone();
    let mut adjacent_walls_pillar_map: HashMap<MazePoint, MazePoint> = HashMap::new();

    let txn_get_my_pillars = db.begin().await?;
    let my_pillars: Vec<maze_cell_owner_view::Model> = maze_cell_owner_view::Entity::find()
        .filter(maze_cell_owner_view::Column::CellType.eq(MazeCellTypeEnum::PILLAR.to_string()))
        .filter(maze_cell_owner_view::Column::ThreadId.eq(my_thread_id.clone()))
        .filter(maze_cell_owner_view::Column::CreateUnixtime.eq(my_create_unixtime))
        .order_by_desc(maze_cell_owner_view::Column::CreateUnixtime)
        .all(&txn_get_my_pillars)
        .await?;
    txn_get_my_pillars.commit().await?;

    for pillar_cell in my_pillars {
        let pillar_point = MazePoint::new(pillar_cell.x, pillar_cell.y);
        let adjacent_points = pillar_point.generate_adjacent_maze_points(2);
        for adj_point in adjacent_points {
            let txn_get_adj_pillar = db.begin().await?;
            let adj_cell = match get_used_cell_status(&txn_get_adj_pillar, &adj_point).await {
                Ok(cell) => cell,
                Err(e) => {
                    warn!(
                        "Error getting used cell status for point {:?}, continuing. Error: {}",
                        adj_point, e
                    );
                    continue;
                } // get_used_cell_status()がエラーの場合は未使用セルか存在不明セルなのでcontinueする。
            };
            txn_get_adj_pillar.commit().await?;
            // 自分自身の柱セルではなく、かつCELL_TYPEがPILLARで、かつOUTSIDE_WALL_CONNECT_TYPEがNOT_CONNECTでない場合に隣接柱セルとして追加する。
            if (adj_point.x() != pillar_cell.x || adj_point.y() != pillar_cell.y)
                && adj_cell.cell_type == MazeCellTypeEnum::PILLAR.to_string()
                && adj_cell.outside_wall_connect_type
                    != OutsideWallConnectTypeEnum::NOT_CONNECT.to_string()
            {
                adjacent_walls_pillar_map.insert(adj_point, pillar_point);
            }
        }
    }
    debug!(
        "Adjacent wall pillars for thread {:?} {:?}: {:?}",
        my_thread_id, my_create_unixtime, adjacent_walls_pillar_map
    );
    Ok(adjacent_walls_pillar_map)
}

async fn connect_to_outside_wall(
    db: &sea_orm::DbConn,
    wall_thread: &thread_list::Model,
) -> Result<(), Box<dyn std::error::Error>> {
    let my_id = wall_thread.id;
    let my_thread_id = wall_thread.thread_id.clone();
    let my_create_unixtime = wall_thread.create_unixtime;
    let mut adjacent_pillars =
        get_all_adjacent_not_myself_pillar(db, my_thread_id.clone(), my_create_unixtime).await?;
    loop {
        if adjacent_pillars.is_empty() {
            warn!(
                "No available adjacent wall pillars to connect for thread ID: {}, create_unixtime: {}. Cannot connect to outside wall.",
                my_thread_id, my_create_unixtime
            );
            // 周りが全部NOT_CONNTCTの場合はここに来る可能性がある。
            break;
        }
        // 隣接する柱セルの中からランダムに1つ選択する。
        let mut rng = rand::rng();
        let selected_adj_point = {
            let keys: Vec<&MazePoint> = adjacent_pillars.keys().collect();
            let random_index = rand::Rng::random_range(&mut rng, 0..keys.len());
            *keys[random_index]
        };
        
        let selected_my_point_opt = adjacent_pillars.get(&selected_adj_point);
        let selected_my_point = match selected_my_point_opt {
            Some(point) => point.clone(),
            None => {
                // 隣接柱セルの取得に失敗した場合はループをリトライする。
                warn!(
                    "Selected adjacent point {:?} not found in adjacent pillars map, continuing.",
                    selected_adj_point
                );
                continue;
            }
        };
        // 選択した隣接柱セルを隣接リストから削除する。
        adjacent_pillars.remove(&selected_adj_point);
        debug!(
            "Thread ID: {}, create_unixtime: {} connecting pillar at {:?} to outside wall via adjacent pillar at {:?}",
            my_thread_id, my_create_unixtime, selected_my_point, selected_adj_point
        );

        // 選択した自分自身の柱セルを外壁に接続する。
        let txn_connect = db.begin().await?;
        let res_connect = path_to_wall(
            &txn_connect,
            my_thread_id.clone(),
            my_create_unixtime,
            &selected_my_point,
            &selected_adj_point,
        )
        .await;
        match res_connect {
            Ok(_) => {
                info!(
                    "Successfully connected pillar at {:?} to outside wall for thread ID: {}, create_unixtime: {}",
                    selected_my_point, my_thread_id, my_create_unixtime
                );
            }
            Err(e) => {
                warn!(
                    "Failed to connect pillar at {:?} to outside wall for thread ID: {}, create_unixtime: {}. Error: {}. Continuing with next adjacent pillar.",
                    selected_my_point, my_thread_id, my_create_unixtime, e
                );
                txn_connect.rollback().await?;
                continue;
            }
        }
        // THERAD_LISTのOUTSIDE_WALL_CONNECT_TYPEをNOT_CONNECTからINDIRECT_CONNECTに更新する。
        let mut active_thread = thread_list::ActiveModel {
            id: sea_orm::ActiveValue::unchanged(my_id),
            thread_id: sea_orm::ActiveValue::NotSet,
            create_unixtime: sea_orm::ActiveValue::NotSet,
            outside_wall_connect_type: sea_orm::ActiveValue::Set(
                OutsideWallConnectTypeEnum::INDIRECT_CONNECT.to_string(),
            ),
            start_cell: sea_orm::ActiveValue::NotSet,
        };
        let _update_result = thread_list::Entity::update(active_thread)
            .filter(thread_list::Column::Id.eq(my_id))
            .exec(&txn_connect)
            .await?;
        txn_connect.commit().await?;
        break;
    }
    Ok(())
}

pub async fn connect_all_not_connected_threads_to_outside_wall(
    db: &sea_orm::DbConn,
) -> Result<(), Box<dyn std::error::Error>> {
    loop{
    let txn_get_targets = db.begin().await?;
    let not_connected_threads = get_not_connect_thread_records(&txn_get_targets).await?;
    txn_get_targets.commit().await?;
    // NOT_CONNECT状態の迷路スレッドが存在しない場合は処理を終了する。
    if not_connected_threads.is_empty() {
        info!("All maze threads are already connected to outside wall. No action needed.");
        break;
    }
    let local_set = tokio::task::LocalSet::new();
    local_set
        .run_until(async {
            let mut handles = vec![];
            for rec in not_connected_threads {
                let db_clone = db.clone();
                let handle = tokio::task::spawn_local(async move {
                    connect_to_outside_wall(&db_clone, &rec).await
                });
                handles.push(handle);
            }
            // すべてのタスクの完了を待つ
            for handle in handles {
                let result = handle.await;
                // 利用可能な開始点がなくなった時点で必ずエラーになる。
                match result {
                    Ok(_) => info!("Maze finalize task completed successfully."),
                    Err(e) => warn!("Maze finalize task failed: {}", e),
                }
            }
        })
        .await;
    }
    Ok(())
}
