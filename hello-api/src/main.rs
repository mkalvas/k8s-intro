use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(Router::new().route("/", get(hello)).into_make_service())
        .await
        .unwrap();
}

async fn hello() -> &'static str {
    "Hello, World!"
}
