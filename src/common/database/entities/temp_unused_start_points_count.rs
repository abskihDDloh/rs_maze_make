use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "TEMP_UNUSED_START_POINTS_COUNT")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[sea_orm(column_name = "UNUSED_START_POINTS")]
    pub unused_start_points: u64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
