use crate::{
    auth_validator::{encode_user, ok_validator},
    database::{get_by_id, get_collection},
    models::{AppState, CreateToDoSchema, QueryOptions, ToDo, UpdateToDoSchema, User, UserRequest},
    responses::{GenericResponse, ToDoData, ToDoInfo, ToDoListResponse},
};
use actix_web::{delete, get, patch, post, web, HttpResponse, Responder};
use actix_web_httpauth::middleware::HttpAuthentication;
use chrono::Utc;
use futures::stream::TryStreamExt;
use mongodb::{
    bson::doc,
    options::{FindOneAndUpdateOptions, FindOptions, ReturnDocument},
};

pub(crate) fn healthchecks_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("healthchecks").route(web::get().to(healthcheck_handler)));
}

pub(crate) fn web_service_config(cfg: &mut web::ServiceConfig) {
    let scope = web::scope("/api")
        .wrap(HttpAuthentication::bearer(ok_validator))
        .service(todos_list_handler)
        .service(create_todo_handler)
        .service(get_todo_handler)
        .service(update_todo_handler)
        .service(delete_todo_handler);
    cfg.service(scope);
}

pub(crate) fn auth_service_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .service(register_user_handler)
            .service(login_user_handler),
    );
}

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
    let todos: mongodb::Collection<ToDo> = get_collection(
        &state.connection_pool,
        &state.database_name,
        &state.collection_name,
    );

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
    let todos: mongodb::Collection<ToDo> = get_collection(
        &state.connection_pool,
        &state.database_name,
        &state.collection_name,
    );

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
    let todos: mongodb::Collection<ToDo> = get_collection(
        &state.connection_pool,
        &state.database_name,
        &state.collection_name,
    );

    let id = path.into_inner();
    let todo = get_by_id(&todos, &id).await;

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

#[patch("/todos/{id}")]
pub(crate) async fn update_todo_handler(
    path: web::Path<String>,
    body: web::Json<UpdateToDoSchema>,
    state: web::Data<AppState>,
) -> impl Responder {
    let todos: mongodb::Collection<ToDo> = get_collection(
        &state.connection_pool,
        &state.database_name,
        &state.collection_name,
    );

    let id = path.into_inner();
    let todo = get_by_id(&todos, &id).await;

    if todo.is_none() {
        let error_response = GenericResponse {
            status: "fail".to_string(),
            message: format!("ToDo with id : '{}' not found", id),
        };
        return HttpResponse::NotFound().json(error_response);
    }

    let todo = todo.unwrap();

    let datetime = Utc::now();
    let title = body.title.to_owned().unwrap_or(todo.title.clone());
    let update_result = todos
        .find_one_and_update(
            doc! { "_id":id.clone()},
            doc! {
                "$set" : doc!{
                    "title": title,
                    "content": if !body.content.is_none() {
                        body.content.clone()
                    } else {
                        todo.content.to_owned()
                    },
                    "completed": if !body.completed.is_none() {
                        body.completed.unwrap()
                    } else {
                        todo.completed.to_owned()
                    },
                    "updated_at": datetime.to_string(),
                }
            },
            FindOneAndUpdateOptions::builder()
                .return_document(ReturnDocument::After)
                .build(),
        )
        .await
        .expect("error updating the todo");
    if let Some(updated_todo) = update_result {
        let json_response = ToDoInfo {
            status: "success".to_string(),
            data: ToDoData { todo: updated_todo },
        };
        return HttpResponse::Ok().json(json_response);
    } else {
        let error_response = GenericResponse {
            status: "fail".to_string(),
            message: format!("ToDo with id : '{}' not found", &id),
        };
        return HttpResponse::NotFound().json(error_response);
    }
}

#[delete("/todos/{id}")]
pub(crate) async fn delete_todo_handler(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let todos: mongodb::Collection<ToDo> = get_collection(
        &state.connection_pool,
        &state.database_name,
        &state.collection_name,
    );
    let id = path.into_inner();
    let _delete_result = todos
        .delete_one(doc! { "_id": id}, None)
        .await
        .expect("Error deleting targeted todo");

    HttpResponse::NoContent().finish()
}

#[post("/register")]
pub(crate) async fn register_user_handler(
    reg: web::Json<UserRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let users: mongodb::Collection<User> = get_collection(
        &state.connection_pool,
        &state.database_name,
        &state.user_collection_name,
    );
    let user: User = reg.into_inner().into();
    let token = encode_user(&user).expect("error encoding user to jwt token");
    match users.insert_one(&user, None).await {
        Ok(_) => HttpResponse::Ok().json(GenericResponse {
            status: "success".to_string(),
            message: token,
        }),
        Err(error) => HttpResponse::InternalServerError().json(GenericResponse {
            status: "error".to_string(),
            message: error.to_string(),
        }),
    }
}

#[post("/login")]
pub(crate) async fn login_user_handler(
    reg: web::Json<UserRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let users: mongodb::Collection<User> = get_collection(
        &state.connection_pool,
        &state.database_name,
        &state.user_collection_name,
    );
    let reg = reg.into_inner();
    let user_lookup = users
        .find_one(
            doc! {
                "username": reg.username,
                "password": reg.password,
            },
            None,
        )
        .await
        .expect("error finding user");

    match user_lookup {
        Some(user) => {
            let token = encode_user(&user).expect("error encoding user to jwt token");
            return HttpResponse::Ok().json(GenericResponse {
                status: "success".to_string(),
                message: token,
            });
        }
        None => HttpResponse::NotFound().json(GenericResponse {
            status: "error".to_string(),
            message: "username / password incorret".to_string(),
        }),
    }
}
