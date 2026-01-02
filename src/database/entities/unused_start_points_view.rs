use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, DeriveEntityModel, Eq, PartialEq)]
#[sea_orm(table_name = "UNUSED_START_POINTS_VIEW")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub x: u64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub y: u64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
