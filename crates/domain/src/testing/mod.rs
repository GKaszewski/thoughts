use crate::{
    errors::DomainError,
    events::DomainEvent,
    models::{
        api_key::ApiKey,
        feed::{FeedEntry, PageParams, Paginated, UserSummary},
        notification::Notification,
        remote_actor::RemoteActor,
        social::{Block, Boost, Follow, FollowState, Like},
        tag::Tag,
        thought::Thought,
        top_friend::TopFriend,
        user::{UpdateProfileInput, User},
    },
    ports::*,
    value_objects::{ApiKeyId, Content, Email, NotificationId, ThoughtId, UserId, Username},
};
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Default, Clone)]
pub struct TestStore {
    pub users: Arc<Mutex<Vec<User>>>,
    pub thoughts: Arc<Mutex<Vec<Thought>>>,
    pub likes: Arc<Mutex<Vec<Like>>>,
    pub boosts: Arc<Mutex<Vec<Boost>>>,
    pub follows: Arc<Mutex<Vec<Follow>>>,
    pub blocks: Arc<Mutex<Vec<Block>>>,
    pub tags: Arc<Mutex<Vec<Tag>>>,
    pub api_keys: Arc<Mutex<Vec<ApiKey>>>,
    pub top_friends: Arc<Mutex<Vec<TopFriend>>>,
    pub notifications: Arc<Mutex<Vec<Notification>>>,
    pub events: Arc<Mutex<Vec<DomainEvent>>>,
    /// AP URL → UserId for remote actors (used by find_remote_actor_id / intern_remote_actor)
    pub actor_ap_ids: Arc<Mutex<HashMap<String, UserId>>>,
    /// ThoughtId → AP object URL (used by get_thought_ap_id)
    pub thought_ap_ids: Arc<Mutex<HashMap<ThoughtId, String>>>,
    pub remote_following: Arc<Mutex<Vec<RemoteActor>>>,
    pub remote_followers: Arc<Mutex<Vec<RemoteActor>>>,
}

#[async_trait]
impl UserReader for TestStore {
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, DomainError> {
        Ok(self
            .users
            .lock()
            .unwrap()
            .iter()
            .find(|u| &u.id == id)
            .cloned())
    }
    async fn find_by_username(&self, username: &Username) -> Result<Option<User>, DomainError> {
        Ok(self
            .users
            .lock()
            .unwrap()
            .iter()
            .find(|u| u.username.as_str() == username.as_str())
            .cloned())
    }
    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, DomainError> {
        Ok(self
            .users
            .lock()
            .unwrap()
            .iter()
            .find(|u| u.email.as_str() == email.as_str())
            .cloned())
    }
    async fn list_with_stats(&self) -> Result<Vec<UserSummary>, DomainError> {
        Ok(vec![])
    }
    async fn count(&self) -> Result<i64, DomainError> {
        Ok(self
            .users
            .lock()
            .unwrap()
            .iter()
            .filter(|u| u.local)
            .count() as i64)
    }

    async fn list_paginated(
        &self,
        page: PageParams,
    ) -> Result<Paginated<UserSummary>, DomainError> {
        let all = self.list_with_stats().await?;
        let total = all.len() as i64;
        let start = page.offset() as usize;
        let items: Vec<UserSummary> = all
            .into_iter()
            .skip(start)
            .take(page.limit() as usize)
            .collect();
        Ok(Paginated {
            items,
            total,
            page: page.page,
            per_page: page.per_page,
        })
    }

    async fn find_by_ids(&self, ids: &[UserId]) -> Result<HashMap<UserId, User>, DomainError> {
        let g = self.users.lock().unwrap();
        let map = g
            .iter()
            .filter(|u| ids.contains(&u.id))
            .map(|u| (u.id.clone(), u.clone()))
            .collect();
        Ok(map)
    }
}

#[async_trait]
impl UserWriter for TestStore {
    async fn save(&self, user: &User) -> Result<(), DomainError> {
        let mut g = self.users.lock().unwrap();
        g.retain(|u| u.id != user.id);
        g.push(user.clone());
        Ok(())
    }
    async fn update_profile(
        &self,
        user_id: &UserId,
        input: UpdateProfileInput,
    ) -> Result<(), DomainError> {
        if let Some(u) = self
            .users
            .lock()
            .unwrap()
            .iter_mut()
            .find(|u| &u.id == user_id)
        {
            if let Some(v) = input.display_name {
                u.display_name = Some(v);
            }
            if let Some(v) = input.bio {
                u.bio = Some(v);
            }
            if let Some(v) = input.avatar_url {
                u.avatar_url = Some(v);
            }
            if let Some(v) = input.header_url {
                u.header_url = Some(v);
            }
            if let Some(v) = input.custom_css {
                u.custom_css = Some(v);
            }
        }
        Ok(())
    }

    async fn set_also_known_as(
        &self,
        _user_id: &UserId,
        _value: Option<String>,
    ) -> Result<(), DomainError> {
        Ok(())
    }
}

#[async_trait]
impl ThoughtRepository for TestStore {
    async fn save(&self, t: &Thought) -> Result<(), DomainError> {
        let mut g = self.thoughts.lock().unwrap();
        g.retain(|x| x.id != t.id);
        g.push(t.clone());
        Ok(())
    }
    async fn find_by_id(&self, id: &ThoughtId) -> Result<Option<Thought>, DomainError> {
        Ok(self
            .thoughts
            .lock()
            .unwrap()
            .iter()
            .find(|t| &t.id == id)
            .cloned())
    }
    async fn delete(&self, id: &ThoughtId, user_id: &UserId) -> Result<(), DomainError> {
        let mut g = self.thoughts.lock().unwrap();
        let before = g.len();
        g.retain(|t| !(&t.id == id && &t.user_id == user_id));
        if g.len() == before {
            return Err(DomainError::NotFound);
        }
        Ok(())
    }
    async fn update_content(&self, id: &ThoughtId, content: &Content) -> Result<(), DomainError> {
        if let Some(t) = self
            .thoughts
            .lock()
            .unwrap()
            .iter_mut()
            .find(|t| &t.id == id)
        {
            t.content = content.clone();
            t.updated_at = Some(Utc::now());
        }
        Ok(())
    }
    async fn get_thread(&self, id: &ThoughtId) -> Result<Vec<Thought>, DomainError> {
        Ok(self
            .thoughts
            .lock()
            .unwrap()
            .iter()
            .filter(|t| t.in_reply_to_id.as_ref() == Some(id) || &t.id == id)
            .cloned()
            .collect())
    }
    async fn list_by_user(
        &self,
        _user_id: &UserId,
        _page: &PageParams,
    ) -> Result<Paginated<Thought>, DomainError> {
        Ok(Paginated {
            items: vec![],
            total: 0,
            page: 1,
            per_page: 20,
        })
    }
}

#[async_trait]
impl LikeRepository for TestStore {
    async fn save(&self, like: &Like) -> Result<(), DomainError> {
        let mut g = self.likes.lock().unwrap();
        if g.iter()
            .any(|l| l.user_id == like.user_id && l.thought_id == like.thought_id)
        {
            return Err(DomainError::Conflict("already liked".into()));
        }
        g.push(like.clone());
        Ok(())
    }
    async fn delete(&self, user_id: &UserId, thought_id: &ThoughtId) -> Result<(), DomainError> {
        let mut g = self.likes.lock().unwrap();
        let before = g.len();
        g.retain(|l| !(&l.user_id == user_id && &l.thought_id == thought_id));
        if g.len() == before {
            return Err(DomainError::NotFound);
        }
        Ok(())
    }
    async fn find(
        &self,
        user_id: &UserId,
        thought_id: &ThoughtId,
    ) -> Result<Option<Like>, DomainError> {
        Ok(self
            .likes
            .lock()
            .unwrap()
            .iter()
            .find(|l| &l.user_id == user_id && &l.thought_id == thought_id)
            .cloned())
    }
    async fn count_for_thought(&self, thought_id: &ThoughtId) -> Result<i64, DomainError> {
        Ok(self
            .likes
            .lock()
            .unwrap()
            .iter()
            .filter(|l| &l.thought_id == thought_id)
            .count() as i64)
    }
}

#[async_trait]
impl BoostRepository for TestStore {
    async fn save(&self, boost: &Boost) -> Result<(), DomainError> {
        let mut g = self.boosts.lock().unwrap();
        if g.iter()
            .any(|b| b.user_id == boost.user_id && b.thought_id == boost.thought_id)
        {
            return Err(DomainError::Conflict("already boosted".into()));
        }
        g.push(boost.clone());
        Ok(())
    }
    async fn delete(&self, user_id: &UserId, thought_id: &ThoughtId) -> Result<(), DomainError> {
        let mut g = self.boosts.lock().unwrap();
        let before = g.len();
        g.retain(|b| !(&b.user_id == user_id && &b.thought_id == thought_id));
        if g.len() == before {
            return Err(DomainError::NotFound);
        }
        Ok(())
    }
    async fn find(
        &self,
        user_id: &UserId,
        thought_id: &ThoughtId,
    ) -> Result<Option<Boost>, DomainError> {
        Ok(self
            .boosts
            .lock()
            .unwrap()
            .iter()
            .find(|b| &b.user_id == user_id && &b.thought_id == thought_id)
            .cloned())
    }
    async fn count_for_thought(&self, thought_id: &ThoughtId) -> Result<i64, DomainError> {
        Ok(self
            .boosts
            .lock()
            .unwrap()
            .iter()
            .filter(|b| &b.thought_id == thought_id)
            .count() as i64)
    }
}

#[async_trait]
impl EngagementRepository for TestStore {
    async fn get_for_thoughts(
        &self,
        thought_ids: &[ThoughtId],
        viewer_id: Option<&UserId>,
    ) -> Result<
        HashMap<
            ThoughtId,
            (
                crate::models::feed::EngagementStats,
                Option<crate::models::feed::ViewerContext>,
            ),
        >,
        DomainError,
    > {
        use crate::models::feed::{EngagementStats, ViewerContext};
        let likes = self.likes.lock().unwrap();
        let boosts = self.boosts.lock().unwrap();
        let thoughts = self.thoughts.lock().unwrap();

        let mut result = HashMap::new();
        for tid in thought_ids {
            let like_count = likes.iter().filter(|l| &l.thought_id == tid).count() as i64;
            let boost_count = boosts.iter().filter(|b| &b.thought_id == tid).count() as i64;
            let reply_count = thoughts
                .iter()
                .filter(|t| t.in_reply_to_id.as_ref() == Some(tid))
                .count() as i64;
            let viewer = viewer_id.map(|vid| ViewerContext {
                liked: likes
                    .iter()
                    .any(|l| &l.thought_id == tid && &l.user_id == vid),
                boosted: boosts
                    .iter()
                    .any(|b| &b.thought_id == tid && &b.user_id == vid),
            });
            result.insert(
                tid.clone(),
                (
                    EngagementStats {
                        like_count,
                        boost_count,
                        reply_count,
                    },
                    viewer,
                ),
            );
        }
        Ok(result)
    }
}

#[async_trait]
impl FollowRepository for TestStore {
    async fn save(&self, follow: &Follow) -> Result<(), DomainError> {
        let mut g = self.follows.lock().unwrap();
        g.retain(|f| {
            !(f.follower_id == follow.follower_id && f.following_id == follow.following_id)
        });
        g.push(follow.clone());
        Ok(())
    }
    async fn delete(&self, follower_id: &UserId, following_id: &UserId) -> Result<(), DomainError> {
        let mut g = self.follows.lock().unwrap();
        let before = g.len();
        g.retain(|f| !(&f.follower_id == follower_id && &f.following_id == following_id));
        if g.len() == before {
            return Err(DomainError::NotFound);
        }
        Ok(())
    }
    async fn find(
        &self,
        follower_id: &UserId,
        following_id: &UserId,
    ) -> Result<Option<Follow>, DomainError> {
        Ok(self
            .follows
            .lock()
            .unwrap()
            .iter()
            .find(|f| &f.follower_id == follower_id && &f.following_id == following_id)
            .cloned())
    }
    async fn update_state(
        &self,
        follower_id: &UserId,
        following_id: &UserId,
        state: &FollowState,
    ) -> Result<(), DomainError> {
        if let Some(f) = self
            .follows
            .lock()
            .unwrap()
            .iter_mut()
            .find(|f| &f.follower_id == follower_id && &f.following_id == following_id)
        {
            f.state = state.clone();
        }
        Ok(())
    }
    async fn list_followers(
        &self,
        _user_id: &UserId,
        _p: &PageParams,
    ) -> Result<Paginated<User>, DomainError> {
        Ok(Paginated {
            items: vec![],
            total: 0,
            page: 1,
            per_page: 20,
        })
    }
    async fn list_following(
        &self,
        _user_id: &UserId,
        _p: &PageParams,
    ) -> Result<Paginated<User>, DomainError> {
        Ok(Paginated {
            items: vec![],
            total: 0,
            page: 1,
            per_page: 20,
        })
    }
    async fn get_accepted_following_ids(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<UserId>, DomainError> {
        Ok(self
            .follows
            .lock()
            .unwrap()
            .iter()
            .filter(|f| &f.follower_id == user_id && f.state == FollowState::Accepted)
            .map(|f| f.following_id.clone())
            .collect())
    }
    async fn list_mutual(
        &self,
        user_id: &UserId,
        page: &PageParams,
    ) -> Result<Paginated<User>, DomainError> {
        use std::collections::HashSet;
        let follows = self.follows.lock().unwrap();
        let following_ids: HashSet<UserId> = follows
            .iter()
            .filter(|f| &f.follower_id == user_id && f.state == FollowState::Accepted)
            .map(|f| f.following_id.clone())
            .collect();
        let follower_ids: HashSet<UserId> = follows
            .iter()
            .filter(|f| &f.following_id == user_id && f.state == FollowState::Accepted)
            .map(|f| f.follower_id.clone())
            .collect();
        let mutual_ids: HashSet<UserId> =
            following_ids.intersection(&follower_ids).cloned().collect();
        drop(follows);
        let users = self.users.lock().unwrap();
        let all_items: Vec<User> = users
            .iter()
            .filter(|u| mutual_ids.contains(&u.id))
            .cloned()
            .collect();
        let total = all_items.len() as i64;
        let offset = page.offset() as usize;
        let items: Vec<User> = all_items
            .into_iter()
            .skip(offset)
            .take(page.limit() as usize)
            .collect();
        Ok(Paginated {
            items,
            total,
            page: page.page,
            per_page: page.per_page,
        })
    }
}

#[async_trait]
impl BlockRepository for TestStore {
    async fn save(&self, block: &Block) -> Result<(), DomainError> {
        self.blocks.lock().unwrap().push(block.clone());
        Ok(())
    }
    async fn delete(&self, blocker_id: &UserId, blocked_id: &UserId) -> Result<(), DomainError> {
        self.blocks
            .lock()
            .unwrap()
            .retain(|b| !(&b.blocker_id == blocker_id && &b.blocked_id == blocked_id));
        Ok(())
    }
    async fn exists(&self, blocker_id: &UserId, blocked_id: &UserId) -> Result<bool, DomainError> {
        Ok(self
            .blocks
            .lock()
            .unwrap()
            .iter()
            .any(|b| &b.blocker_id == blocker_id && &b.blocked_id == blocked_id))
    }
}

#[async_trait]
impl TagRepository for TestStore {
    async fn find_or_create(&self, name: &str) -> Result<Tag, DomainError> {
        let mut g = self.tags.lock().unwrap();
        if let Some(t) = g.iter().find(|t| t.name == name) {
            return Ok(t.clone());
        }
        let tag = Tag {
            id: g.len() as i32 + 1,
            name: name.to_string(),
        };
        g.push(tag.clone());
        Ok(tag)
    }
    async fn attach_to_thought(&self, _tid: &ThoughtId, _tag_id: i32) -> Result<(), DomainError> {
        Ok(())
    }
    async fn detach_from_thought(&self, _tid: &ThoughtId) -> Result<(), DomainError> {
        Ok(())
    }
    async fn list_for_thought(&self, _tid: &ThoughtId) -> Result<Vec<Tag>, DomainError> {
        Ok(vec![])
    }
    async fn list_thoughts_by_tag(
        &self,
        _name: &str,
        _p: &PageParams,
    ) -> Result<Paginated<Thought>, DomainError> {
        Ok(Paginated {
            items: vec![],
            total: 0,
            page: 1,
            per_page: 20,
        })
    }
    async fn popular_tags(&self, _limit: usize) -> Result<Vec<(String, i64)>, DomainError> {
        Ok(vec![])
    }
}

#[async_trait]
impl ApiKeyRepository for TestStore {
    async fn save(&self, key: &ApiKey) -> Result<(), DomainError> {
        self.api_keys.lock().unwrap().push(key.clone());
        Ok(())
    }
    async fn find_by_hash(&self, hash: &str) -> Result<Option<ApiKey>, DomainError> {
        Ok(self
            .api_keys
            .lock()
            .unwrap()
            .iter()
            .find(|k| k.key_hash == hash)
            .cloned())
    }
    async fn list_for_user(&self, uid: &UserId) -> Result<Vec<ApiKey>, DomainError> {
        Ok(self
            .api_keys
            .lock()
            .unwrap()
            .iter()
            .filter(|k| &k.user_id == uid)
            .cloned()
            .collect())
    }
    async fn delete(&self, id: &ApiKeyId, uid: &UserId) -> Result<(), DomainError> {
        self.api_keys
            .lock()
            .unwrap()
            .retain(|k| !(&k.id == id && &k.user_id == uid));
        Ok(())
    }
}

#[async_trait]
impl ApiKeyService for TestStore {
    async fn validate_key(&self, raw_key: &str) -> Result<Option<UserId>, DomainError> {
        use sha2::{Digest, Sha256};
        let hash = hex::encode(Sha256::digest(raw_key.as_bytes()));
        Ok(self
            .api_keys
            .lock()
            .unwrap()
            .iter()
            .find(|k| k.key_hash == hash)
            .map(|k| k.user_id.clone()))
    }
}

#[async_trait]
impl TopFriendRepository for TestStore {
    async fn set_top_friends(
        &self,
        user_id: &UserId,
        friends: Vec<(UserId, i16)>,
    ) -> Result<(), DomainError> {
        let mut g = self.top_friends.lock().unwrap();
        g.retain(|tf| &tf.user_id != user_id);
        for (fid, pos) in friends {
            g.push(TopFriend {
                user_id: user_id.clone(),
                friend_id: fid,
                position: pos,
            });
        }
        Ok(())
    }
    async fn list_for_user(&self, _uid: &UserId) -> Result<Vec<(TopFriend, User)>, DomainError> {
        Ok(vec![])
    }
}

#[async_trait]
impl NotificationRepository for TestStore {
    async fn save(&self, n: &Notification) -> Result<(), DomainError> {
        self.notifications.lock().unwrap().push(n.clone());
        Ok(())
    }
    async fn list_for_user(
        &self,
        uid: &UserId,
        _p: &PageParams,
    ) -> Result<Paginated<Notification>, DomainError> {
        let items: Vec<_> = self
            .notifications
            .lock()
            .unwrap()
            .iter()
            .filter(|n| &n.user_id == uid)
            .cloned()
            .collect();
        let total = items.len() as i64;
        Ok(Paginated {
            items,
            total,
            page: 1,
            per_page: 20,
        })
    }
    async fn count_unread(&self, uid: &UserId) -> Result<u64, DomainError> {
        Ok(self
            .notifications
            .lock()
            .unwrap()
            .iter()
            .filter(|n| &n.user_id == uid && !n.read)
            .count() as u64)
    }
    async fn mark_read(&self, id: &NotificationId, _uid: &UserId) -> Result<(), DomainError> {
        if let Some(n) = self
            .notifications
            .lock()
            .unwrap()
            .iter_mut()
            .find(|n| &n.id == id)
        {
            n.read = true;
        }
        Ok(())
    }
    async fn mark_all_read(&self, uid: &UserId) -> Result<(), DomainError> {
        for n in self
            .notifications
            .lock()
            .unwrap()
            .iter_mut()
            .filter(|n| &n.user_id == uid)
        {
            n.read = true;
        }
        Ok(())
    }
}

#[async_trait]
impl RemoteActorRepository for TestStore {
    async fn upsert(&self, _a: &RemoteActor) -> Result<(), DomainError> {
        Ok(())
    }
    async fn find_by_url(&self, _url: &str) -> Result<Option<RemoteActor>, DomainError> {
        Ok(None)
    }
}

#[async_trait]
impl FederationLookupPort for TestStore {
    async fn lookup_actor(&self, _handle: &str) -> Result<RemoteActor, DomainError> {
        Err(DomainError::NotFound)
    }

    async fn actor_json(&self, _user_id: &UserId) -> Result<String, DomainError> {
        Err(DomainError::NotFound)
    }

    async fn followers_collection_json(
        &self,
        _user_id: &UserId,
        _page: Option<u32>,
    ) -> Result<String, DomainError> {
        Err(DomainError::NotFound)
    }

    async fn following_collection_json(
        &self,
        _user_id: &UserId,
        _page: Option<u32>,
    ) -> Result<String, DomainError> {
        Err(DomainError::NotFound)
    }
}

#[async_trait]
impl FederationFollowPort for TestStore {
    async fn follow_remote(
        &self,
        _local_user_id: &UserId,
        _handle: &str,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn unfollow_remote(
        &self,
        _local_user_id: &UserId,
        _handle: &str,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn get_remote_following(
        &self,
        _user_id: &UserId,
    ) -> Result<Vec<RemoteActor>, DomainError> {
        Ok(self.remote_following.lock().unwrap().clone())
    }

    async fn broadcast_move(
        &self,
        _user_id: &UserId,
        _new_actor_url: url::Url,
    ) -> Result<(), DomainError> {
        Ok(())
    }
}

#[async_trait]
impl FederationFollowRequestPort for TestStore {
    async fn get_pending_followers(
        &self,
        _user_id: &UserId,
    ) -> Result<Vec<RemoteActor>, DomainError> {
        Ok(vec![])
    }

    async fn accept_follow_request(
        &self,
        _user_id: &UserId,
        _actor_url: &str,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn reject_follow_request(
        &self,
        _user_id: &UserId,
        _actor_url: &str,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn get_remote_followers(
        &self,
        _user_id: &UserId,
    ) -> Result<Vec<RemoteActor>, DomainError> {
        Ok(self.remote_followers.lock().unwrap().clone())
    }

    async fn remove_remote_follower(
        &self,
        _user_id: &UserId,
        _actor_url: &str,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn mark_follower_accepted(
        &self,
        _user_id: &UserId,
        _actor_url: &str,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn mark_follower_rejected(
        &self,
        _user_id: &UserId,
        _actor_url: &str,
    ) -> Result<(), DomainError> {
        Ok(())
    }
}

#[async_trait]
impl FederationFetchPort for TestStore {
    async fn fetch_outbox_page(
        &self,
        _outbox_url: &str,
        _page: u32,
    ) -> Result<Vec<crate::models::remote_note::RemoteNote>, DomainError> {
        Ok(vec![])
    }

    async fn fetch_actor_urls_from_collection(
        &self,
        _collection_url: &str,
    ) -> Result<Vec<String>, DomainError> {
        Ok(vec![])
    }

    async fn resolve_actor_profiles(
        &self,
        _urls: Vec<String>,
    ) -> Vec<crate::models::actor_connection_summary::ActorConnectionSummary> {
        vec![]
    }
}

#[async_trait]
impl FederationBlockPort for TestStore {
    async fn block_remote(
        &self,
        _local_user_id: &UserId,
        _handle: &str,
    ) -> Result<(), DomainError> {
        Ok(())
    }
    async fn unblock_remote(
        &self,
        _local_user_id: &UserId,
        _handle: &str,
    ) -> Result<(), DomainError> {
        Ok(())
    }
}

#[async_trait]
impl RemoteActorConnectionRepository for TestStore {
    async fn upsert_connections(
        &self,
        _actor_url: &str,
        _connection_type: &str,
        _page: u32,
        _actors: &[crate::models::actor_connection_summary::ActorConnectionSummary],
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn list_connections(
        &self,
        _actor_url: &str,
        _connection_type: &str,
        _page: u32,
    ) -> Result<Vec<crate::models::actor_connection_summary::ActorConnectionSummary>, DomainError>
    {
        Ok(vec![])
    }

    async fn connection_page_age(
        &self,
        _actor_url: &str,
        _connection_type: &str,
        _page: u32,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, DomainError> {
        Ok(None)
    }
}

#[async_trait]
impl FeedRepository for TestStore {
    async fn query(
        &self,
        _req: &crate::ports::FeedRequest,
    ) -> Result<Paginated<FeedEntry>, DomainError> {
        Ok(Paginated {
            items: vec![],
            total: 0,
            page: 1,
            per_page: 20,
        })
    }
}

#[async_trait]
impl SearchPort for TestStore {
    async fn search_thoughts(
        &self,
        _q: &str,
        _p: &PageParams,
        _v: Option<&UserId>,
    ) -> Result<Paginated<FeedEntry>, DomainError> {
        Ok(Paginated {
            items: vec![],
            total: 0,
            page: 1,
            per_page: 20,
        })
    }
    async fn search_users(
        &self,
        _q: &str,
        _p: &PageParams,
    ) -> Result<Paginated<User>, DomainError> {
        Ok(Paginated {
            items: vec![],
            total: 0,
            page: 1,
            per_page: 20,
        })
    }
}

#[async_trait]
impl FederationSchedulerPort for TestStore {
    async fn schedule_actor_posts_fetch(&self, _: &str, _: &str) -> Result<(), DomainError> {
        Ok(())
    }
    async fn schedule_connections_fetch(
        &self,
        _: &str,
        _: &str,
        _: &str,
        _: u32,
    ) -> Result<(), DomainError> {
        Ok(())
    }
}

#[async_trait]
impl EventPublisher for TestStore {
    async fn publish(&self, event: &DomainEvent) -> Result<(), DomainError> {
        self.events.lock().unwrap().push(event.clone());
        Ok(())
    }
}

pub struct NoOpEventPublisher;
#[async_trait]
impl EventPublisher for NoOpEventPublisher {
    async fn publish(&self, _e: &DomainEvent) -> Result<(), DomainError> {
        Ok(())
    }
}

#[derive(Default, Clone)]
pub struct TestOutbox {
    pub entries: Arc<Mutex<Vec<DomainEvent>>>,
}

impl TestOutbox {
    pub fn staged(&self) -> Vec<DomainEvent> {
        self.entries.lock().unwrap().clone()
    }
}

#[async_trait]
impl OutboxWriter for TestOutbox {
    async fn append(&self, event: &DomainEvent) -> Result<(), DomainError> {
        self.entries.lock().unwrap().push(event.clone());
        Ok(())
    }
}

pub struct NoOpOutboxWriter;
#[async_trait]
impl OutboxWriter for NoOpOutboxWriter {
    async fn append(&self, _e: &DomainEvent) -> Result<(), DomainError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests;
