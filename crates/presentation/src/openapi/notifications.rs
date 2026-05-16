use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    crate::handlers::notifications::list_notifications,
    crate::handlers::notifications::mark_notification_read,
    crate::handlers::notifications::mark_all_read,
))]
pub struct NotificationsDoc;
