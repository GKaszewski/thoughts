use models::{
    domains::{thought, user},
    schemas::thought::ThoughtWithAuthor,
};
use sea_orm::{
    prelude::{Expr, Uuid},
    DatabaseConnection, DbErr, EntityTrait, JoinType, QueryFilter, QuerySelect, RelationTrait,
    Value,
};

use crate::persistence::follow;

fn is_visible(
    author_id: Uuid,
    viewer_id: Option<Uuid>,
    friend_ids: &[Uuid],
    visibility: &thought::Visibility,
) -> bool {
    match visibility {
        thought::Visibility::Public => true,
        thought::Visibility::Private => viewer_id.map_or(false, |v| v == author_id),
        thought::Visibility::FriendsOnly => {
            viewer_id.map_or(false, |v| v == author_id || friend_ids.contains(&author_id))
        }
    }
}

pub async fn search_thoughts(
    db: &DatabaseConnection,
    query: &str,
    viewer_id: Option<Uuid>,
) -> Result<Vec<ThoughtWithAuthor>, DbErr> {
    let mut friend_ids = Vec::new();
    if let Some(viewer) = viewer_id {
        friend_ids = follow::get_friend_ids(db, viewer).await?;
    }

    // We must join with the user table to get the author's username
    let thoughts_with_authors = thought::Entity::find()
        .column_as(user::Column::Username, "author_username")
        .join(JoinType::InnerJoin, thought::Relation::User.def())
        .filter(Expr::cust_with_values(
            "thought.search_document @@ websearch_to_tsquery('english', $1)",
            [Value::from(query)],
        ))
        .into_model::<ThoughtWithAuthor>() // Convert directly in the query
        .all(db)
        .await?;

    // Apply visibility filtering in Rust after the search
    Ok(thoughts_with_authors
        .into_iter()
        .filter(|t| is_visible(t.author_id, viewer_id, &friend_ids, &t.visibility))
        .collect())
}

pub async fn search_users(db: &DatabaseConnection, query: &str) -> Result<Vec<user::Model>, DbErr> {
    user::Entity::find()
        .filter(Expr::cust_with_values(
            "\"user\".search_document @@ websearch_to_tsquery('english', $1)",
            [Value::from(query)],
        ))
        .all(db)
        .await
}
