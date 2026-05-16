use crate::value_objects::UserId;

#[derive(Debug, Clone)]
pub struct TopFriend {
    pub user_id: UserId,
    pub friend_id: UserId,
    pub position: i16,
}
