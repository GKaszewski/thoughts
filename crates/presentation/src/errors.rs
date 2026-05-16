use api_types::responses::ErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use domain::errors::DomainError;

pub enum ApiError {
    Domain(DomainError),
    Unauthorized,
    BadRequest(String),
}

impl From<DomainError> for ApiError {
    fn from(e: DomainError) -> Self {
        Self::Domain(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            Self::Domain(DomainError::NotFound) => (StatusCode::NOT_FOUND, "not found".into()),
            Self::Domain(DomainError::Unauthorized) => {
                (StatusCode::UNAUTHORIZED, "unauthorized".into())
            }
            Self::Domain(DomainError::Forbidden) => (StatusCode::FORBIDDEN, "forbidden".into()),
            Self::Domain(DomainError::Conflict(m)) => (StatusCode::CONFLICT, m),
            Self::Domain(DomainError::UniqueViolation { field }) => {
                (StatusCode::CONFLICT, format!("{field} already taken"))
            }
            Self::Domain(DomainError::InvalidInput(m)) => (StatusCode::UNPROCESSABLE_ENTITY, m),
            Self::Domain(DomainError::ExternalService(_)) => {
                (StatusCode::BAD_GATEWAY, "external service error".into())
            }
            Self::Domain(DomainError::Internal(_)) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".into(),
            ),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized".into()),
            Self::BadRequest(m) => (StatusCode::BAD_REQUEST, m),
        };
        (status, Json(ErrorResponse { error: msg })).into_response()
    }
}
