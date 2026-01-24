use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "MAZE_CELL_OWNER_VIEW")]
pub struct Model {
    #[sea_orm(primary_key, column_name = "CELL_ID")]
    pub cell_id: u64,
    #[sea_orm(column_name = "X")]
    pub x: u64,
    #[sea_orm(column_name = "Y")]
    pub y: u64,
    #[sea_orm(column_name = "CELL_TYPE")]
    pub cell_type: String,
    #[sea_orm(column_name = "CELL_OWNER_THREAD_ID")]
    pub cell_owner_thread_id: u64,
    #[sea_orm(column_name = "THREAD_ID")]
    pub thread_id: String,
    #[sea_orm(column_name = "CREATE_UNIXTIME")]
    pub create_unixtime: i64,
    #[sea_orm(column_name = "OUTSIDE_WALL_CONNECT_TYPE")]
    pub outside_wall_connect_type: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
