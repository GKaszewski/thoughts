use app::{
    persistence::{follow, user},
    state::AppState,
};
use models::domains::thought;
use serde_json::json;

// This function handles pushing a new thought to all followers.
pub async fn federate_thought(
    state: AppState,
    thought: thought::Model,
    author: models::domains::user::Model,
) {
    // Find all followers of the author
    let follower_ids = match follow::get_follower_ids(&state.conn, author.id).await {
        Ok(ids) => ids,
        Err(e) => {
            tracing::error!("Failed to get followers for federation: {}", e);
            return;
        }
    };

    if follower_ids.is_empty() {
        println!("No followers to federate to for user {}", author.username);
        return;
    }

    let thought_url = format!("{}/thoughts/{}", &state.base_url, thought.id);
    let author_url = format!("{}/users/{}", &state.base_url, author.username);

    // Construct the "Create" activity containing the "Note" object
    let activity = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": format!("{}/activity", thought_url),
        "type": "Create",
        "actor": author_url,
        "object": {
            "id": thought_url,
            "type": "Note",
            "attributedTo": author_url,
            "content": thought.content,
            "published": thought.created_at.to_rfc3339(),
            "to": ["https://www.w3.org/ns/activitystreams#Public"],
            "cc": [format!("{}/followers", author_url)]
        }
    });

    // Get the inbox URLs for all followers
    // In a real federated app, you would store remote users' full inbox URLs.
    // For now, we assume followers are local and construct their inbox URLs.
    let followers = match user::get_users_by_ids(&state.conn, follower_ids).await {
        Ok(users) => users,
        Err(e) => {
            tracing::error!("Failed to get follower user objects: {}", e);
            return;
        }
    };

    let client = reqwest::Client::new();
    for follower in followers {
        let inbox_url = format!("{}/users/{}/inbox", &state.base_url, follower.username);
        tracing::info!("Federating post {} to {}", thought.id, inbox_url);

        let res = client.post(&inbox_url).json(&activity).send().await;

        if let Err(e) = res {
            tracing::error!("Failed to federate to {}: {}", inbox_url, e);
        }
    }
}
