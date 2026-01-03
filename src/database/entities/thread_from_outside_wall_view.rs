//! `SeaORM` Entity for THREAD_FROM_OUTSIDE_WALL_VIEW

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "THREAD_FROM_OUTSIDE_WALL_VIEW")]
pub struct Model {
    #[sea_orm(primary_key, column_name = "TID", auto_increment = false)]
    pub tid: u64,
    #[sea_orm(column_name = "THREAD_ID")]
    pub thread_id: String,
    #[sea_orm(column_name = "CREATE_UNIXTIME")]
    pub create_unixtime: i64,
    #[sea_orm(column_name = "START_CELL_ID")]
    pub start_cell_id: u64,
    #[sea_orm(column_name = "START_X")]
    pub start_x: u64,
    #[sea_orm(column_name = "START_Y")]
    pub start_y: u64,
    #[sea_orm(column_name = "OUTSIDE_WALL_CONNECT_TYPE")]
    pub outside_wall_connect_type: String,
    #[sea_orm(column_name = "CELL_TYPE")]
    pub cell_type: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
