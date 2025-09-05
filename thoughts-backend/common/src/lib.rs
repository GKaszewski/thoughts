use sea_orm::prelude::DateTimeWithTimeZone;
use serde::Serialize;
use utoipa::ToSchema;

// Wrapper type for DateTimeWithTimeZone
#[derive(Serialize, ToSchema)]
#[schema(example = "2025-09-05T12:34:56Z")] // Example for OpenAPI
pub struct DateTimeWithTimeZoneWrapper(String);

impl From<DateTimeWithTimeZone> for DateTimeWithTimeZoneWrapper {
    fn from(value: DateTimeWithTimeZone) -> Self {
        DateTimeWithTimeZoneWrapper(value.to_rfc3339())
    }
}
