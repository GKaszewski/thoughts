use sea_orm::{
    prelude::Uuid, sea_query::SimpleExpr, ActiveModelTrait, ColumnTrait, Condition, DbConn, DbErr,
    EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect, RelationTrait, Set,
    TransactionTrait,
};

use models::{
    domains::{tag, thought, thought_tag, user},
    params::thought::CreateThoughtParams,
    schemas::thought::ThoughtWithAuthor,
};

use crate::{
    error::UserError,
    persistence::{
        follow,
        tag::{find_or_create_tags, link_tags_to_thought, parse_hashtags},
    },
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
        visibility: Set(params.visibility.unwrap_or(thought::Visibility::Public)),
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
    viewer_id: Option<Uuid>,
) -> Result<Vec<ThoughtWithAuthor>, DbErr> {
    let mut friend_ids = vec![];
    if let Some(viewer) = viewer_id {
        friend_ids = follow::get_friend_ids(db, viewer).await?;
    }

    thought::Entity::find()
        .select_only()
        .column(thought::Column::Id)
        .column(thought::Column::Content)
        .column(thought::Column::ReplyToId)
        .column(thought::Column::CreatedAt)
        .column(thought::Column::AuthorId)
        .column(thought::Column::Visibility)
        .column_as(user::Column::Username, "author_username")
        .join(JoinType::InnerJoin, thought::Relation::User.def())
        .filter(apply_visibility_filter(user_id, viewer_id, &friend_ids))
        .filter(thought::Column::AuthorId.eq(user_id))
        .order_by_desc(thought::Column::CreatedAt)
        .into_model::<ThoughtWithAuthor>()
        .all(db)
        .await
}

pub async fn get_feed_for_user(
    db: &DbConn,
    following_ids: Vec<Uuid>,
    viewer_id: Option<Uuid>,
) -> Result<Vec<ThoughtWithAuthor>, UserError> {
    if following_ids.is_empty() {
        return Ok(vec![]);
    }

    let mut friend_ids = vec![];
    if let Some(viewer) = viewer_id {
        friend_ids = follow::get_friend_ids(db, viewer)
            .await
            .map_err(|e| UserError::Internal(e.to_string()))?;
    }

    thought::Entity::find()
        .select_only()
        .column(thought::Column::Id)
        .column(thought::Column::Content)
        .column(thought::Column::ReplyToId)
        .column(thought::Column::CreatedAt)
        .column(thought::Column::Visibility)
        .column(thought::Column::AuthorId)
        .column_as(user::Column::Username, "author_username")
        .join(JoinType::InnerJoin, thought::Relation::User.def())
        .filter(
            Condition::any().add(following_ids.iter().fold(
                Condition::all(),
                |cond, &author_id| {
                    cond.add(apply_visibility_filter(author_id, viewer_id, &friend_ids))
                },
            )),
        )
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
    viewer_id: Option<Uuid>,
) -> Result<Vec<ThoughtWithAuthor>, DbErr> {
    let mut friend_ids = Vec::new();
    if let Some(viewer) = viewer_id {
        friend_ids = follow::get_friend_ids(db, viewer).await?;
    }

    let thoughts = thought::Entity::find()
        .select_only()
        .column(thought::Column::Id)
        .column(thought::Column::Content)
        .column(thought::Column::ReplyToId)
        .column(thought::Column::CreatedAt)
        .column(thought::Column::AuthorId)
        .column(thought::Column::Visibility)
        .column_as(user::Column::Username, "author_username")
        .join(JoinType::InnerJoin, thought::Relation::User.def())
        .join(JoinType::InnerJoin, thought::Relation::ThoughtTag.def())
        .join(JoinType::InnerJoin, thought_tag::Relation::Tag.def())
        .filter(tag::Column::Name.eq(tag_name.to_lowercase()))
        .order_by_desc(thought::Column::CreatedAt)
        .into_model::<ThoughtWithAuthor>()
        .all(db)
        .await?;

    let visible_thoughts = thoughts
        .into_iter()
        .filter(|thought| {
            let mut condition = thought.visibility == thought::Visibility::Public;
            if let Some(viewer) = viewer_id {
                if thought.author_id == viewer {
                    condition = true;
                }
                if thought.visibility == thought::Visibility::FriendsOnly
                    && friend_ids.contains(&thought.author_id)
                {
                    condition = true;
                }
            }
            condition
        })
        .collect();

    Ok(visible_thoughts)
}

fn apply_visibility_filter(
    user_id: Uuid,
    viewer_id: Option<Uuid>,
    friend_ids: &[Uuid],
) -> SimpleExpr {
    let mut condition =
        Condition::any().add(thought::Column::Visibility.eq(thought::Visibility::Public));

    if let Some(viewer) = viewer_id {
        // Viewers can see their own thoughts of any visibility
        if user_id == viewer {
            condition = condition
                .add(thought::Column::Visibility.eq(thought::Visibility::FriendsOnly))
                .add(thought::Column::Visibility.eq(thought::Visibility::Private));
        }
        // If the thought's author is a friend of the viewer, they can see it
        else if !friend_ids.is_empty() && friend_ids.contains(&user_id) {
            condition =
                condition.add(thought::Column::Visibility.eq(thought::Visibility::FriendsOnly));
        }
    }
    condition.into()
}
