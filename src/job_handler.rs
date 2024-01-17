use axum::{extract::{Query,State}, http::StatusCode, response::IntoResponse, Json};
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::AppState;

// Public functions
// 排程 啟動!
pub async fn active_probe_job(State(state): State<AppState>) -> impl IntoResponse{
    // using app state
    let scheduler = state.scheduler.clone();
    // Add job
    let add = scheduler.add(
        Job::new("1/10 * * * * *", |_uuid, _l| {
            tracing::info!("Job running every 10 seconds");
        }).unwrap()
    ).await;
    match add {
        Ok(_) => {
            println!("Job added");
        }
        Err(e) => {
            println!("Job failed to add: {}", e);
        }
    }
    let scheduler = scheduler.start().await;
    match scheduler {
        Ok(_) => {
            println!("Scheduler started");
        }
        Err(e) => {
            println!("Scheduler failed to start: {}", e);
        }
    }
    "排程 啟動!" 
}

pub async fn delete_probe_job() -> impl IntoResponse{
    "排程 倒了!"    

}

// Private functions

