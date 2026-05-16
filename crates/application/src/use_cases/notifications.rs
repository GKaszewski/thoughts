use domain::{
    errors::DomainError,
    models::feed::{PageParams, Paginated},
    models::notification::Notification,
    ports::NotificationRepository,
    value_objects::{NotificationId, UserId},
};

pub async fn list_notifications(
    repo: &dyn NotificationRepository,
    user_id: &UserId,
    page: PageParams,
) -> Result<Paginated<Notification>, DomainError> {
    repo.list_for_user(user_id, &page).await
}

pub async fn count_unread_notifications(
    repo: &dyn NotificationRepository,
    user_id: &UserId,
) -> Result<u64, DomainError> {
    repo.count_unread(user_id).await
}

pub async fn mark_notification_read(
    repo: &dyn NotificationRepository,
    id: &NotificationId,
    user_id: &UserId,
    is_read: bool,
) -> Result<(), DomainError> {
    if is_read {
        repo.mark_read(id, user_id).await
    } else {
        Ok(())
    }
}

pub async fn mark_all_notifications_read(
    repo: &dyn NotificationRepository,
    user_id: &UserId,
    is_read: bool,
) -> Result<(), DomainError> {
    if is_read {
        repo.mark_all_read(user_id).await
    } else {
        Ok(())
    }
}
