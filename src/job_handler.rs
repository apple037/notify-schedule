use axum::{extract::{Query,State}, response::IntoResponse};
use reqwest::StatusCode;
use tokio_cron_scheduler::Job;
use uuid::Uuid;

use crate::ninja_handler::{QueryParams,get_data_from_ninja};
use crate::AppState;

// Public functions
// 排程 啟動!
pub async fn active_probe_job(State(state): State<AppState>) -> impl IntoResponse{
    // using app state
    let scheduler = state.scheduler.clone();
    // Add job
    let add = scheduler.add(
        Job::new("0 0 * * * *", |_uuid, _l| {
            tracing::info!("Job running every hour");
            // Refresh currency data
            let query_params = QueryParams::new("Affliction".to_string(), "Currency".to_string());
            tracing::info!("Query params: {:?}", query_params);
            let _ = tokio::spawn(async {
                let _ = get_data_from_ninja(Query(query_params)).await;
            });
            // Refresh essence data
            let query_params = QueryParams::new("Affliction".to_string(), "Essence".to_string());
            let _ = tokio::spawn(async {
                let _ = get_data_from_ninja(Query(query_params)).await;
            });
        }).expect("Failed to create job"),
    ).await;
    match add {
        Ok(_) => {
            let uuid = add.unwrap();
            let uuid = uuid.to_string();
            // Add job to redis
            let mut redis = state.redis.clone();
            let _ = redis.set_with_expire("probe_job", &uuid, 3600);
            println!("Job added with uuid: {}", uuid);
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

pub async fn delete_probe_job(State(state): State<AppState>) -> impl IntoResponse{
    // using app state
    let scheduler = state.scheduler.clone();
    // Delete job
    let mut redis = state.redis.clone();
    let uuid = redis.get("probe_job").unwrap();
    match uuid {
        _ => {
            if uuid.is_empty() {
                return "排程 沒有啟動!";
            }
            let uuid = Uuid::parse_str(&uuid).unwrap();
            let delete = scheduler.remove(&uuid).await;
            match delete {
                Ok(_) => {
                    tracing::info!("Job {} deleted", uuid);
                    let _ = redis.delete("probe_job");
                }
                Err(e) => {
                    tracing::error!("Job {} failed to delete: {}", uuid, e);
                }
            }
        }
    }
    "排程 倒了!"    

}

// Private functions

