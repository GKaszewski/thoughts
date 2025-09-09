use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub page: u64,
    pub page_size: u64,
    pub total_pages: u64,
    pub total_items: u64,
}
