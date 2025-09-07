use chrono::{Duration, Utc};
use models::domains::{tag, thought, thought_tag};
use sea_orm::{
    prelude::Expr, sea_query::Alias, sqlx::types::uuid, ColumnTrait, ConnectionTrait, DbErr,
    EntityTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait, Set,
};
use std::collections::HashSet;

pub fn parse_hashtags(content: &str) -> Vec<String> {
    content
        .split_whitespace()
        .filter_map(|word| {
            if word.starts_with('#') && word.len() > 1 {
                Some(word[1..].to_lowercase().to_string())
            } else {
                None
            }
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}

pub async fn find_or_create_tags<C>(db: &C, names: Vec<String>) -> Result<Vec<tag::Model>, DbErr>
where
    C: ConnectionTrait,
{
    if names.is_empty() {
        return Ok(vec![]);
    }
    let existing_tags = tag::Entity::find()
        .filter(tag::Column::Name.is_in(names.clone()))
        .all(db)
        .await?;

    let existing_names: HashSet<String> = existing_tags.iter().map(|t| t.name.clone()).collect();
    let new_names: Vec<String> = names
        .into_iter()
        .filter(|n| !existing_names.contains(n))
        .collect();

    if !new_names.is_empty() {
        let new_tags: Vec<tag::ActiveModel> = new_names
            .clone()
            .into_iter()
            .map(|name| tag::ActiveModel {
                name: Set(name),
                ..Default::default()
            })
            .collect();
        tag::Entity::insert_many(new_tags).exec(db).await?;
    }

    tag::Entity::find()
        .filter(
            tag::Column::Name.is_in(
                existing_names
                    .union(&new_names.into_iter().collect())
                    .cloned()
                    .collect::<Vec<_>>(),
            ),
        )
        .all(db)
        .await
}

pub async fn link_tags_to_thought<C>(
    db: &C,
    thought_id: uuid::Uuid,
    tags: Vec<tag::Model>,
) -> Result<(), DbErr>
where
    C: ConnectionTrait,
{
    if tags.is_empty() {
        return Ok(());
    }
    let links: Vec<thought_tag::ActiveModel> = tags
        .into_iter()
        .map(|tag| thought_tag::ActiveModel {
            thought_id: Set(thought_id),
            tag_id: Set(tag.id),
        })
        .collect();

    thought_tag::Entity::insert_many(links).exec(db).await?;
    Ok(())
}

pub async fn get_popular_tags<C>(db: &C) -> Result<Vec<String>, DbErr>
where
    C: ConnectionTrait,
{
    let seven_days_ago = Utc::now() - Duration::days(7);

    let popular_tags = tag::Entity::find()
        .select_only()
        .column(tag::Column::Name)
        .column_as(Expr::col((tag::Entity, tag::Column::Id)).count(), "count")
        .join(
            sea_orm::JoinType::InnerJoin,
            tag::Relation::ThoughtTag.def(),
        )
        .join(
            sea_orm::JoinType::InnerJoin,
            thought_tag::Relation::Thought.def(),
        )
        .filter(thought::Column::CreatedAt.gte(seven_days_ago))
        .filter(thought::Column::Visibility.eq(thought::Visibility::Public))
        .group_by(tag::Column::Name)
        .group_by(tag::Column::Id)
        .order_by_desc(Expr::col(Alias::new("count")))
        .order_by_asc(tag::Column::Name)
        .limit(10)
        .into_tuple::<(String, i64)>()
        .all(db)
        .await?;

    Ok(popular_tags.into_iter().map(|(name, _)| name).collect())
}
