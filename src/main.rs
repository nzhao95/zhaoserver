use axum::{Router, routing::get};
use std::net::SocketAddr;
use tokio::net::TcpListener;

async fn hello_world() -> &'static str {
    "Hello world!"
}

#[tokio::main]
async fn main() {
    let router = Router::new().route("/", get(hello_world));

    let addr = SocketAddr::from(([127,0,0,1], 8888));
    let tcp = TcpListener::bind(&addr).await.unwrap();

    axum::serve(tcp, router).await.unwrap();
}