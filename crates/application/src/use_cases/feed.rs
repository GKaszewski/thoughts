use domain::{
    errors::DomainError,
    models::feed::{FeedEntry, PageParams, Paginated},
    ports::{FeedOptions, FeedQuery, FeedRepository, FeedRequest, FollowRepository},
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
