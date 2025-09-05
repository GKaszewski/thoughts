use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, JoinType, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait, Set,
};

use models::{
    domains::{thought, user},
    params::thought::CreateThoughtParams,
    schemas::thought::ThoughtWithAuthor,
};

use crate::error::UserError;

pub async fn create_thought(
    db: &DbConn,
    author_id: i32,
    params: CreateThoughtParams,
) -> Result<thought::Model, DbErr> {
    thought::ActiveModel {
        author_id: Set(author_id),
        content: Set(params.content),
        ..Default::default()
    }
    .insert(db)
    .await
}

pub async fn get_thought(db: &DbConn, thought_id: i32) -> Result<Option<thought::Model>, DbErr> {
    thought::Entity::find_by_id(thought_id).one(db).await
}

pub async fn delete_thought(db: &DbConn, thought_id: i32) -> Result<(), DbErr> {
    thought::Entity::delete_by_id(thought_id).exec(db).await?;
    Ok(())
}

pub async fn get_thoughts_by_user(
    db: &DbConn,
    user_id: i32,
) -> Result<Vec<ThoughtWithAuthor>, DbErr> {
    thought::Entity::find()
        .select_only()
        .column(thought::Column::Id)
        .column(thought::Column::Content)
        .column(thought::Column::CreatedAt)
        .column(thought::Column::AuthorId)
        .column_as(user::Column::Username, "author_username")
        .join(JoinType::InnerJoin, thought::Relation::User.def())
        .filter(thought::Column::AuthorId.eq(user_id))
        .order_by_desc(thought::Column::CreatedAt)
        .into_model::<ThoughtWithAuthor>()
        .all(db)
        .await
}

pub async fn get_feed_for_user(
    db: &DbConn,
    followed_ids: Vec<i32>,
) -> Result<Vec<ThoughtWithAuthor>, UserError> {
    if followed_ids.is_empty() {
        return Ok(vec![]);
    }

    thought::Entity::find()
        .select_only()
        .column(thought::Column::Id)
        .column(thought::Column::Content)
        .column(thought::Column::CreatedAt)
        .column(thought::Column::AuthorId)
        .column_as(user::Column::Username, "author_username")
        .join(JoinType::InnerJoin, thought::Relation::User.def())
        .filter(thought::Column::AuthorId.is_in(followed_ids))
        .order_by_desc(thought::Column::CreatedAt)
        .into_model::<ThoughtWithAuthor>()
        .all(db)
        .await
        .map_err(|e| UserError::Internal(e.to_string()))
}
