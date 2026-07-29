//! Todo module: data types, OpenAPI spec, and in-memory CRUD handlers.
//!
//! Handler functions are plain synchronous functions that take a `&Store` and return
//! `(status_code, json_body)` — no HTTP framework dependency.

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

#[derive(OpenApi)]
#[openapi(
    paths(list_todos, create_todo, delete_todo, mark_done),
    components(schemas(Todo, TodoError)),
    tags(
        (name = "todo", description = "Todo items management endpoints.")
    )
)]
pub struct TodoApi;

/// Item to complete
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct Todo {
    /// Unique database id for `Todo`
    #[schema(example = 1)]
    pub id: i32,
    /// Description of task to complete
    #[schema(example = "Buy coffee")]
    pub value: String,
    /// Indicates whether task is done or not
    pub done: bool,
}

/// Error that might occur when managing `Todo` items
#[derive(Serialize, Deserialize, ToSchema)]
pub enum TodoError {
    /// Happens when Todo item already exists
    Config(String),
    /// Todo not found from storage
    NotFound(String),
}

pub type Store = Arc<Mutex<Vec<Todo>>>;

/// List todos from in-memory storage.
///
/// List all todos from in memory storage.
#[utoipa::path(
    get,
    path = "",
    responses(
        (status = 200, description = "List all todos successfully", body = [Todo])
    )
)]
pub fn list_todos(store: &Store) -> (u16, String) {
    let todos = store.lock().unwrap().clone();
    (200, serde_json::to_string(&todos).unwrap_or_default())
}

/// Create new todo
///
/// Create new todo to in-memory storage if not exists.
#[utoipa::path(
    post,
    path = "",
    request_body = Todo,
    responses(
        (status = 201, description = "Todo created successfully", body = Todo),
        (status = 409, description = "Todo already exists", body = TodoError, example = json!(TodoError::Config(String::from("id = 1"))))
    )
)]
pub fn create_todo(store: &Store, body: &[u8]) -> (u16, String) {
    let new_todo: Todo = match serde_json::from_slice(body) {
        Ok(t) => t,
        Err(e) => return (400, format!("{{\"error\":\"{}\"}}", e)),
    };
    let mut todos = store.lock().unwrap();
    if let Some(existing) = todos.iter().find(|t| t.id == new_todo.id) {
        let err = TodoError::Config(format!("id = {}", existing.id));
        return (409, serde_json::to_string(&err).unwrap_or_default());
    }
    todos.push(new_todo.clone());
    (201, serde_json::to_string(&new_todo).unwrap_or_default())
}

/// Delete todo by id.
///
/// Delete todo from in-memory storage.
#[utoipa::path(
    delete,
    path = "/{id}",
    responses(
        (status = 200, description = "Todo deleted successfully"),
        (status = 401, description = "Unauthorized to delete Todo"),
        (status = 404, description = "Todo not found", body = TodoError, example = json!(TodoError::NotFound(String::from("id = 1"))))
    ),
    params(
        ("id" = i32, Path, description = "Id of todo item to delete")
    ),
    security(
        ("api_key" = [])
    )
)]
pub fn delete_todo(store: &Store, id: i32, api_key: &str) -> (u16, String) {
    if api_key != "utoipa-rocks" {
        return (401, String::new());
    }
    let mut todos = store.lock().unwrap();
    let old_len = todos.len();
    todos.retain(|t| t.id != id);
    if todos.len() == old_len {
        let err = TodoError::NotFound(format!("id = {id}"));
        (404, serde_json::to_string(&err).unwrap_or_default())
    } else {
        (200, String::new())
    }
}

/// Mark todo done by id
#[utoipa::path(
    put,
    path = "/{id}",
    responses(
        (status = 200, description = "Todo marked done successfully"),
        (status = 404, description = "Todo not found", body = TodoError, example = json!(TodoError::NotFound(String::from("id = 1"))))
    ),
    params(
        ("id" = i32, Path, description = "Id of todo item to mark done")
    )
)]
pub fn mark_done(store: &Store, id: i32) -> (u16, String) {
    let mut todos = store.lock().unwrap();
    match todos.iter_mut().find(|t| t.id == id) {
        Some(todo) => {
            todo.done = true;
            (200, String::new())
        }
        None => {
            let err = TodoError::NotFound(format!("id = {id}"));
            (404, serde_json::to_string(&err).unwrap_or_default())
        }
    }
}
