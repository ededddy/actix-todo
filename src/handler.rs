use crate::{
    models::{AppState, QueryOptions, ToDo, UpdateToDoSchema},
    responses::{GenericResponse, ToDoData, ToDoInfo, ToDoListResponse},
};
use actix_web::{delete, get, patch, post, web, HttpResponse, Responder};
use chrono::Utc;
use uuid::Uuid;

pub(crate) fn web_service_config(cfg: &mut web::ServiceConfig) {
    let scope = web::scope("/api")
        .service(healthcheck_handler)
        .service(todos_list_handler)
        .service(create_todo_handler)
        .service(get_todo_handler)
        .service(update_todo_handler)
        .service(delete_todo_handler);
    cfg.service(scope);
}

#[get("/healthchecks")]
pub(crate) async fn healthcheck_handler() -> impl Responder {
    const MESSAGE: &str = "Build simple CRUD API with Rust and actix-web";
    let response_json = GenericResponse {
        status: "healthy".to_string(),
        message: MESSAGE.to_string(),
    };

    HttpResponse::Ok().json(&response_json)
}

#[get("/todos")]
pub async fn todos_list_handler(
    opts: web::Query<QueryOptions>,
    data: web::Data<AppState>,
) -> impl Responder {
    let todos = data.todo_db.lock().unwrap();

    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    let todos: Vec<ToDo> = todos.clone().into_iter().skip(offset).take(limit).collect();
    let json_response = ToDoListResponse {
        status: "success".to_string(),
        result: todos.len(),
        todos,
    };
    HttpResponse::Ok().json(json_response)
}

#[post("/todos")]
pub(crate) async fn create_todo_handler(
    mut body: web::Json<ToDo>,
    data: web::Data<AppState>,
) -> impl Responder {
    let mut todos = data.todo_db.lock().unwrap();

    let todo = todos.iter().find(|todo| todo.title == body.title);

    if todo.is_some() {
        let error_response = GenericResponse {
            status: "fail".to_string(),
            message: format!("ToDo with title:'{}' already exists", body.title),
        };
        return HttpResponse::Conflict().json(error_response);
    }
    let uuid_v4 = Uuid::new_v4();
    let datetime = Utc::now();

    body.id = Some(uuid_v4.to_string());
    body.completed = Some(false);
    body.created_at = Some(datetime);
    body.updated_at = Some(datetime);

    let todo = body.to_owned();
    todos.push(body.into_inner());

    let json_response = ToDoInfo {
        status: "success".to_string(),
        data: ToDoData { todo },
    };

    HttpResponse::Ok().json(json_response)
}

#[get("/todos/{id}")]
pub(crate) async fn get_todo_handler(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    let todos = data.todo_db.lock().unwrap();

    let id = path.into_inner();
    let todo = todos.iter().find(|item| item.id == Some(id.to_string()));
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
        data: ToDoData { todo: todo.clone() },
    };
    HttpResponse::Ok().json(json_response)
}

#[patch("/todos/{id}")]
pub(crate) async fn update_todo_handler(
    path: web::Path<String>,
    body: web::Json<UpdateToDoSchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let mut todos = data.todo_db.lock().unwrap();

    let id = path.into_inner();
    let todo = todos.iter_mut().find(|item| item.id == Some(id.clone()));

    if todo.is_none() {
        let error_response = GenericResponse {
            status: "fail".to_string(),
            message: format!("ToDo with id : '{}' not found", id),
        };
        return HttpResponse::NotFound().json(error_response);
    }

    let todo /* mutable reference */ = todo.unwrap();
    let datetime = Utc::now();
    let title = body.title.to_owned().unwrap_or(todo.title.to_owned());
    let content = body.content.to_owned().unwrap_or(todo.content.to_owned());
    let payload = ToDo {
        id: todo.id.to_owned(),
        title,
        content,
        completed: if !body.completed.is_none() {
            body.completed
        } else {
            todo.completed.to_owned()
        },
        created_at: todo.created_at,
        updated_at: Some(datetime),
    };
    *todo = payload;
    let json_response = ToDoInfo {
        status: "success".to_string(),
        data: ToDoData { todo: todo.clone() },
    };

    HttpResponse::Ok().json(json_response)
}

#[delete("/todos/{id}")]
pub(crate) async fn delete_todo_handler(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    let mut todos = data.todo_db.lock().unwrap();

    let id = path.into_inner();
    let todo = todos.iter_mut().find(|item| item.id == Some(id.to_owned()));

    if todo.is_none() {
        let error_response = GenericResponse {
            status: "fail".to_string(),
            message: format!("ToDo with id : '{}' not found", id),
        };
        return HttpResponse::NotFound().json(error_response);
    }

    todos.retain(|item| item.id != Some(id.to_owned()));
    HttpResponse::NoContent().finish()
}
