use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
    Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, ToSchema,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "thought_visibility")]
pub enum Visibility {
    #[sea_orm(string_value = "public")]
    Public,
    #[sea_orm(string_value = "friends_only")]
    FriendsOnly,
    #[sea_orm(string_value = "private")]
    Private,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "thought")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub author_id: Uuid,
    pub content: String,
    pub reply_to_id: Option<Uuid>,
    pub visibility: Visibility,
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(column_type = "custom(\"tsvector\")", nullable, ignore)]
    pub search_document: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::AuthorId",
        to = "super::user::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    User,

    #[sea_orm(has_many = "super::thought_tag::Entity")]
    ThoughtTag,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::tag::Entity> for Entity {
    fn to() -> RelationDef {
        super::thought_tag::Relation::Tag.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::thought_tag::Relation::Thought.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
