use axum::extract::Query;
use serde_derive::{Deserialize,Serialize};

use crate::get_profile;
use crate::redis::RedisInstance;
use crate::ninja_handler::{request_data_from_ninja,QueryParams,get_data_from_ninja};
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
#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub ninja: Ninja,
    pub redis: RedisConfig,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct Ninja {
    pub currency: String,
    pub item: String,
}
// Define a struct to hold our Redis configuration values
#[derive(Deserialize, Serialize, Debug)]
pub struct RedisConfig {
    pub host: String,
    pub port: u16,
    pub db: u8,
    pub password: Option<String>,
}


impl Config {
    pub fn to_json_string(&self) -> String {
        let json_string = serde_json::to_string(&self).unwrap();
        json_string
    }
}

pub async fn init_config(profile: &str, redis: &RedisInstance) {
    // Get config file
    let config_name = format!("Config_{}.toml", profile);
    let config_value: String =
        std::fs::read_to_string(&config_name).expect("Unable to read config file");
    let config: Config = toml::from_str(&config_value).unwrap();
    let mut redis = redis.clone();
    let _ = redis.set(&format!("Config"), &config.to_json_string());
    tracing::info!("Init config done");
}

pub async fn get_config(profile: &str) -> Config {
    let mut redis = RedisInstance::new(&profile);
    if redis.exists(&format!("Config")).unwrap() == false {
        init_config(profile, &redis).await;
    }
    let config_string = redis.get(&format!("Config")).unwrap();
    let config: Config = serde_json::from_str(&config_string).unwrap();
    config
}

pub fn get_redis_config(profile: &str) -> RedisConfig {
    // Get config file
    let config_name = format!("Config_{}.toml", profile);
    let config_value: String =
        std::fs::read_to_string(&config_name).expect("Unable to read config file");
    let config: Config = toml::from_str(&config_value).unwrap();
    config.redis
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

pub async fn init_call_before_start() {
    // Call get ninja data before start the server
    let init_data = std::fs::read_to_string("default.toml").expect("Unable to read default file");
    let init_data: InitData = toml::from_str(&init_data).unwrap();
    let league = init_data.league;
    for currency in init_data.currency {
        let query_params = QueryParams::new(league.clone(), currency.name);
        tracing::debug!("Init call with query params: {:?}", query_params);
        let _ = tokio::spawn(async {
            let _ = get_data_from_ninja(Query(query_params)).await;
        });
    }
    for item in init_data.item {
        let query_params = QueryParams::new(league.clone(), item.name);
        tracing::debug!("Init call with query params: {:?}", query_params);
        let _ = tokio::spawn(async {
            let _ = get_data_from_ninja(Query(query_params)).await;
        });
    }
    tracing::info!("Call get ninja data before start the server done");
}

#[cfg(test)]
#[tokio::test]
async fn test_get_init_data() {
    let redis = RedisInstance::new("local");
    init_data(&redis).await;
}