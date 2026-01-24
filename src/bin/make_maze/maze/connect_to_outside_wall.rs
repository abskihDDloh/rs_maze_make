use std::collections::{HashMap, HashSet};

use log::{debug, warn};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder};

use rs_maze_maker::common::database::entities::maze_cell_owner_view;
use rs_maze_maker::common::database::initializer::MazeCellTypeEnum;
use rs_maze_maker::common::database::initializer::OutsideWallConnectTypeEnum;
use rs_maze_maker::common::{maze_point::MazePoint, maze_thread_identifier::MazeThreadIdentifier};

use crate::maze::maze_thread_utility::get_used_cell_status;

/// 指定した迷路スレッドIDに属する柱セルに隣接する、かつ自分自身の柱セルではない柱セル、かつ外壁につながっている壁に属する柱セルの座標一覧を取得する。
/// # Arguments
/// * `txn` - データベーストランザクション参照
/// * `tid` - 迷路スレッド識別子参照
/// # Returns
/// * `Result<Vec<MazePoint>, Box<dyn std::error::Error>>` - 隣接する柱セルの座標一覧、またはエラー
pub async fn get_all_adjacent_not_myself_pillar(
    txn: &DatabaseTransaction,
    tid: &MazeThreadIdentifier,
) -> Result<HashMap<MazePoint, Vec<MazePoint>>, Box<dyn std::error::Error>> {
    let mut adjacent_walls_pillar_map: HashMap<MazePoint, Vec<MazePoint>> = HashMap::new();

    let my_pillar_cells: Vec<maze_cell_owner_view::Model> = maze_cell_owner_view::Entity::find()
        .filter(maze_cell_owner_view::Column::CellType.eq(MazeCellTypeEnum::PILLAR.to_string()))
        .filter(maze_cell_owner_view::Column::ThreadId.eq(tid.thread_id_as_str().to_string()))
        .filter(maze_cell_owner_view::Column::CreateUnixtime.eq(tid.unix_time()))
        .order_by_desc(maze_cell_owner_view::Column::CreateUnixtime)
        .all(txn)
        .await?;

    for pillar_cell in my_pillar_cells {
        let mut adjacent_wall_pillars: HashSet<MazePoint> = HashSet::new();
        let pillar_point = MazePoint::new(pillar_cell.x, pillar_cell.y);
        let adjacent_points = pillar_point.generate_adjacent_maze_points(2);
        for adj_point in adjacent_points {
            let adj_cell = match get_used_cell_status(txn, &adj_point).await {
                Ok(cell) => cell,
                Err(e) => {
                    warn!(
                        "Error getting used cell status for point {:?}, continuing. Error: {}",
                        adj_point, e
                    );
                    continue;
                } // get_used_cell_status()がエラーの場合は未使用セルか存在不明セルなのでcontinueする。
            };
            // 自分自身の柱セルではなく、かつCELL_TYPEがPILLARで、かつOUTSIDE_WALL_CONNECT_TYPEがNOT_CONNECTでない場合に隣接柱セルとして追加する。
            if (adj_point.x() != pillar_cell.x || adj_point.y() != pillar_cell.y)
                && adj_cell.cell_type == MazeCellTypeEnum::PILLAR.to_string()
                && adj_cell.outside_wall_connect_type
                    != OutsideWallConnectTypeEnum::NOT_CONNECT.to_string()
            {
                adjacent_wall_pillars.insert(adj_point);
            }
        }
        if !adjacent_wall_pillars.is_empty() {
            adjacent_walls_pillar_map
                .insert(pillar_point, adjacent_wall_pillars.into_iter().collect());
        }
    }
    debug!(
        "Adjacent wall pillars for thread {:?}: {:?}",
        tid, adjacent_walls_pillar_map
    );
    Ok(adjacent_walls_pillar_map)
}
