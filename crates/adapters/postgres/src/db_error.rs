use domain::errors::DomainError;

pub(crate) trait IntoDbResult<T> {
    fn into_domain(self) -> Result<T, DomainError>;
}

impl<T> IntoDbResult<T> for Result<T, sqlx::Error> {
    fn into_domain(self) -> Result<T, DomainError> {
        self.map_err(|e| {
            if let sqlx::Error::Database(ref db) = e {
                if db.code().as_deref() == Some("23505") {
                    return DomainError::Conflict(
                        db.constraint().unwrap_or("conflict").to_string(),
                    );
                }
            }
            DomainError::Internal(e.to_string())
        })
    }
}
