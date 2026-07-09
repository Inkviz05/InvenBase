use std::sync::Arc;

use crate::config::Config;
use crate::database::Database;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub config: Config,
}

impl AppState {
    pub fn new(db: Arc<Database>, config: Config) -> Self {
        AppState { db, config }
    }
}
