use mongodb::{bson::doc, Client, Collection};
use serde::de::DeserializeOwned;

pub(crate) async fn init_client(
    db_name: &str,
    collection_name: &str,
    user_collection_name: &str,
) -> Client {
    let connection_url = env!("mongo_connection_string");
    let conn_pool = Client::with_uri_str(connection_url)
        .await
        .expect("Failed to connect to mongoDB");
    create_todo_collection(&conn_pool, db_name, collection_name, user_collection_name).await;
    conn_pool
}

pub(crate) async fn create_todo_collection(
    client: &Client,
    db_name: &str,
    collection_name: &str,
    user_collection_name: &str,
) {
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
    if !collections.contains(&user_collection_name.to_string()) {
        db.create_collection(user_collection_name, None)
            .await
            .expect("error creating collection {user_collection_name}");
    }
}

pub(crate) fn get_collection<T>(
    connection_pool: &Client,
    database_name: &str,
    collection_name: &str,
) -> Collection<T>
where
    T: DeserializeOwned + Unpin + Send + Sync,
{
    connection_pool
        .database(&database_name)
        .collection::<T>(&collection_name)
}

pub(crate) async fn get_by_id<T>(collection: &Collection<T>, id: &str) -> Option<T>
where
    T: DeserializeOwned + Unpin + Send + Sync,
{
    collection
        .find_one(doc! { "_id":id}, None)
        .await
        .expect("error looking up the document in 'ToDo'")
}
