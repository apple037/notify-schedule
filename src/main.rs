use axum::{
    routing::{get, post},
    Router,
};
use redis::RedisInstance;
use std::io;
use std::net::SocketAddr;
use tracing::Level;
use tokio_cron_scheduler::{JobScheduler};
mod models;
mod ninja_handler;
mod job_handler;
mod discord;
mod redis;
mod init;

#[derive(Clone)]
pub struct AppState {
    scheduler: JobScheduler,
    redis: RedisInstance,
}

impl AppState {
    fn new(scheduler: JobScheduler, redis: RedisInstance) -> AppState {
        AppState {
            scheduler: scheduler,
            redis: redis,
        }
    }
}


#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_writer(io::stdout)
        .init();

    // initialize redis
    let redis = RedisInstance::new();

    // initialize scheduler
    let scheduler = JobScheduler::new().await;
    match scheduler {
        Ok(_) => {
            tracing::info!("Scheduler initialized");
        }
        Err(e) => {
            tracing::error!("Scheduler failed to initialize: {}", e);
            panic!("Scheduler failed to initialize: {}", e);
        }
    }

    // initialize data
    init::init_data(&redis).await;

    // initialize app state
    let app = AppState::new(scheduler.unwrap(), redis);

    // build our application with a route
    let app = Router::new()
        // ninja handler
        .route("/hb", get(ninja_handler::hb))
        .route("/ninja_data", get(ninja_handler::get_data_from_ninja))
        .route("/filter_data", get(ninja_handler::get_filter_data))
        .route("/add_filter", post(ninja_handler::add_data_filter))
        .route("/add_skip_check", post(ninja_handler::add_skip_check))
        // Job handler
        .route("/job/active", post(job_handler::active_probe_job))
        .route("/job/delete", post(job_handler::delete_probe_job))
        .with_state(app);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
