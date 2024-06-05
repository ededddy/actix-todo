use crate::{
    models::{AppState, CreateToDoSchema, QueryOptions, ToDo, UpdateToDoSchema},
    responses::{GenericResponse, ToDoData, ToDoInfo, ToDoListResponse},
};
use actix_web::{delete, get, patch, post, web, HttpResponse, Responder};
use chrono::Utc;
use futures::stream::{StreamExt, TryStreamExt};
use mongodb::{bson::doc, options::FindOptions, Client};
use uuid::Uuid;

pub(crate) fn web_service_config(cfg: &mut web::ServiceConfig) {
    let scope = web::scope("/api")
        .service(healthcheck_handler)
        .service(todos_list_handler)
        .service(create_todo_handler)
        .service(get_todo_handler)
        //.service(update_todo_handler)
        //.service(delete_todo_handler);
        ;
    cfg.service(scope);
}

#[get("/healthchecks")]
pub(crate) async fn healthcheck_handler(state: web::Data<AppState>) -> impl Responder {
    const MESSAGE: &str = "All Depended Service are in healthy shape";
    match state.connection_pool.list_database_names(None, None).await {
        Ok(_) => {
            let response_json = GenericResponse {
                status: "healthy".to_string(),
                message: MESSAGE.to_string(),
            };

            HttpResponse::Ok().json(&response_json)
        }
        Err(error) => HttpResponse::Ok().json(&GenericResponse {
            status: "unhealthy".to_string(),
            message: error.to_string(),
        }),
    }
}

#[get("/todos")]
pub async fn todos_list_handler(
    opts: web::Query<QueryOptions>,
    state: web::Data<AppState>,
) -> impl Responder {
    let todos = state
        .connection_pool
        .database(&state.database_name)
        .collection(&state.collection_name);

    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;
    let options = FindOptions::builder()
        .limit(Some(limit as i64))
        .skip(Some(offset as u64))
        .build();

    let cursor = todos
        .find(None, Some(options))
        .await
        .expect("error getting all documents");
    let todos: Vec<_> = cursor
        .try_collect()
        .await
        .expect("error converting cursors to todos");

    let json_response = ToDoListResponse {
        status: "success".to_string(),
        result: todos.len(),
        todos,
    };
    HttpResponse::Ok().json(json_response)
}

#[post("/todos")]
pub(crate) async fn create_todo_handler(
    body: web::Json<CreateToDoSchema>,
    state: web::Data<AppState>,
) -> impl Responder {
    let todos: mongodb::Collection<ToDo> = state
        .connection_pool
        .database(&state.database_name)
        .collection(&state.collection_name);

    let body = body.into_inner();

    let todo: ToDo = body.into();
    let insert_result = todos.insert_one(todo.clone(), None).await;
    match insert_result {
        Ok(_) => {
            let json_response = ToDoInfo {
                status: "success".to_string(),
                data: ToDoData { todo },
            };

            return HttpResponse::Ok().json(json_response);
        }
        Err(error) => HttpResponse::InternalServerError().json(GenericResponse {
            status: "fail".to_string(),
            message: format!("Failed to add todo. Reason:{}", error),
        }),
    }
}

#[get("/todos/{id}")]
pub(crate) async fn get_todo_handler(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let todos: mongodb::Collection<ToDo> = state
        .connection_pool
        .database(&state.database_name)
        .collection(&state.collection_name);

    let id = path.into_inner();
    let todo = todos
        .find_one(doc! {"_id": id.clone()}, None)
        .await
        .expect("error looking up the document in 'ToDo'");

    if todo.is_none() {
        let error_response = GenericResponse {
            status: "fail".to_string(),
            message: format!("ToDo with id : '{}' not found", id),
        };
        return HttpResponse::NotFound().json(error_response);
    }

    let todo = todo.unwrap();
    let json_response = ToDoInfo {
        status: "success".to_string(),
        data: ToDoData { todo },
    };
    HttpResponse::Ok().json(json_response)
}

// #[patch("/todos/{id}")]
// pub(crate) async fn update_todo_handler(
//     path: web::Path<String>,
//     body: web::Json<UpdateToDoSchema>,
//     data: web::Data<AppState>,
// ) -> impl Responder {
//     let mut todos = data.todo_db.lock().unwrap();
//
//     let id = path.into_inner();
//     let todo = todos.iter_mut().find(|item| item.id == Some(id.clone()));
//
//     if todo.is_none() {
//         let error_response = GenericResponse {
//             status: "fail".to_string(),
//             message: format!("ToDo with id : '{}' not found", id),
//         };
//         return HttpResponse::NotFound().json(error_response);
//     }
//
//     let todo /* mutable reference */ = todo.unwrap();
//     let datetime = Utc::now();
//     let title = body.title.to_owned().unwrap_or(todo.title.to_owned());
//     let content = body.content.to_owned().unwrap_or(todo.content.to_owned());
//     let payload = ToDo {
//         id: todo.id.to_owned(),
//         title,
//         content,
//         completed: if !body.completed.is_none() {
//             body.completed
//         } else {
//             todo.completed.to_owned()
//         },
//         created_at: todo.created_at,
//         updated_at: Some(datetime),
//     };
//     *todo = payload;
//     let json_response = ToDoInfo {
//         status: "success".to_string(),
//         data: ToDoData { todo: todo.clone() },
//     };
//
//     HttpResponse::Ok().json(json_response)
// }
//
// #[delete("/todos/{id}")]
// pub(crate) async fn delete_todo_handler(
//     path: web::Path<String>,
//     data: web::Data<AppState>,
// ) -> impl Responder {
//     let mut todos = data.todo_db.lock().unwrap();
//
//     let id = path.into_inner();
//     let todo = todos.iter_mut().find(|item| item.id == Some(id.to_owned()));
//
//     if todo.is_none() {
//         let error_response = GenericResponse {
//             status: "fail".to_string(),
//             message: format!("ToDo with id : '{}' not found", id),
//         };
//         return HttpResponse::NotFound().json(error_response);
//     }
//
//     todos.retain(|item| item.id != Some(id.to_owned()));
//     HttpResponse::NoContent().finish()
// }
