use domain::{
    errors::DomainError,
    models::feed::{FeedEntry, PageParams, Paginated},
    ports::{FeedOptions, FeedQuery, FeedRepository, FeedRequest, FollowRepository, TagRepository},
    value_objects::UserId,
};

pub async fn get_home_feed(
    feed: &dyn FeedRepository,
    follows: &dyn FollowRepository,
    user_id: &UserId,
    page: PageParams,
    opts: FeedOptions,
) -> Result<Paginated<FeedEntry>, DomainError> {
    let mut following_ids = follows.get_accepted_following_ids(user_id).await?;
    following_ids.push(user_id.clone());
    feed.query(&FeedRequest {
        query: FeedQuery::home(user_id.clone(), following_ids, page),
        options: opts,
    })
    .await
}

pub async fn get_public_feed(
    feed: &dyn FeedRepository,
    viewer: Option<UserId>,
    page: PageParams,
    opts: FeedOptions,
) -> Result<Paginated<FeedEntry>, DomainError> {
    feed.query(&FeedRequest {
        query: FeedQuery::public(page, viewer),
        options: opts,
    })
    .await
}

pub async fn get_user_feed(
    feed: &dyn FeedRepository,
    user_id: UserId,
    page: PageParams,
    opts: FeedOptions,
    viewer: Option<UserId>,
) -> Result<Paginated<FeedEntry>, DomainError> {
    feed.query(&FeedRequest {
        query: FeedQuery::user(user_id, page, viewer),
        options: opts,
    })
    .await
}

pub async fn get_tag_feed(
    feed: &dyn FeedRepository,
    tag: &str,
    page: PageParams,
    opts: FeedOptions,
    viewer: Option<UserId>,
) -> Result<Paginated<FeedEntry>, DomainError> {
    feed.query(&FeedRequest {
        query: FeedQuery::tag(tag, page, viewer),
        options: opts,
    })
    .await
}

pub async fn get_popular_tags(
    tags: &dyn TagRepository,
    limit: usize,
) -> Result<Vec<(String, i64)>, DomainError> {
    tags.popular_tags(limit).await
}
