use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "MAZE_CELL_MIN_MAX_VIEW")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_name = "max_x")]
    pub max_x: u64,
    #[sea_orm(column_name = "min_x")]
    pub min_x: u64,
    #[sea_orm(column_name = "max_y")]
    pub max_y: u64,
    #[sea_orm(column_name = "min_y")]
    pub min_y: u64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
