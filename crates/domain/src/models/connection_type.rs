#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionType {
    Followers,
    Following,
}

impl ConnectionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Followers => "followers",
            Self::Following => "following",
        }
    }
}
