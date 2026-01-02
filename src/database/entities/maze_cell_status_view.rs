use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "MAZE_CELL_STATUS_VIEW")]
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
    pub cell_owner_thread_id: Option<u64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
