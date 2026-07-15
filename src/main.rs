use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Serialize, Deserialize)]
struct Item {
    id: u32,
    name: String,
}

#[derive(Deserialize)]
struct CreateItem {
    name: String,
}

type Db = Arc<Mutex<HashMap<u32, Item>>>;

#[tokio::main]
async fn main() {
    let db: Db = Arc::new(Mutex::new(HashMap::new()));

    let app = Router::new()
        .route("/items", get(list_items).post(create_item))
        .route("/items/{id}", get(get_item).put(update_item).delete(delete_item))
        .with_state(db);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("Server running on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn list_items(State(db): State<Db>) -> Json<Vec<Item>> {
    let db = db.lock().unwrap();
    Json(db.values().cloned().collect())
}

async fn create_item(
    State(db): State<Db>,
    Json(payload): Json<CreateItem>,
) -> (StatusCode, Json<Item>) {
    let mut db = db.lock().unwrap();
    let id = db.len() as u32 + 1;
    let item = Item { id, name: payload.name };
    db.insert(id, item.clone());
    (StatusCode::CREATED, Json(item))
}

async fn get_item(
    State(db): State<Db>,
    Path(id): Path<u32>,
) -> Result<Json<Item>, StatusCode> {
    let db = db.lock().unwrap();
    db.get(&id).cloned().map(Json).ok_or(StatusCode::NOT_FOUND)
}

async fn update_item(
    State(db): State<Db>,
    Path(id): Path<u32>,
    Json(payload): Json<CreateItem>,
) -> Result<Json<Item>, StatusCode> {
    let mut db = db.lock().unwrap();
    match db.get_mut(&id) {
        Some(item) => {
            item.name = payload.name;
            Ok(Json(item.clone()))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn delete_item(State(db): State<Db>, Path(id): Path<u32>) -> StatusCode {
    let mut db = db.lock().unwrap();
    if db.remove(&id).is_some() {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}
