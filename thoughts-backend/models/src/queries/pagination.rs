use serde::Deserialize;
use utoipa::IntoParams;

const DEFAULT_PAGE: u64 = 1;
const DEFAULT_PAGE_SIZE: u64 = 20;

#[derive(Deserialize, IntoParams)]
pub struct PaginationQuery {
    #[param(nullable = true, example = 1)]
    page: Option<u64>,
    #[param(nullable = true, example = 20)]
    page_size: Option<u64>,
}

impl PaginationQuery {
    pub fn page(&self) -> u64 {
        self.page.unwrap_or(DEFAULT_PAGE).max(1)
    }

    pub fn page_size(&self) -> u64 {
        self.page_size.unwrap_or(DEFAULT_PAGE_SIZE).max(1)
    }

    pub fn offset(&self) -> u64 {
        (self.page() - 1) * self.page_size()
    }
}
