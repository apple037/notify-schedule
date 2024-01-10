use axum::{
    routing::{get, post}, Router,
};
use std::io;
use std::net::SocketAddr;
use tracing::Level;

mod ninja_handler;
mod models;

#[tokio::main]
async fn main() {
        // initialize tracing
        tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_writer(io::stdout)
        .init();

    // build our application with a route
    let app = Router::new()
        // `GET /hb` goes to `handlers::hb`
        .route("/hb", get(ninja_handler::hb))
        // `POST /users` goes to `handlers::create_user`
        .route("/data", get(ninja_handler::get_data_from_ninja));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

