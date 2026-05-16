use activitypub_federation::{axum::json::FederationJson, config::Data};
use axum::extract::{Path, Query};
use serde::Deserialize;
use serde_json::json;

use crate::data::FederationData;
use crate::error::Error;
use crate::urls::AP_PAGE_SIZE;

#[derive(Deserialize)]
pub struct PageQuery {
    page: Option<u32>,
}

async fn collection_handler(
    user_id_str: &str,
    query: PageQuery,
    data: Data<FederationData>,
    collection_type: &str,
) -> Result<FederationJson<serde_json::Value>, Error> {
    let user_id = uuid::Uuid::parse_str(user_id_str)
        .map_err(|_| Error::bad_request(anyhow::anyhow!("invalid user id")))?;

    data.user_repo
        .find_by_id(user_id)
        .await
        .map_err(Error::from)?
        .ok_or_else(|| Error::not_found(anyhow::anyhow!("user not found")))?;

    let collection_id = format!(
        "{}/users/{}/{}",
        data.base_url, user_id_str, collection_type
    );

    let total = match collection_type {
        "followers" => data.federation_repo.count_followers(user_id).await,
        _ => data.federation_repo.count_following(user_id).await,
    }
    .map_err(Error::from)?;

    if let Some(page) = query.page {
        let page = page.max(1);
        let offset = (page.saturating_sub(1) as usize) * AP_PAGE_SIZE;

        let items: Vec<String> = match collection_type {
            "followers" => data
                .federation_repo
                .get_followers_page(user_id, offset as u32, AP_PAGE_SIZE)
                .await
                .map_err(Error::from)?
                .into_iter()
                .map(|f| f.actor.url)
                .collect(),
            _ => data
                .federation_repo
                .get_following_page(user_id, offset as u32, AP_PAGE_SIZE)
                .await
                .map_err(Error::from)?
                .into_iter()
                .map(|a| a.url)
                .collect(),
        };

        let has_next = offset + items.len() < total;

        let mut obj = json!({
            "@context": crate::urls::AP_CONTEXT,
            "type": "OrderedCollectionPage",
            "id": format!("{}?page={}", collection_id, page),
            "partOf": collection_id,
            "totalItems": total,
            "orderedItems": items,
        });

        if has_next {
            obj["next"] = json!(format!("{}?page={}", collection_id, page + 1));
        }

        Ok(FederationJson(obj))
    } else {
        Ok(FederationJson(json!({
            "@context": crate::urls::AP_CONTEXT,
            "type": "OrderedCollection",
            "id": collection_id,
            "totalItems": total,
            "first": format!("{}?page=1", collection_id),
        })))
    }
}

pub async fn followers_handler(
    Path(user_id_str): Path<String>,
    Query(query): Query<PageQuery>,
    data: Data<FederationData>,
) -> Result<FederationJson<serde_json::Value>, Error> {
    collection_handler(&user_id_str, query, data, "followers").await
}

pub async fn following_handler(
    Path(user_id_str): Path<String>,
    Query(query): Query<PageQuery>,
    data: Data<FederationData>,
) -> Result<FederationJson<serde_json::Value>, Error> {
    collection_handler(&user_id_str, query, data, "following").await
}
