use crate::redis::RedisInstance;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::Json;
use serde_derive::Deserialize;
use crate::ninja_handler::{request_data_from_ninja,QueryParams};
use crate::models::QueryResponse;
#[derive(Deserialize, Debug)]
struct InitData {
    league: String,
    currency: Vec<DefaultData>,
    item: Vec<DefaultData>,
}
#[derive(Deserialize, Debug)]
struct DefaultData {
    name: String,
    default: Vec<String>,
}
#[derive(Deserialize, Debug)]
struct Config {
    ninja: Ninja,
}
#[derive(Deserialize, Debug)]
struct Ninja {
    currency: String,
    item: String,
}


pub async fn init_data(redis: &RedisInstance) {
    // Get initial data from default.toml
    let mut redis = redis.clone();
    let init_data = std::fs::read_to_string("default.toml").expect("Unable to read default file");
    let init_data: InitData = toml::from_str(&init_data).unwrap();

    tracing::info!("InitData: {:?}", init_data);
    // Put initial data into redis for default filter
    for currency in init_data.currency {
        // delete filter data first
        let filter_key = format!("Currency:{}:filter", currency.name);
        let _ = redis.delete(&filter_key);
        for default_value in currency.default {
            let _ = redis.push_list(&filter_key, &default_value);
        }
        // set list expire time
        let _ = redis.set_expire(&filter_key, 3600);
    }
    let mut item_name_list = Vec::new();
    for item in init_data.item {
        // delete filter data first
        let filter_key = format!("Item:{}:filter", item.name);
        let _ = redis.delete(&filter_key);
        for default_value in item.default {
            let _ = redis.push_list(&filter_key, &default_value);
        }
        // set list expire time
        let _ = redis.set_expire(&filter_key, 3600);
        item_name_list.push(item.name);
    }
    // set skip list
    let skip_key = format!("Item:skip");
    let _ = redis.delete(&skip_key);
    let _ = redis.set_list_expire(&skip_key, &item_name_list, 3600);
    // set divine to chaos ratio
    let league = init_data.league;
    let category = String::from("Currency");
    let api_response = request_data_from_ninja(&league, &category).await;
    let api_response = api_response.currency_response.unwrap();
    let api_response = api_response.lines;
    for item in api_response {
        if item.currencyTypeName == "Divine Orb" {
            let _ = redis.push_hash_expire(&format!("{}:D2C",&league),"ratio", &item.chaosEquivalent.to_string(), 3600);
            let _ = redis.push_hash_expire(&format!("{}:D2C",&league),"update_time", &chrono::offset::Utc::now().to_string(), 3600);
        }
    }
    tracing::info!("Init data done");
}

#[cfg(test)]
#[tokio::test]
async fn test_get_init_data() {
    let redis = RedisInstance::new();
    init_data(&redis).await;
}