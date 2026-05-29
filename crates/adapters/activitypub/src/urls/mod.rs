use url::Url;

pub struct ThoughtsUrls {
    pub base_url: String,
}

impl ThoughtsUrls {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    pub fn user_url(&self, id: &str) -> Url {
        Url::parse(&format!("{}/users/{}", self.base_url, id)).expect("valid URL")
    }

    pub fn thought_url(&self, thought_id: uuid::Uuid) -> Url {
        Url::parse(&format!("{}/thoughts/{}", self.base_url, thought_id)).expect("valid URL")
    }

    pub fn user_inbox(&self, id: &str) -> Url {
        Url::parse(&format!("{}/users/{}/inbox", self.base_url, id)).expect("valid URL")
    }

    pub fn user_outbox(&self, id: &str) -> Url {
        Url::parse(&format!("{}/users/{}/outbox", self.base_url, id)).expect("valid URL")
    }

    pub fn user_followers(&self, id: &str) -> Url {
        Url::parse(&format!("{}/users/{}/followers", self.base_url, id)).expect("valid URL")
    }
}

#[cfg(test)]
mod tests;
