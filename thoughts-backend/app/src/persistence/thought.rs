use sea_orm::{
    prelude::Uuid, ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, JoinType,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait, Set, TransactionTrait,
};

use models::{
    domains::{tag, thought, thought_tag, user},
    params::thought::CreateThoughtParams,
    schemas::thought::ThoughtWithAuthor,
};

use crate::{
    error::UserError,
    persistence::tag::{find_or_create_tags, link_tags_to_thought, parse_hashtags},
};

pub async fn create_thought(
    db: &DbConn,
    author_id: Uuid,
    params: CreateThoughtParams,
) -> Result<thought::Model, DbErr> {
    let txn = db.begin().await?;

    let new_thought = thought::ActiveModel {
        author_id: Set(author_id),
        content: Set(params.content.clone()),
        reply_to_id: Set(params.reply_to_id),
        ..Default::default()
    }
    .insert(&txn)
    .await?;

    let tag_names = parse_hashtags(&params.content);
    if !tag_names.is_empty() {
        let tags = find_or_create_tags(&txn, tag_names).await?;
        link_tags_to_thought(&txn, new_thought.id, tags).await?;
    }

    txn.commit().await?;
    Ok(new_thought)
}

pub async fn get_thought(db: &DbConn, thought_id: Uuid) -> Result<Option<thought::Model>, DbErr> {
    thought::Entity::find_by_id(thought_id).one(db).await
}

pub async fn delete_thought(db: &DbConn, thought_id: Uuid) -> Result<(), DbErr> {
    thought::Entity::delete_by_id(thought_id).exec(db).await?;
    Ok(())
}

pub async fn get_thoughts_by_user(
    db: &DbConn,
    user_id: Uuid,
) -> Result<Vec<ThoughtWithAuthor>, DbErr> {
    thought::Entity::find()
        .select_only()
        .column(thought::Column::Id)
        .column(thought::Column::Content)
        .column(thought::Column::ReplyToId)
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
    following_ids: Vec<Uuid>,
) -> Result<Vec<ThoughtWithAuthor>, UserError> {
    if following_ids.is_empty() {
        return Ok(vec![]);
    }

    thought::Entity::find()
        .select_only()
        .column(thought::Column::Id)
        .column(thought::Column::Content)
        .column(thought::Column::ReplyToId)
        .column(thought::Column::CreatedAt)
        .column(thought::Column::AuthorId)
        .column_as(user::Column::Username, "author_username")
        .join(JoinType::InnerJoin, thought::Relation::User.def())
        .filter(thought::Column::AuthorId.is_in(following_ids))
        .order_by_desc(thought::Column::CreatedAt)
        .into_model::<ThoughtWithAuthor>()
        .all(db)
        .await
        .map_err(|e| UserError::Internal(e.to_string()))
}

pub async fn get_thoughts_by_tag_name(
    db: &DbConn,
    tag_name: &str,
) -> Result<Vec<ThoughtWithAuthor>, DbErr> {
    thought::Entity::find()
        .select_only()
        .column(thought::Column::Id)
        .column(thought::Column::Content)
        .column(thought::Column::ReplyToId)
        .column(thought::Column::CreatedAt)
        .column(thought::Column::AuthorId)
        .column_as(user::Column::Username, "author_username")
        .join(JoinType::InnerJoin, thought::Relation::User.def())
        .join(JoinType::InnerJoin, thought::Relation::ThoughtTag.def())
        .join(JoinType::InnerJoin, thought_tag::Relation::Tag.def())
        .filter(tag::Column::Name.eq(tag_name.to_lowercase()))
        .order_by_desc(thought::Column::CreatedAt)
        .into_model::<ThoughtWithAuthor>()
        .all(db)
        .await
}
