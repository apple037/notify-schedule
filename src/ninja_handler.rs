use axum::{debug_handler, extract::{Query,State}, http::StatusCode, response::IntoResponse, Json};
use reqwest;
use serde_derive::Deserialize;
use serde_json::json;

use crate::models::{AddFilterRequest, DataStore, QueryResponse};
use crate::redis::RedisInstance;

// Structs
#[derive(Deserialize, Debug)]
struct Config {
    ninja: Ninja,
}
#[derive(Deserialize, Debug)]
struct Ninja {
    currency: String,
    item: String,
}
#[derive(Deserialize, Debug)]
pub struct QueryParams {
    league: String,
    category: String,
}

impl QueryParams {
    pub fn new(league: String, category: String) -> QueryParams {
        QueryParams { league, category }
    }
}

// Public functions
// Heartbeat function
pub async fn hb() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}

// Get data from ninja
#[debug_handler]
pub async fn get_data_from_ninja(
    query_params: Query<QueryParams>,
) -> (StatusCode, Json<QueryResponse>) {
    // Trace the request
    tracing::info!(
        "Requesting data from ninja: {} {}",
        query_params.league,
        query_params.category
    );
    // Build the request
    let client = reqwest::Client::new();
    let main_category = get_query_type(query_params.category.to_string());
    let url = format!(
        "{}?league={}&type={}",
        get_base_url_from_toml(query_params.category.to_string()),
        query_params.league,
        query_params.category
    );
    tracing::info!("URL: {}", url);
    let res = client.get(&url).send().await;
    match res {
        Ok(res) => {
            // Parse to QueryResponse
            let content = res.text().await;
            let res_json = serde_json::from_str::<QueryResponse>(&content.unwrap());
            match res_json {
                Ok(res_json) => {
                    //tracing::info!("Response: {:?}", res_json);
                    // Write to Redis
                    write_to_redis(
                        main_category,
                        query_params.category.to_string(),
                        res_json.clone(),
                    );
                    return (StatusCode::OK, Json(res_json));
                }
                Err(e) => {
                    tracing::error!("Parse data error: {}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(QueryResponse::empty()),
                    );
                }
            }
        }
        Err(e) => {
            tracing::error!("Get data error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(QueryResponse::empty()),
            );
        }
    }
}

// Add a dataFilter and return the filterList
pub async fn add_data_filter(Json(payload): Json<AddFilterRequest>) -> (StatusCode, &'static str) {
    let mut redis_instance = get_redis_instance();
    let filter_type = payload.filter_type;
    let main_category = get_query_type(filter_type.to_string());
    let name = payload.name;
    let redis_key = format!("{}:{}:filter", main_category, filter_type);
    let existance = redis_instance.exist_in_list(redis_key.as_str(), name.as_str());
    // if exist skip else add
    if existance.is_ok() && existance.unwrap() == true {
        tracing::info!("Filter {} already exists", redis_key);
        (StatusCode::OK, "Already exists")
    } else {
        let res = redis_instance.push_list(&redis_key, &name);
        match res {
            Ok(_) => {
                tracing::info!("Filter {} {} added", redis_key, name);
                (StatusCode::OK, "Ok!")
            }
            Err(e) => {
                tracing::error!("Add filter error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Add filter failed!")
            }
        }
    }
}
pub async fn get_filter_data(
    query_params: Query<QueryParams>,
) -> (StatusCode, Json<Vec<DataStore>>) {
    let mut redis_instance = get_redis_instance();
    let redis_key = format!(
        "{}:{}",
        get_query_type(query_params.category.to_string()),
        query_params.category
    );
    let res = redis_instance.get_list(redis_key.as_str());
    match res {
        Ok(_) => {
            // Parse to DataStore
            let mut data_list: Vec<DataStore> = Vec::new();
            for data in res.unwrap() {
                let data_json = serde_json::from_str::<DataStore>(&data);
                match data_json {
                    Ok(data_json) => {
                        data_list.push(data_json);
                    }
                    Err(e) => {
                        tracing::error!("Parse data error: {}", e);
                        return (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::new()));
                    }
                }
            }
            return (StatusCode::OK, Json(data_list));
        }
        Err(e) => {
            tracing::error!("Get filter data error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::new()));
        }
    }
}

// Private functions
fn get_query_type(category: String) -> String {
    match category.as_str() {
        "Currency" => "Currency",
        "Fragment" => "Currency",
        _ => "Item",
    }
    .to_string()
}

fn get_base_url_from_toml(category: String) -> String {
    let config_value: String =
        std::fs::read_to_string("Config.toml").expect("Unable to read config file");
    let config: Config = toml::from_str(&config_value).unwrap();
    tracing::info!("Config: {:?}", config);
    match category.as_str() {
        "Currency" => config.ninja.currency,
        "Fragment" => config.ninja.currency,
        _ => config.ninja.item,
    }
}

// Store the query data to redis
fn write_to_redis(main_category: String, category: String, query_res: QueryResponse) {
    // Initialize Redis
    let mut redis_instance = get_redis_instance();
    let mut data_list: Vec<String> = Vec::new();
    for line in query_res.lines {
        // Match the currency
        let redis_key = format!("{}:{}:filter", main_category, category);
        // Check if the key exists
        let existance = redis_instance.exist_in_list(&redis_key, &line.currencyTypeName);
        // if exist update else skip
        match existance {
            Ok(existance) => {
                if existance == true {
                    tracing::info!("{} {} exists", category, redis_key);
                    // if sparkline exists get the totalChange or 0
                    let pay_total_change = match line.paySparkLine {
                        Some(pay_spark_line) => pay_spark_line.totalChange,
                        None => {
                            tracing::info!("{} paySparkLine not exists", redis_key);
                            0.0
                        }
                    };
                    let receive_total_change = match line.receiveSparkLine {
                        Some(receive_spark_line) => receive_spark_line.totalChange,
                        None => {
                            tracing::info!("{} receiveSparkLine not exists", redis_key);
                            0.0
                        }
                    };
                    // Build the data
                    let data = DataStore::new(
                        line.currencyTypeName,
                        line.chaosEquivalent,
                        pay_total_change,
                        receive_total_change,
                    );
                    data_list.push(data.to_json_string());
                } else {
                    tracing::info!("{} {} not exists", category, redis_key);
                    continue;
                }
            }
            Err(e) => {
                tracing::error!("Redis execution error: {}", e);
                continue;
            }
        }
    }
    if data_list.len() == 0 {
        tracing::info!("No data to write");
        return;
    }
    // Write to Redis
    let redis_key = format!("{}:{}", main_category, category);
    let res = redis_instance.set_list_expire(redis_key.as_str(), &data_list, 3600);
    match res {
        Ok(_) => {
            tracing::info!("{} data written to Redis", category);
        }
        Err(e) => {
            tracing::error!("Write to Redis error: {}", e);
        }
    }
}
// Get Redis instance
fn get_redis_instance() -> RedisInstance {
    let redis_instance = RedisInstance::new();
    redis_instance
}

fn get_current_data_list() {

}

// Tests
#[cfg(test)]
#[tokio::test]
async fn test_get_data_from_ninja() {
    let league = String::from("Affliction");
    let category = String::from("Currency");
    let query_params = QueryParams { league, category };
    let (status, response) = get_data_from_ninja(Query(query_params)).await;
    assert_eq!(status, StatusCode::OK);
    assert_ne!(response.lines.len(), 0);
}
