use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};

use crate::maze::{maze_point::MazePoint, maze_thread_identifier::MazeThreadIdentifier};

pub async fn select_my_thread_record_from_tx(
    txn: &DatabaseTransaction,
    tid: &MazeThreadIdentifier,
) -> Result<crate::database::entities::thread_list::Model, Box<dyn std::error::Error>> {
    let thread_record = crate::database::entities::thread_list::Entity::find()
        .filter(
            crate::database::entities::thread_list::Column::ThreadId
                .eq(tid.thread_id_as_str())
                .and(
                    crate::database::entities::thread_list::Column::CreateUnixtime
                        .eq(tid.unix_time()),
                ),
        )
        .one(txn)
        .await?
        .ok_or(format!("Thread record not found. tid:{:?}", tid))?;
    Ok(thread_record)
}

pub async fn get_all_unused_pillars(
    txn: &DatabaseTransaction,
) -> Result<
    Vec<crate::database::entities::unused_start_points_view::Model>,
    Box<dyn std::error::Error>,
> {
    // UNUSED_START_POINTS_VIEWを全件取得する。
    let unused_start_points: Vec<crate::database::entities::unused_start_points_view::Model> =
        crate::database::entities::unused_start_points_view::Entity::find()
            .all(txn)
            .await?;
    Ok(unused_start_points)
}

pub async fn check_unused_pillars(
    txn: &DatabaseTransaction,
    pillars: &Vec<MazePoint>,
) -> Result<
    Vec<crate::database::entities::unused_start_points_view::Model>,
    Box<dyn std::error::Error>,
> {
    let unused_points: Vec<crate::database::entities::unused_start_points_view::Model> =
        crate::database::entities::unused_start_points_view::Entity::find()
            .filter(
                crate::database::entities::unused_start_points_view::Column::X
                    .is_in(pillars.iter().map(|p| p.x()).collect::<Vec<u64>>())
                    .and(
                        crate::database::entities::unused_start_points_view::Column::Y
                            .is_in(pillars.iter().map(|p| p.y()).collect::<Vec<u64>>()),
                    ),
            )
            .all(txn)
            .await?;
    Ok(unused_points)
}

pub async fn get_cell_status(
    txn: &DatabaseTransaction,
    cell: &MazePoint,
) -> Result<Vec<crate::database::entities::maze_cell_status_view::Model>, Box<dyn std::error::Error>>
{
    let cell_status: Vec<crate::database::entities::maze_cell_status_view::Model> =
        crate::database::entities::maze_cell_status_view::Entity::find()
            .filter(
                crate::database::entities::maze_cell_status_view::Column::X
                    .eq(cell.x())
                    .and(crate::database::entities::maze_cell_status_view::Column::Y.eq(cell.y())),
            )
            .all(txn)
            .await?;
    Ok(cell_status)
}

pub async fn is_this_thread_from_outside_wall(
    txn: &DatabaseTransaction,
    tid: &MazeThreadIdentifier,
) -> Result<bool, Box<dyn std::error::Error>> {
    // THREAD_FROM_OUTSIDE_WALL_VIEWから、MazeThreadIdentifierの内容に当てはまるレコードを取得する。
    let thread_from_outside_wall_view_records: Vec<
        crate::database::entities::thread_from_outside_wall_view::Model,
    > = crate::database::entities::thread_from_outside_wall_view::Entity::find()
        .filter(
            crate::database::entities::thread_from_outside_wall_view::Column::Tid
                .eq(tid.thread_id_as_str()),
        )
        .all(txn)
        .await?;
    Ok(!thread_from_outside_wall_view_records.is_empty())
}

pub async fn is_point_outside_wall(txn: &DatabaseTransaction, pillar: &MazePoint) -> bool {
    // OUTSIDE_WALL_START_POINTS_VIEWに、pillarの内容に(X AND Y)が当てはまるレコードが存在するか確認する。
    let outside_wall_start_points: Vec<
        crate::database::entities::outside_wall_start_points_view::Model,
    > = crate::database::entities::outside_wall_start_points_view::Entity::find()
        .filter(
            crate::database::entities::outside_wall_start_points_view::Column::X
                .eq(pillar.x())
                .and(
                    crate::database::entities::outside_wall_start_points_view::Column::Y
                        .eq(pillar.y()),
                ),
        )
        .all(txn)
        .await
        .unwrap_or_default();
    !outside_wall_start_points.is_empty()
}
/// cell_id に当てはまるMAZE_FIELDのCELL_TYPEがPILLARかつCELL_OWNER_THREAD_IDがNullの場合にかぎり、CELL_OWNER_THREAD_IDをthread_idに更新する。
pub async fn get_pillar(
    txn: &DatabaseTransaction,
    cell_id: u64,
    thread_id: u64,
) -> Result<crate::database::entities::maze_field::Model, Box<dyn std::error::Error>> {
    let update_model = crate::database::entities::maze_field::ActiveModel {
        id: sea_orm::ActiveValue::Set(cell_id),
        cell_owner_thread_id: sea_orm::ActiveValue::Set(Some(thread_id)),
        ..Default::default()
    };
    let result = crate::database::entities::maze_field::Entity::update(update_model)
        .filter(
            crate::database::entities::maze_field::Column::Id
                .eq(cell_id)
                .and(
                    crate::database::entities::maze_field::Column::CellType
                        .eq(crate::database::initializer::MazeCellTypeEnum::PILLAR.to_string()),
                )
                .and(crate::database::entities::maze_field::Column::CellOwnerThreadId.is_null()),
        )
        .exec(txn)
        .await?;
    let updated_record = crate::database::entities::maze_field::Entity::find_by_id(cell_id)
        .one(txn)
        .await?
        .ok_or("Updated record not found.")?;
    Ok(updated_record)
}
