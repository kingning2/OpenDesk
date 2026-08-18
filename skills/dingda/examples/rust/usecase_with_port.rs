//! Example: UseCase with Port injection (skeleton).

use tracing::instrument;

pub trait ItemRepo: Send + Sync {
    fn list(&self) -> Result<Vec<String>, RepoError>;
}

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("not found")]
    NotFound,
}

pub struct ListItems<'a> {
    repo: &'a dyn ItemRepo,
}

impl<'a> ListItems<'a> {
    pub fn new(repo: &'a dyn ItemRepo) -> Self {
        Self { repo }
    }

    #[instrument(skip(self))]
    pub fn execute(&self) -> Result<Vec<String>, RepoError> {
        self.repo.list()
    }
}
