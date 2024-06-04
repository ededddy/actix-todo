use crate::models::ToDo;
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct GenericResponse {
    pub(crate) status: String,
    pub(crate) message: String,
}

#[derive(Serialize)]
pub(crate) struct ToDoData {
    pub(crate) todo: ToDo,
}

#[derive(Serialize)]
pub(crate) struct ToDoInfo {
    pub(crate) status: String,
    pub(crate) data: ToDoData,
}

#[derive(Serialize)]
pub(crate) struct ToDoListResponse {
    pub(crate) status: String,
    pub(crate) result: usize,
    pub(crate) todos: Vec<ToDo>,
}
