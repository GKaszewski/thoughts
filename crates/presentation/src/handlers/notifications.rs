use crate::{
    deps_struct,
    errors::ApiError,
    extractors::{AuthUser, Deps},
};
use api_types::requests::NotificationUpdateRequest;
use application::use_cases::notifications::{
    count_unread_notifications, list_notifications as uc_list_notifications,
    mark_all_notifications_read, mark_notification_read as uc_mark_notification_read,
};
use axum::{
    extract::Path,
    http::StatusCode,
    Json,
};
use domain::{
    models::feed::PageParams, ports::NotificationRepository, value_objects::NotificationId,
};
use uuid::Uuid;

deps_struct!(NotificationsDeps {
    notifications: NotificationRepository,
});

#[utoipa::path(get, path = "/notifications", responses((status = 200, description = "Notification summary")), security(("bearer_auth" = [])))]
pub async fn list_notifications(
    Deps(d): Deps<NotificationsDeps>,
    AuthUser(uid): AuthUser,
) -> Result<Json<serde_json::Value>, ApiError> {
    let page = PageParams {
        page: 1,
        per_page: 20,
    };
    let result = uc_list_notifications(&*d.notifications, &uid, page).await?;
    let unread = count_unread_notifications(&*d.notifications, &uid).await?;
    Ok(Json(serde_json::json!({
        "total": result.total,
        "unread": unread
    })))
}

#[utoipa::path(patch, path = "/notifications/{id}", params(("id" = uuid::Uuid, Path, description = "Notification ID")), request_body = NotificationUpdateRequest, responses((status = 204, description = "Marked read")), security(("bearer_auth" = [])))]
pub async fn mark_notification_read(
    Deps(d): Deps<NotificationsDeps>,
    AuthUser(uid): AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<NotificationUpdateRequest>,
) -> Result<StatusCode, ApiError> {
    uc_mark_notification_read(
        &*d.notifications,
        &NotificationId::from_uuid(id),
        &uid,
        body.read,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(patch, path = "/notifications", request_body = NotificationUpdateRequest, responses((status = 204, description = "All marked read")), security(("bearer_auth" = [])))]
pub async fn mark_all_read(
    Deps(d): Deps<NotificationsDeps>,
    AuthUser(uid): AuthUser,
    Json(body): Json<NotificationUpdateRequest>,
) -> Result<StatusCode, ApiError> {
    mark_all_notifications_read(&*d.notifications, &uid, body.read).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::make_state;
    use axum::{
        body::Body,
        http::{header, Request},
        routing::{get, patch},
        Router,
    };
    use tower::ServiceExt;

    fn app() -> Router {
        Router::new()
            .route("/notifications", patch(mark_all_read))
            .route("/notifications/{id}", patch(mark_notification_read))
            .with_state(make_state())
    }

    #[tokio::test]
    async fn patch_notification_without_auth_returns_401() {
        let resp = app()
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/notifications/00000000-0000-0000-0000-000000000001")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"read":true}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 401);
    }

    #[tokio::test]
    async fn patch_all_without_auth_returns_401() {
        let resp = app()
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/notifications")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"read":true}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 401);
    }
}
