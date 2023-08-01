use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use models::prelude::*;
use sea_orm::{prelude::*, Database, DatabaseConnection};
use std::sync::Arc;

mod models;

#[tokio::main]
async fn main() {
    let state = create_state().await;
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(
            Router::new()
                .route("/users", get(get_users))
                .with_state(state)
                .into_make_service(),
        )
        .await
        .unwrap();
}

type AppState = State<Arc<AppContext>>;

struct AppContext {
    pub db: DatabaseConnection,
}

async fn create_state() -> Arc<AppContext> {
    let db = Database::connect(connection_string()).await.unwrap(); // panic if DB is unreachable todo anyhow stuff
    Arc::new(AppContext { db })
}

fn connection_string() -> String {
    let host = std::env::var("DB_HOST").unwrap_or("localhost".to_string());
    let password = std::env::var("DB_PASS").unwrap_or("testpassword".to_string());
    format!("mysql://root:{password}@{host}:3306/test")
}

async fn get_users(State(ctx): AppState) -> impl IntoResponse {
    match Users::find().into_json().all(&ctx.db).await {
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        Ok(records) => Json(records).into_response(),
    }
}
