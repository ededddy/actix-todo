use crate::models::ToDo;
use mongodb::{bson::doc, options::IndexOptions, Client, IndexModel};

pub(crate) async fn init_client(db_name: &str, collection_name: &str) -> Client {
    let connection_url = env!("mongo_connection_string");
    let conn_pool = Client::with_uri_str(connection_url)
        .await
        .expect("Failed to connect to mongoDB");
    create_todo_index(&conn_pool, db_name, collection_name).await;
    conn_pool
}

pub(crate) async fn create_todo_index(client: &Client, db_name: &str, collection_name: &str) {
    let options = IndexOptions::builder().unique(true).build();
    let index_model = IndexModel::builder()
        .keys(doc! {"id": 1})
        .options(options)
        .build();
    let db = client.database(db_name);
    let collections = db
        .list_collection_names(None)
        .await
        .expect("error getting collection names from database {db_name}");

    if !collections.contains(&collection_name.to_string()) {
        db.create_collection(collection_name, None)
            .await
            .expect("error creating collection {collection_name}");
    }

    db.collection::<ToDo>(collection_name)
        .create_index(index_model, None)
        .await
        .expect("creating an index should succeed");
}
