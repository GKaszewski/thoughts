use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "tag")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::thought_tag::Entity")]
    ThoughtTag,
}

impl Related<super::thought::Entity> for Entity {
    fn to() -> RelationDef {
        super::thought_tag::Relation::Thought.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::thought_tag::Relation::Tag.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
