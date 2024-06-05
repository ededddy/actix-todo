use crate::database::init_client;
use chrono::{DateTime, Utc};
use mongodb::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct ToDo {
    pub(crate) id: Option<String>,
    pub(crate) title: String,
    pub(crate) content: String,
    pub(crate) completed: Option<bool>,
    pub(crate) created_at: Option<DateTime<Utc>>,
    pub(crate) updated_at: Option<DateTime<Utc>>,
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

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    pub(crate) connection_pool: Client,
    pub(crate) database_name: String,
    pub(crate) collection_name: String,
}

impl AppState {
    pub(crate) async fn init() -> Self {
        let db_name = env!("db_name");
        let collection_name = env!("collection_name");
        let pool = init_client(db_name, collection_name).await;

        Self {
            connection_pool: pool.clone(),
            database_name: db_name.to_string(),
            collection_name: collection_name.to_string(),
        }
    }
}
