use sea_orm::{
    prelude::Uuid, sea_query::SimpleExpr, ActiveModelTrait, ColumnTrait, Condition, DbConn, DbErr,
    EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect, RelationTrait, Set,
    TransactionTrait,
};

use models::{
    domains::{tag, thought, thought_tag, user},
    params::thought::CreateThoughtParams,
    schemas::thought::{ThoughtSchema, ThoughtThreadSchema, ThoughtWithAuthor},
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

    if new_thought.visibility == thought::Visibility::Public {
        let tag_names = parse_hashtags(&params.content);
        if !tag_names.is_empty() {
            let tags = find_or_create_tags(&txn, tag_names).await?;
            link_tags_to_thought(&txn, new_thought.id, tags).await?;
        }
    }

    txn.commit().await?;
    Ok(new_thought)
}

pub async fn get_thought(
    db: &DbConn,
    thought_id: Uuid,
    viewer_id: Option<Uuid>,
) -> Result<Option<thought::Model>, DbErr> {
    let thought = thought::Entity::find_by_id(thought_id).one(db).await?;

    match thought {
        Some(t) => {
            if t.visibility == thought::Visibility::Public {
                return Ok(Some(t));
            }

            if let Some(viewer) = viewer_id {
                if t.author_id == viewer {
                    return Ok(Some(t));
                }

                if t.visibility == thought::Visibility::FriendsOnly {
                    let author_friends = follow::get_friend_ids(db, t.author_id).await?;
                    if author_friends.contains(&viewer) {
                        return Ok(Some(t));
                    }
                }
            }

            Ok(None)
        }
        None => Ok(None),
    }
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
        .column_as(user::Column::DisplayName, "author_display_name")
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
        .column_as(user::Column::DisplayName, "author_display_name")
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

pub async fn get_feed_for_users_and_self(
    db: &DbConn,
    user_id: Uuid,
    following_ids: Vec<Uuid>,
) -> Result<Vec<ThoughtWithAuthor>, DbErr> {
    let mut authors_to_include = following_ids;
    authors_to_include.push(user_id);

    thought::Entity::find()
        .select_only()
        .column(thought::Column::Id)
        .column(thought::Column::Content)
        .column(thought::Column::ReplyToId)
        .column(thought::Column::CreatedAt)
        .column(thought::Column::Visibility)
        .column(thought::Column::AuthorId)
        .column_as(user::Column::Username, "author_username")
        .column_as(user::Column::DisplayName, "author_display_name")
        .join(JoinType::InnerJoin, thought::Relation::User.def())
        .filter(thought::Column::AuthorId.is_in(authors_to_include))
        .filter(
            Condition::any()
                .add(thought::Column::Visibility.eq(thought::Visibility::Public))
                .add(thought::Column::Visibility.eq(thought::Visibility::FriendsOnly)),
        )
        .order_by_desc(thought::Column::CreatedAt)
        .into_model::<ThoughtWithAuthor>()
        .all(db)
        .await
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
        .column_as(user::Column::DisplayName, "author_display_name")
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

pub fn apply_visibility_filter(
    user_id: Uuid,
    viewer_id: Option<Uuid>,
    friend_ids: &[Uuid],
) -> SimpleExpr {
    let mut condition =
        Condition::any().add(thought::Column::Visibility.eq(thought::Visibility::Public));

    if let Some(viewer) = viewer_id {
        if user_id == viewer {
            condition = condition
                .add(thought::Column::Visibility.eq(thought::Visibility::FriendsOnly))
                .add(thought::Column::Visibility.eq(thought::Visibility::Private));
        } else if !friend_ids.is_empty() && friend_ids.contains(&user_id) {
            condition =
                condition.add(thought::Column::Visibility.eq(thought::Visibility::FriendsOnly));
        }
    }
    condition.into()
}

pub async fn get_thought_with_replies(
    db: &DbConn,
    thought_id: Uuid,
    viewer_id: Option<Uuid>,
) -> Result<Option<ThoughtThreadSchema>, DbErr> {
    let root_thought = match get_thought(db, thought_id, viewer_id).await? {
        Some(t) => t,
        None => return Ok(None),
    };

    let mut all_thoughts_in_thread = vec![root_thought.clone()];
    let mut ids_to_fetch = vec![root_thought.id];
    let mut friend_ids = vec![];
    if let Some(viewer) = viewer_id {
        friend_ids = follow::get_friend_ids(db, viewer).await?;
    }

    while !ids_to_fetch.is_empty() {
        let replies = thought::Entity::find()
            .filter(thought::Column::ReplyToId.is_in(ids_to_fetch))
            .all(db)
            .await?;

        if replies.is_empty() {
            break;
        }

        ids_to_fetch = replies.iter().map(|r| r.id).collect();
        all_thoughts_in_thread.extend(replies);
    }

    let mut thought_schemas = vec![];
    for thought in all_thoughts_in_thread {
        if let Some(author) = user::Entity::find_by_id(thought.author_id).one(db).await? {
            let is_visible = match thought.visibility {
                thought::Visibility::Public => true,
                thought::Visibility::Private => viewer_id.map_or(false, |v| v == thought.author_id),
                thought::Visibility::FriendsOnly => viewer_id.map_or(false, |v| {
                    v == thought.author_id || friend_ids.contains(&thought.author_id)
                }),
            };

            if is_visible {
                thought_schemas.push(ThoughtSchema::from_models(&thought, &author));
            }
        }
    }

    fn build_thread(
        thought_id: Uuid,
        schemas_map: &std::collections::HashMap<Uuid, ThoughtSchema>,
        replies_map: &std::collections::HashMap<Uuid, Vec<Uuid>>,
    ) -> Option<ThoughtThreadSchema> {
        schemas_map.get(&thought_id).map(|thought_schema| {
            let replies = replies_map
                .get(&thought_id)
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|reply_id| build_thread(*reply_id, schemas_map, replies_map))
                .collect();

            ThoughtThreadSchema {
                id: thought_schema.id,
                author_username: thought_schema.author_username.clone(),
                author_display_name: thought_schema.author_display_name.clone(),
                content: thought_schema.content.clone(),
                visibility: thought_schema.visibility.clone(),
                reply_to_id: thought_schema.reply_to_id,
                created_at: thought_schema.created_at.clone(),
                replies,
            }
        })
    }

    let schemas_map: std::collections::HashMap<Uuid, ThoughtSchema> =
        thought_schemas.into_iter().map(|s| (s.id, s)).collect();

    let mut replies_map: std::collections::HashMap<Uuid, Vec<Uuid>> =
        std::collections::HashMap::new();
    for thought in schemas_map.values() {
        if let Some(parent_id) = thought.reply_to_id {
            if schemas_map.contains_key(&parent_id) {
                replies_map.entry(parent_id).or_default().push(thought.id);
            }
        }
    }

    Ok(build_thread(root_thought.id, &schemas_map, &replies_map))
}
