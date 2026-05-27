use std::collections::HashMap;
use std::pin::Pin;

use crate::{
    errors::DomainError,
    events::{DomainEvent, EventEnvelope},
    models::{
        api_key::ApiKey,
        feed::{EngagementStats, FeedEntry, PageParams, Paginated, UserSummary, ViewerContext},
        notification::Notification,
        remote_actor::RemoteActor,
        social::{Block, Boost, Follow, FollowState, Like},
        tag::Tag,
        thought::Thought,
        top_friend::TopFriend,
        user::{UpdateProfileInput, User},
    },
    value_objects::{
        ApiKeyId, Content, Email, NotificationId, PasswordHash, ThoughtId, UserId, Username,
    },
};
use async_trait::async_trait;
use bytes::Bytes;

pub type DataStream =
    Pin<Box<dyn futures::stream::Stream<Item = Result<Bytes, DomainError>> + Send>>;

#[async_trait]
pub trait MediaStore: Send + Sync {
    async fn put(&self, key: &str, data: DataStream) -> Result<(), DomainError>;
    async fn get(&self, key: &str) -> Result<DataStream, DomainError>;
    async fn delete(&self, key: &str) -> Result<(), DomainError>;
}

pub struct GeneratedToken {
    pub token: String,
    pub user_id: UserId,
}

#[async_trait]
pub trait AuthService: Send + Sync {
    fn generate_token(&self, user_id: &UserId) -> Result<GeneratedToken, DomainError>;
    fn validate_token(&self, token: &str) -> Result<UserId, DomainError>;
}

#[async_trait]
pub trait PasswordHasher: Send + Sync {
    async fn hash(&self, plain: &str) -> Result<PasswordHash, DomainError>;
    async fn verify(&self, plain: &str, hash: &PasswordHash) -> Result<bool, DomainError>;
}

#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, event: &DomainEvent) -> Result<(), DomainError>;
}

pub trait EventConsumer: Send + Sync {
    fn consume(&self) -> futures::stream::BoxStream<'_, Result<EventEnvelope, DomainError>>;
}

#[async_trait]
pub trait OutboxWriter: Send + Sync {
    async fn append(&self, event: &DomainEvent) -> Result<(), DomainError>;
}

#[async_trait]
pub trait UserReader: Send + Sync {
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, DomainError>;
    async fn find_by_username(&self, username: &Username) -> Result<Option<User>, DomainError>;
    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, DomainError>;
    async fn list_with_stats(&self) -> Result<Vec<UserSummary>, DomainError>;
    async fn count(&self) -> Result<i64, DomainError>;
    async fn list_paginated(&self, page: PageParams)
        -> Result<Paginated<UserSummary>, DomainError>;
    async fn find_by_ids(&self, ids: &[UserId]) -> Result<HashMap<UserId, User>, DomainError>;
}

#[async_trait]
pub trait UserWriter: Send + Sync {
    async fn save(&self, user: &User) -> Result<(), DomainError>;
    async fn update_profile(
        &self,
        user_id: &UserId,
        input: UpdateProfileInput,
    ) -> Result<(), DomainError>;
}

/// Combined supertrait — `AppState.users` stays `Arc<dyn UserRepository>`.
/// Blanket impl: any type implementing both sub-traits gets `UserRepository` for free.
pub trait UserRepository: UserReader + UserWriter {}
impl<T: UserReader + UserWriter> UserRepository for T {}

#[async_trait]
pub trait ThoughtRepository: Send + Sync {
    async fn save(&self, thought: &Thought) -> Result<(), DomainError>;
    async fn find_by_id(&self, id: &ThoughtId) -> Result<Option<Thought>, DomainError>;
    async fn delete(&self, id: &ThoughtId, user_id: &UserId) -> Result<(), DomainError>;
    async fn update_content(&self, id: &ThoughtId, content: &Content) -> Result<(), DomainError>;
    async fn get_thread(&self, id: &ThoughtId) -> Result<Vec<Thought>, DomainError>;
    async fn list_by_user(
        &self,
        user_id: &UserId,
        page: &PageParams,
    ) -> Result<Paginated<Thought>, DomainError>;
}

#[async_trait]
pub trait LikeRepository: Send + Sync {
    async fn save(&self, like: &Like) -> Result<(), DomainError>;
    async fn delete(&self, user_id: &UserId, thought_id: &ThoughtId) -> Result<(), DomainError>;
    async fn find(
        &self,
        user_id: &UserId,
        thought_id: &ThoughtId,
    ) -> Result<Option<Like>, DomainError>;
    async fn count_for_thought(&self, thought_id: &ThoughtId) -> Result<i64, DomainError>;
}

#[async_trait]
pub trait BoostRepository: Send + Sync {
    async fn save(&self, boost: &Boost) -> Result<(), DomainError>;
    async fn delete(&self, user_id: &UserId, thought_id: &ThoughtId) -> Result<(), DomainError>;
    async fn find(
        &self,
        user_id: &UserId,
        thought_id: &ThoughtId,
    ) -> Result<Option<Boost>, DomainError>;
    async fn count_for_thought(&self, thought_id: &ThoughtId) -> Result<i64, DomainError>;
}

#[async_trait]
pub trait EngagementRepository: Send + Sync {
    async fn get_for_thoughts(
        &self,
        thought_ids: &[ThoughtId],
        viewer_id: Option<&UserId>,
    ) -> Result<HashMap<ThoughtId, (EngagementStats, Option<ViewerContext>)>, DomainError>;
}

#[async_trait]
pub trait FollowRepository: Send + Sync {
    async fn save(&self, follow: &Follow) -> Result<(), DomainError>;
    async fn delete(&self, follower_id: &UserId, following_id: &UserId) -> Result<(), DomainError>;
    async fn find(
        &self,
        follower_id: &UserId,
        following_id: &UserId,
    ) -> Result<Option<Follow>, DomainError>;
    async fn update_state(
        &self,
        follower_id: &UserId,
        following_id: &UserId,
        state: &FollowState,
    ) -> Result<(), DomainError>;
    async fn list_followers(
        &self,
        user_id: &UserId,
        page: &PageParams,
    ) -> Result<Paginated<User>, DomainError>;
    async fn list_following(
        &self,
        user_id: &UserId,
        page: &PageParams,
    ) -> Result<Paginated<User>, DomainError>;
    async fn get_accepted_following_ids(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<UserId>, DomainError>;
}

#[async_trait]
pub trait BlockRepository: Send + Sync {
    async fn save(&self, block: &Block) -> Result<(), DomainError>;
    async fn delete(&self, blocker_id: &UserId, blocked_id: &UserId) -> Result<(), DomainError>;
    async fn exists(&self, blocker_id: &UserId, blocked_id: &UserId) -> Result<bool, DomainError>;
}

#[async_trait]
pub trait TagRepository: Send + Sync {
    async fn find_or_create(&self, name: &str) -> Result<Tag, DomainError>;
    async fn attach_to_thought(
        &self,
        thought_id: &ThoughtId,
        tag_id: i32,
    ) -> Result<(), DomainError>;
    async fn detach_from_thought(&self, thought_id: &ThoughtId) -> Result<(), DomainError>;
    async fn list_for_thought(&self, thought_id: &ThoughtId) -> Result<Vec<Tag>, DomainError>;
    async fn list_thoughts_by_tag(
        &self,
        tag_name: &str,
        page: &PageParams,
    ) -> Result<Paginated<Thought>, DomainError>;
    /// Returns (tag_name, thought_count) pairs ordered by usage, most popular first.
    async fn popular_tags(&self, limit: usize) -> Result<Vec<(String, i64)>, DomainError>;
}

#[async_trait]
pub trait ApiKeyRepository: Send + Sync {
    async fn save(&self, key: &ApiKey) -> Result<(), DomainError>;
    async fn find_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>, DomainError>;
    async fn list_for_user(&self, user_id: &UserId) -> Result<Vec<ApiKey>, DomainError>;
    async fn delete(&self, id: &ApiKeyId, user_id: &UserId) -> Result<(), DomainError>;
}

#[async_trait]
pub trait ApiKeyService: Send + Sync {
    async fn validate_key(&self, raw_key: &str) -> Result<Option<UserId>, DomainError>;
}

#[async_trait]
pub trait TopFriendRepository: Send + Sync {
    async fn set_top_friends(
        &self,
        user_id: &UserId,
        friends: Vec<(UserId, i16)>,
    ) -> Result<(), DomainError>;
    async fn list_for_user(&self, user_id: &UserId) -> Result<Vec<(TopFriend, User)>, DomainError>;
}

#[async_trait]
pub trait NotificationRepository: Send + Sync {
    async fn save(&self, n: &Notification) -> Result<(), DomainError>;
    async fn list_for_user(
        &self,
        user_id: &UserId,
        page: &PageParams,
    ) -> Result<Paginated<Notification>, DomainError>;
    async fn count_unread(&self, user_id: &UserId) -> Result<u64, DomainError>;
    async fn mark_read(&self, id: &NotificationId, user_id: &UserId) -> Result<(), DomainError>;
    async fn mark_all_read(&self, user_id: &UserId) -> Result<(), DomainError>;
}

#[async_trait]
pub trait RemoteActorRepository: Send + Sync {
    async fn upsert(&self, actor: &RemoteActor) -> Result<(), DomainError>;
    async fn find_by_url(&self, url: &str) -> Result<Option<RemoteActor>, DomainError>;
}

#[async_trait]
pub trait RemoteActorConnectionRepository: Send + Sync {
    async fn upsert_connections(
        &self,
        actor_url: &str,
        connection_type: &str,
        page: u32,
        actors: &[crate::models::actor_connection_summary::ActorConnectionSummary],
    ) -> Result<(), DomainError>;

    async fn list_connections(
        &self,
        actor_url: &str,
        connection_type: &str,
        page: u32,
    ) -> Result<Vec<crate::models::actor_connection_summary::ActorConnectionSummary>, DomainError>;

    async fn connection_page_age(
        &self,
        actor_url: &str,
        connection_type: &str,
        page: u32,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, DomainError>;
}

#[async_trait]
pub trait FederationLookupPort: Send + Sync {
    async fn lookup_actor(&self, handle: &str) -> Result<RemoteActor, DomainError>;
    async fn actor_json(&self, user_id: &UserId) -> Result<String, DomainError>;
    async fn followers_collection_json(
        &self,
        user_id: &UserId,
        page: Option<u32>,
    ) -> Result<String, DomainError>;
    async fn following_collection_json(
        &self,
        user_id: &UserId,
        page: Option<u32>,
    ) -> Result<String, DomainError>;
}

#[async_trait]
pub trait FederationFollowPort: Send + Sync {
    async fn follow_remote(&self, local_user_id: &UserId, handle: &str) -> Result<(), DomainError>;
    async fn unfollow_remote(
        &self,
        local_user_id: &UserId,
        handle: &str,
    ) -> Result<(), DomainError>;
    async fn get_remote_following(&self, user_id: &UserId)
        -> Result<Vec<RemoteActor>, DomainError>;
    async fn broadcast_move(
        &self,
        user_id: &UserId,
        new_actor_url: url::Url,
    ) -> Result<(), DomainError>;
}

#[async_trait]
pub trait FederationFollowRequestPort: Send + Sync {
    async fn get_pending_followers(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<RemoteActor>, DomainError>;
    async fn accept_follow_request(
        &self,
        user_id: &UserId,
        actor_url: &str,
    ) -> Result<(), DomainError>;
    async fn reject_follow_request(
        &self,
        user_id: &UserId,
        actor_url: &str,
    ) -> Result<(), DomainError>;
    async fn get_remote_followers(&self, user_id: &UserId)
        -> Result<Vec<RemoteActor>, DomainError>;
    async fn remove_remote_follower(
        &self,
        user_id: &UserId,
        actor_url: &str,
    ) -> Result<(), DomainError>;
}

#[async_trait]
pub trait FederationFetchPort: Send + Sync {
    async fn fetch_outbox_page(
        &self,
        outbox_url: &str,
        page: u32,
    ) -> Result<Vec<crate::models::remote_note::RemoteNote>, DomainError>;
    async fn fetch_actor_urls_from_collection(
        &self,
        collection_url: &str,
    ) -> Result<Vec<String>, DomainError>;
    async fn resolve_actor_profiles(
        &self,
        urls: Vec<String>,
    ) -> Vec<crate::models::actor_connection_summary::ActorConnectionSummary>;
}

pub trait FederationActionPort:
    FederationLookupPort + FederationFollowPort + FederationFollowRequestPort + FederationFetchPort
{
}
impl<
        T: FederationLookupPort
            + FederationFollowPort
            + FederationFollowRequestPort
            + FederationFetchPort,
    > FederationActionPort for T
{
}

#[derive(Debug, Clone)]
pub enum FeedScope {
    Home { following_ids: Vec<UserId> },
    Public,
    Tag { tag_name: String },
    User { user_id: UserId },
    Search { query: String },
}

#[derive(Debug, Clone)]
pub struct FeedQuery {
    pub scope: FeedScope,
    pub page: PageParams,
    pub viewer_id: Option<UserId>,
}

impl FeedQuery {
    pub fn home(viewer_id: UserId, following_ids: Vec<UserId>, page: PageParams) -> Self {
        Self {
            scope: FeedScope::Home { following_ids },
            page,
            viewer_id: Some(viewer_id),
        }
    }
    pub fn public(page: PageParams, viewer_id: Option<UserId>) -> Self {
        Self {
            scope: FeedScope::Public,
            page,
            viewer_id,
        }
    }
    pub fn tag(tag_name: impl Into<String>, page: PageParams, viewer_id: Option<UserId>) -> Self {
        Self {
            scope: FeedScope::Tag {
                tag_name: tag_name.into(),
            },
            page,
            viewer_id,
        }
    }
    pub fn user(user_id: UserId, page: PageParams, viewer_id: Option<UserId>) -> Self {
        Self {
            scope: FeedScope::User { user_id },
            page,
            viewer_id,
        }
    }
    pub fn search(query: impl Into<String>, page: PageParams, viewer_id: Option<UserId>) -> Self {
        Self {
            scope: FeedScope::Search {
                query: query.into(),
            },
            page,
            viewer_id,
        }
    }
}

#[async_trait]
pub trait FeedRepository: Send + Sync {
    async fn query(&self, q: &FeedQuery) -> Result<Paginated<FeedEntry>, DomainError>;
}

#[async_trait]
pub trait SearchPort: Send + Sync {
    /// Full-text search over public thoughts, ranked by trigram similarity.
    async fn search_thoughts(
        &self,
        query: &str,
        page: &PageParams,
        viewer_id: Option<&UserId>,
    ) -> Result<Paginated<FeedEntry>, DomainError>;

    /// Search users by username or display_name, ranked by trigram similarity.
    async fn search_users(
        &self,
        query: &str,
        page: &PageParams,
    ) -> Result<Paginated<User>, DomainError>;
}

#[async_trait]
pub trait FederationSchedulerPort: Send + Sync {
    async fn schedule_actor_posts_fetch(
        &self,
        actor_ap_url: &str,
        outbox_url: &str,
    ) -> Result<(), DomainError>;

    async fn schedule_connections_fetch(
        &self,
        actor_ap_url: &str,
        collection_url: &str,
        connection_type: &str,
        page: u32,
    ) -> Result<(), DomainError>;
}
