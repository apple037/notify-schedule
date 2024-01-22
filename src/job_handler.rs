use axum::{extract::{Query,State}, response::IntoResponse};
use tokio_cron_scheduler::Job;
use uuid::Uuid;
use std::collections::HashMap;
use serde_derive::Deserialize;

use crate::ninja_handler::{QueryParams,get_data_from_ninja, self};
use crate::AppState;
use crate::discord::send_message_to_channel;

#[derive(Deserialize, Debug)]
struct InitData {
    league: String,
}
impl InitData {
    pub fn get_league(&self) -> String {
        self.league.clone()
    }
}

// Public functions
// 排程 啟動!
pub async fn active_probe_job(State(state): State<AppState>) -> impl IntoResponse{
    // using app state
    let scheduler = state.scheduler.clone();

    // Add job
    let add = scheduler.add(
        Job::new("0 * * * * *", |_uuid, _l| {
            // Get initial data from default.toml
            let init_data = std::fs::read_to_string("default.toml").expect("Unable to read default file");
            let init_data: InitData = toml::from_str(&init_data).unwrap();
            tracing::info!("Job running every hour");
            // Get the refresh key map
            let refresh_key_map = get_the_refresh_key_map();
            tracing::debug!("Refresh key map: {:?}", refresh_key_map);
            // loop the refresh key map
            for (main_type, sub_type_list) in refresh_key_map {
                // loop the sub type list
                for sub_type in sub_type_list {
                    // Refresh data
                    let query_params = QueryParams::new(init_data.get_league(), sub_type.clone());
                    tracing::debug!("Query params: {:?}", query_params);
                    let _ = tokio::spawn(async {
                        let _ = get_data_from_ninja(Query(query_params)).await;
                    });
                }
                // Get the output data from redis
                let output = ninja_handler::get_format_output(&main_type);
                // tracing::debug!("Output: {:?}", output);
                // Send the output data to discord
                let _ = tokio::spawn(async {
                    let _ = send_message_to_channel(output).await;
                });
            }
    
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
fn get_the_refresh_key_map() -> HashMap<String, Vec<String>> {
    let mut refresh_key_map = HashMap::new();
    let mut redis_instance = crate::redis::RedisInstance::new();
    let main_type_list = redis_instance.get_all_keys_name("Data:*");
    let main_type_list = main_type_list.unwrap();
    println!("Main type list: {:?}", main_type_list);
    for main_type in main_type_list {
        let sub_type_key = format!("Data:{}:*", &main_type);
        let sub_type_list = redis_instance.get_all_keys_name(&sub_type_key);
        let sub_type_list = sub_type_list.unwrap();
        // push into refresh_key_map
        refresh_key_map.insert(main_type, sub_type_list);
    }
    refresh_key_map
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_the_refresh_key_map() {
        let refresh_key_map = get_the_refresh_key_map();
        println!("{:?}", refresh_key_map);
    }
}
