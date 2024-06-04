use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct ToDo {
    pub(crate) id: Option<String>,
    pub(crate) title: String,
    pub(crate) content: String,
    pub(crate) completed: Option<bool>,
    pub(crate) created_at: Option<DateTime<Utc>>,
    pub(crate) updated_at: Option<DateTime<Utc>>,
}

pub struct AppState {
    pub todo_db: Arc<Mutex<Vec<ToDo>>>,
}

impl AppState {
    pub(crate) fn init() -> Self {
        AppState {
            todo_db: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct QueryOptions {
    pub(crate) page: Option<usize>,
    pub(crate) limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct UpdateToDoSchema {
    pub(crate) title: Option<String>,
    pub(crate) content: Option<String>,
    pub(crate) completed: Option<bool>,
}
