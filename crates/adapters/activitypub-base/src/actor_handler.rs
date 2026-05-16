use activitypub_federation::{
    axum::json::FederationJson, config::Data, protocol::context::WithContext, traits::Object,
};
use axum::extract::Path;

use crate::actors::{Person, get_local_actor};
use crate::data::FederationData;
use crate::error::Error;

pub async fn actor_handler(
    Path(username): Path<String>,
    data: Data<FederationData>,
) -> Result<FederationJson<WithContext<Person>>, Error> {
    let ap_user = data
        .user_repo
        .find_by_username(&username)
        .await
        .map_err(Error::from)?
        .ok_or_else(|| Error::bad_request(anyhow::anyhow!("user not found")))?;

    let db_actor = get_local_actor(ap_user.id, &data).await?;
    let person = db_actor.into_json(&data).await?;

    Ok(FederationJson(WithContext::new_default(person)))
}
